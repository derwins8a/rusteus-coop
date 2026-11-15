//! Custom GPUI-compatible HTTP client implementation for NucleusCoop.
//!
//! Provides image asset downloading with local disk caching to prevent
//! redundant network traffic and ensure responsive UI pagination.

use std::path::PathBuf;
use std::sync::OnceLock;
use gpui::http_client::{self, AsyncBody, HttpClient, Url};
use futures::future::BoxFuture;
use futures::FutureExt;
use tracing::debug;

static TOKIO_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub fn get_tokio_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("nucleus-http-worker")
            .build()
            .expect("Failed to create Tokio HTTP runtime for image asset loader")
    })
}

fn fnv1a_hash_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// NucleusCoop HTTP client supporting GPUI image asset loading and local disk caching.
pub struct NucleusHttpClient {
    client: reqwest::Client,
    cache_dir: Option<PathBuf>,
}

impl NucleusHttpClient {
    /// Creates a new `NucleusHttpClient` with default cache directory pointing to gui/covers (C# parity).
    pub fn new() -> Self {
        let cache_dir = nucleus_common::Globals::try_get()
            .map(|g| g.install_root.join("gui").join("covers"))
            .or_else(|| {
                dirs::cache_dir().map(|c| c.join("NucleusCoop").join("gui").join("covers"))
            });

        if let Some(ref dir) = cache_dir {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Some(g) = nucleus_common::Globals::try_get() {
            let _ = std::fs::create_dir_all(g.install_root.join("gui").join("screenshots"));
            let _ = std::fs::create_dir_all(g.install_root.join("gui").join("icons"));
            let _ = std::fs::create_dir_all(g.install_root.join("gui").join("descriptions"));
        }

        let client = reqwest::Client::builder()
            .user_agent("NucleusCoop/0.1.0 (Windows; x64)")
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .unwrap_or_default();

        Self { client, cache_dir }
    }

    /// Computes a deterministic cache file path for a given URL.
    pub fn cache_path_for_url(&self, url_str: &str) -> Option<PathBuf> {
        let dir = self.cache_dir.as_ref()?;
        let hash = format!("{:016x}", fnv1a_hash_64(url_str.as_bytes()));

        let ext = if url_str.contains(".png") {
            "png"
        } else if url_str.contains(".webp") {
            "webp"
        } else {
            "jpg"
        };
        Some(dir.join(format!("{}.{}", hash, ext)))
    }
}

impl Default for NucleusHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for NucleusHttpClient {
    fn user_agent(&self) -> Option<&http_client::http::HeaderValue> {
        None
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }

    fn send(
        &self,
        req: http_client::http::Request<AsyncBody>,
    ) -> BoxFuture<'static, anyhow::Result<http_client::http::Response<AsyncBody>>> {
        let uri_str = req.uri().to_string();
        let cache_file = self.cache_path_for_url(&uri_str);

        // 1. Check local disk cache first in gui/covers
        if let Some(ref path) = cache_file {
            if path.exists() {
                if let Ok(bytes) = std::fs::read(path) {
                    debug!("Serving image from local disk cache: {}", path.display());
                    let resp = http_client::http::Response::builder()
                        .status(200)
                        .header("Content-Type", "image/jpeg")
                        .body(AsyncBody::from(bytes));
                    if let Ok(r) = resp {
                        return futures::future::ready(Ok(r)).boxed();
                    }
                }
            }
        }

        // 1b. Check historical cache location (content/cache/covers)
        if let Some(g) = nucleus_common::Globals::try_get() {
            let alt_cache = g.install_root.join("content").join("cache").join("covers");
            let hash = format!("{:016x}", fnv1a_hash_64(uri_str.as_bytes()));
            for ext in ["jpg", "png", "webp", "jpeg"] {
                let candidate = alt_cache.join(format!("{}.{}", hash, ext));
                if candidate.exists() {
                    if let Ok(bytes) = std::fs::read(&candidate) {
                        debug!("Serving image from historical cache: {}", candidate.display());
                        let resp = http_client::http::Response::builder()
                            .status(200)
                            .header("Content-Type", "image/jpeg")
                            .body(AsyncBody::from(bytes));
                        if let Ok(r) = resp {
                            return futures::future::ready(Ok(r)).boxed();
                        }
                    }
                }
            }
        }

        // 2. Otherwise download asynchronously using Tokio runtime
        let client = self.client.clone();
        let rt = get_tokio_runtime();
        let (tx, rx) = futures::channel::oneshot::channel();

        rt.spawn(async move {
            let res: anyhow::Result<http_client::http::Response<AsyncBody>> = async {
                let (parts, _body) = req.into_parts();
                let mut req_builder = client.request(parts.method, parts.uri.to_string());
                for (name, value) in parts.headers.iter() {
                    req_builder = req_builder.header(name.as_str(), value.as_bytes());
                }

                let response = req_builder.send().await?;
                let status = response.status().as_u16();
                let headers = response.headers().clone();
                let bytes = response.bytes().await?;

                // If successful download and we have a cache file path, save to disk cache
                if status == 200 {
                    if let Some(ref path) = cache_file {
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let _ = std::fs::write(path, &bytes);
                        debug!("Cached downloaded image to: {}", path.display());
                    }
                }

                let mut resp_builder = http_client::http::Response::builder().status(status);
                for (name, val) in headers.iter() {
                    resp_builder = resp_builder.header(name.as_str(), val.as_bytes());
                }
                let body = AsyncBody::from(bytes.to_vec());
                let final_resp = resp_builder.body(body)?;
                Ok(final_resp)
            }.await;

            let _ = tx.send(res);
        });

        async move {
            match rx.await {
                Ok(res) => res,
                Err(_) => Err(anyhow::anyhow!("HTTP request worker task dropped")),
            }
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::AsyncReadExt;

    #[test]
    fn test_nucleus_http_client_creation() {
        let client = NucleusHttpClient::new();
        let cache_path = client.cache_path_for_url("https://images.igdb.com/igdb/image/upload/t_cover_big/co1x7j.jpg");
        assert!(cache_path.is_some());
        let path = cache_path.unwrap();
        assert!(path.to_string_lossy().ends_with(".jpg"));
    }

    #[test]
    fn test_cache_path_extensions() {
        let client = NucleusHttpClient::new();
        let png = client.cache_path_for_url("https://example.com/cover.png").unwrap();
        assert!(png.to_string_lossy().ends_with(".png"));

        let webp = client.cache_path_for_url("https://example.com/art.webp").unwrap();
        assert!(webp.to_string_lossy().ends_with(".webp"));

        let jpg = client.cache_path_for_url("https://example.com/banner.jpeg").unwrap();
        assert!(jpg.to_string_lossy().ends_with(".jpg"));
    }

    #[tokio::test]
    async fn test_nucleus_http_client_disk_cache_serving() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_cache_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let mut client = NucleusHttpClient::new();
        client.cache_dir = Some(temp_dir.clone());

        let url = "https://example.com/fake_cover.jpg";
        let cache_file = client.cache_path_for_url(url).expect("cache path");
        let fake_image_data = b"\xFF\xD8\xFF\xE0\x00\x10JFIFfakeimagedata";
        std::fs::write(&cache_file, fake_image_data).expect("write fake cache");

        // Request through GPUI HttpClient interface
        let req = http_client::http::Request::builder()
            .uri(url)
            .body(AsyncBody::empty())
            .expect("valid request");

        let mut resp = client.send(req).await.expect("send must succeed from cache");
        assert_eq!(resp.status().as_u16(), 200);

        let mut body = Vec::new();
        resp.body_mut().read_to_end(&mut body).await.expect("read body");
        assert_eq!(body, fake_image_data);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

