//! Asynchronous HTTP client for communicating with the SplitScreen.Me Hub API.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use crate::types::{
    HubComment, HubHandler, HubHandlerDetail, HubHandlerSummary, HubHandlersResponse,
    HubPackage, HubPackageResponse, HubPackagesResponse, HubScreenshot, HubScreenshotsResponse,
    HubVersionInfo,
};
use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::file_util;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use zip::ZipArchive;

/// Metadata sidecar persisted next to a Hub-installed handler script.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubMetaSidecar {
    /// Upstream Hub handler ID.
    pub handler_id: String,
    /// Upstream Hub package ID installed locally.
    pub package_id: String,
    /// Installed handler revision number.
    pub installed_version: i32,
}

/// HTTP client for interacting with online handler repositories and community services.
#[derive(Clone)]
pub struct HubClient {
    /// Underlying HTTP client instance.
    client: Client,
    /// Base URL endpoint of the hub API service.
    base_url: String,
    /// Base URL for the CDN.
    cdn_base_url: String,
}

impl HubClient {
    /// Creates a new `HubClient` targeting an explicit API/CDN base URL.
    pub fn with_base_url(base_url: &str) -> Self {
        Self::new(Some(base_url))
    }

    /// Creates a new `HubClient` with an optional custom base URL.
    pub fn new(base_url: Option<&str>) -> Self {
        let (api_base, cdn_base) = match base_url {
            Some(u) => {
                let trimmed = u.trim_end_matches('/');
                if trimmed.ends_with("/api/v1") {
                    let cdn = trimmed.trim_end_matches("/api/v1");
                    (trimmed.to_string(), cdn.to_string())
                } else {
                    (format!("{}/api/v1", trimmed), trimmed.to_string())
                }
            }
            None => (
                "https://hub.splitscreen.me/api/v1".to_string(),
                "https://hub.splitscreen.me".to_string(),
            ),
        };

        Self {
            client: Client::builder()
                .user_agent("NucleusCoop-Rust/0.1.0 (https://github.com/SplitScreen-Me)")
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: api_base,
            cdn_base_url: cdn_base,
        }
    }

    /// Returns the API base URL of the client.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the CDN base URL of the client.
    pub fn cdn_base_url(&self) -> &str {
        &self.cdn_base_url
    }

    /// Fetches all community handlers (up to 500) from the Hub catalog.
    ///
    /// Endpoint: `GET /api/v1/allhandlers`
    pub async fn get_all_handlers(&self) -> NucleusResult<Vec<HubHandler>> {
        let url = format!("{}/allhandlers", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NucleusError::Network(format!("Failed to connect to Hub: {}", e)))?;

        if !resp.status().is_success() {
            return Err(NucleusError::Network(format!(
                "Hub allhandlers returned status code: {}",
                resp.status()
            )));
        }

        let container = resp
            .json::<HubHandlersResponse>()
            .await
            .map_err(|e| NucleusError::Network(format!("Failed to parse allhandlers response: {}", e)))?;

        Ok(container.handlers)
    }

fn percent_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.as_bytes() {
        match *b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*b as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

    /// Searches for community game handlers matching the specified query string.
    /// If query is empty, fetches all available handlers.
    ///
    /// Endpoint: `GET /api/v1/handlers/:search_text` or `GET /api/v1/allhandlers`
    pub async fn search_raw_handlers(&self, query: &str) -> NucleusResult<Vec<HubHandler>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return self.get_all_handlers().await;
        }

        let encoded = Self::percent_encode(trimmed);
        let url = format!("{}/handlers/{}", self.base_url, encoded);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NucleusError::Network(format!("Hub search failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(NucleusError::Network(format!(
                "Hub search returned status code: {}",
                resp.status()
            )));
        }

        let container = resp
            .json::<HubHandlersResponse>()
            .await
            .map_err(|e| NucleusError::Network(format!("Failed to decode search response: {}", e)))?;

        Ok(container.handlers)
    }

    /// Searches for game handlers matching the query and returns `HubHandlerSummary` items.
    pub async fn search_handlers(&self, query: &str) -> NucleusResult<Vec<HubHandlerSummary>> {
        let raw = self.search_raw_handlers(query).await?;
        Ok(raw.into_iter().map(|h| h.to_summary()).collect())
    }

    /// Retrieves specific handler information by its identifier.
    ///
    /// Endpoint: `GET /api/v1/handler/:handler_id`
    pub async fn get_handler(&self, handler_id: &str) -> NucleusResult<Option<HubHandler>> {
        let url = format!("{}/handler/{}", self.base_url, handler_id.trim());
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NucleusError::Network(format!("Failed to get handler {}: {}", handler_id, e)))?;

        if !resp.status().is_success() {
            return Err(NucleusError::Network(format!(
                "Hub get_handler returned status code: {}",
                resp.status()
            )));
        }

        let text = resp
            .text()
            .await
            .map_err(|e| NucleusError::Network(e.to_string()))?;

        if text.trim() == "{}" || text.trim().is_empty() {
            return Ok(None);
        }

        let container: HubHandlersResponse = serde_json::from_str(&text)
            .map_err(|e| NucleusError::Network(format!("Failed to parse handler info: {}", e)))?;

        Ok(container.handlers.into_iter().next())
    }

    /// Retrieves available package versions for a specific handler.
    ///
    /// Endpoint: `GET /api/v1/packages/:handler_id`
    pub async fn get_packages(&self, handler_id: &str) -> NucleusResult<Vec<HubPackage>> {
        let url = format!("{}/packages/{}", self.base_url, handler_id.trim());
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NucleusError::Network(format!("Failed to get packages: {}", e)))?;

        if !resp.status().is_success() {
            return Err(NucleusError::Network(format!(
                "Hub packages returned status code: {}",
                resp.status()
            )));
        }

        let text = resp
            .text()
            .await
            .map_err(|e| NucleusError::Network(e.to_string()))?;

        if text.trim() == "{}" || text.trim().is_empty() {
            return Ok(Vec::new());
        }

        if let Ok(wrapper) = serde_json::from_str::<crate::types::HubPackagesResponse>(&text) {
            Ok(wrapper.packages)
        } else if let Ok(packages) = serde_json::from_str::<Vec<HubPackage>>(&text) {
            Ok(packages)
        } else {
            Ok(Vec::new())
        }
    }

    /// Retrieves specific package metadata by its package ID.
    ///
    /// Endpoint: `GET /api/v1/package/:package_id`
    pub async fn package_details(&self, package_id: &str) -> NucleusResult<HubPackage> {
        let url = format!("{}/package/{}", self.base_url, package_id.trim());
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NucleusError::Network(format!("Failed to get package details: {}", e)))?;

        if !resp.status().is_success() {
            return Err(NucleusError::Network(format!(
                "Hub package details returned status code: {}",
                resp.status()
            )));
        }

        let text = resp
            .text()
            .await
            .map_err(|e| NucleusError::Network(e.to_string()))?;

        if let Ok(wrapper) = serde_json::from_str::<HubPackageResponse>(&text) {
            if let Some(pkg) = wrapper.package {
                return Ok(pkg);
            }
        }
        if let Ok(wrapper) = serde_json::from_str::<HubPackagesResponse>(&text) {
            if let Some(pkg) = wrapper.packages.into_iter().next() {
                return Ok(pkg);
            }
        }
        if let Ok(pkg) = serde_json::from_str::<HubPackage>(&text) {
            return Ok(pkg);
        }

        Err(NucleusError::Network(format!(
            "Failed to parse package details JSON: {}",
            text
        )))
    }

    /// Retrieves IGDB screenshots associated with a game handler.
    ///
    /// Endpoint: `GET /api/v1/screenshots/:handler_id`
    pub async fn get_screenshots(&self, handler_id: &str) -> NucleusResult<Vec<HubScreenshot>> {
        let url = format!("{}/screenshots/{}", self.base_url, handler_id.trim());
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NucleusError::Network(format!("Failed to get screenshots: {}", e)))?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let text = resp
            .text()
            .await
            .map_err(|e| NucleusError::Network(e.to_string()))?;

        if text.trim() == "{}" || text.trim().is_empty() {
            return Ok(Vec::new());
        }

        if let Ok(res) = serde_json::from_str::<HubScreenshotsResponse>(&text) {
            Ok(res.screenshots)
        } else {
            Ok(Vec::new())
        }
    }

    /// Retrieves user comments about a handler.
    ///
    /// Endpoint: `GET /api/v1/comments/:handler_id`
    pub async fn get_comments(&self, handler_id: &str) -> NucleusResult<Vec<HubComment>> {
        let url = format!("{}/comments/{}", self.base_url, handler_id.trim());
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NucleusError::Network(format!("Failed to get comments: {}", e)))?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let text = resp
            .text()
            .await
            .map_err(|e| NucleusError::Network(e.to_string()))?;

        if text.trim() == "{}" || text.trim().is_empty() {
            return Ok(Vec::new());
        }

        if let Ok(wrapper) = serde_json::from_str::<crate::types::HubCommentsResponse>(&text) {
            Ok(wrapper.comments)
        } else if let Ok(comments) = serde_json::from_str::<Vec<HubComment>>(&text) {
            Ok(comments)
        } else {
            Ok(Vec::new())
        }
    }

    /// Retrieves detailed metadata, version history, and download URLs for a specific handler.
    pub async fn get_handler_details(&self, handler_id: &str) -> NucleusResult<HubHandlerDetail> {
        let handler = self.get_handler(handler_id).await?.ok_or_else(|| {
            NucleusError::Network(format!("Handler '{}' not found on community Hub", handler_id))
        })?;

        let packages = self.get_packages(handler_id).await.unwrap_or_default();
        let comments = self.get_comments(handler_id).await.unwrap_or_default();

        let mut versions = Vec::new();
        for p in &packages {
            if let Some(ref meta) = p.meta {
                let ver_num = crate::types::value_to_i32(&meta.handler_version).unwrap_or(1);
                let pkg_id = &p.id;
                let dl = format!(
                    "{}/cdn/storage/packages/{}/original/handler-{}-v{}.nc?download=true",
                    self.cdn_base_url, pkg_id, handler_id, ver_num
                );
                versions.push(HubVersionInfo {
                    version: format!("v{}", ver_num),
                    changelog: meta.release_description.clone().unwrap_or_default(),
                    release_date: meta.release_date.clone().unwrap_or_default(),
                    download_url: dl,
                });
            }
        }

        let download_url = handler.cdn_download_url().unwrap_or_else(|| {
            versions.first().map(|v| v.download_url.clone()).unwrap_or_default()
        });

        let game_cover_url = handler.cover_image_url("cover_big");

        Ok(HubHandlerDetail {
            id: handler.id.clone(),
            game: crate::types::HubGameInfo {
                name: handler.game_name.clone().unwrap_or_else(|| handler.display_title().to_string()),
                steam_id: None,
                publisher: None,
                banner_url: game_cover_url.clone(),
                icon_url: game_cover_url,
            },
            author: handler.author().to_string(),
            current_version: format!("v{}", handler.version_number()),
            description: handler.description.clone().unwrap_or_default(),
            versions,
            comments,
            download_url,
            current_package: handler.current_package.clone(),
        })
    }

    /// Downloads a package archive directly from the Hub CDN using package ID, handler ID, and version.
    ///
    /// Endpoint: `GET /cdn/storage/packages/:package_id/original/handler-{handler_id}-v{version}.nc?download=true`
    pub async fn download_package(
        &self,
        package_id: &str,
        handler_id: &str,
        version: i32,
    ) -> NucleusResult<Vec<u8>> {
        self.download_package_with_progress(package_id, handler_id, version, |_| {}).await
    }

    /// Downloads a package archive with continuous progress updates (0.0 to 1.0).
    pub async fn download_package_with_progress<F>(
        &self,
        package_id: &str,
        handler_id: &str,
        version: i32,
        on_progress: F,
    ) -> NucleusResult<Vec<u8>>
    where
        F: FnMut(f32) + Send + 'static,
    {
        self.download_package_with_progress_and_size(package_id, handler_id, version, None, on_progress).await
    }

    /// Downloads a package archive with continuous progress updates, optionally using known package size as fallback.
    pub async fn download_package_with_progress_and_size<F>(
        &self,
        package_id: &str,
        handler_id: &str,
        version: i32,
        expected_size: Option<u64>,
        mut on_progress: F,
    ) -> NucleusResult<Vec<u8>>
    where
        F: FnMut(f32) + Send + 'static,
    {
        let url = format!(
            "{}/cdn/storage/packages/{}/original/handler-{}-v{}.nc?download=true",
            self.cdn_base_url,
            package_id.trim(),
            handler_id.trim(),
            version
        );

        // Immediate start feedback
        on_progress(0.08);

        let mut resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NucleusError::Network(format!("Failed to download handler package: {}", e)))?;

        if !resp.status().is_success() {
            return Err(NucleusError::Network(format!(
                "Package download failed with status: {} (URL: {})",
                resp.status(),
                url
            )));
        }

        let total_size = resp.content_length().or(expected_size);
        let mut downloaded: u64 = 0;
        let mut buffer = Vec::with_capacity(total_size.unwrap_or(65536) as usize);

        on_progress(0.15);

        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|e| NucleusError::Network(format!("Failed to read stream chunk: {}", e)))?
        {
            downloaded += chunk.len() as u64;
            buffer.extend_from_slice(&chunk);
            if let Some(total) = total_size {
                if total > 0 {
                    let progress = (downloaded as f32 / total as f32).clamp(0.15, 0.95);
                    on_progress(progress);
                }
            } else {
                let simulated = (downloaded as f32 / (downloaded + 4096) as f32).clamp(0.15, 0.90);
                on_progress(simulated);
            }
        }

        on_progress(1.0);
        Ok(buffer)
    }

    /// Downloads the archive package bytes for a handler by resolving its current package and version.
    pub async fn download_handler(&self, handler_id: &str) -> NucleusResult<Vec<u8>> {
        self.download_handler_with_progress(handler_id, |_| {}).await
    }

    /// Resolves the latest package ID, version number, and optional byte size for a handler.
    async fn resolve_latest_package(
        &self,
        handler_id: &str,
    ) -> NucleusResult<(String, String, i32, Option<u64>)> {
        let packages = self.get_packages(handler_id).await.unwrap_or_default();
        let handler_opt = self.get_handler(handler_id).await.ok().flatten();

        if let Some(ref h) = handler_opt {
            if let Some(ref pkg_id) = h.current_package {
                let pkg_match = packages.iter().find(|p| p.id == *pkg_id);
                let expected_size = pkg_match.and_then(|p| p.size);
                return Ok((
                    h.id.clone(),
                    pkg_id.clone(),
                    h.version_number(),
                    expected_size,
                ));
            }
        }

        if let Some(pkg) = packages.first() {
            let ver = pkg
                .meta
                .as_ref()
                .and_then(|m| crate::types::value_to_i32(&m.handler_version))
                .unwrap_or(1);
            return Ok((handler_id.to_string(), pkg.id.clone(), ver, pkg.size));
        }

        Err(NucleusError::Network(format!(
            "Could not find download package for handler ID '{}'",
            handler_id
        )))
    }

    /// Downloads the archive package bytes with progress updates.
    pub async fn download_handler_with_progress<F>(
        &self,
        handler_id: &str,
        on_progress: F,
    ) -> NucleusResult<Vec<u8>>
    where
        F: FnMut(f32) + Send + 'static,
    {
        let (resolved_handler_id, pkg_id, ver, expected_size) =
            self.resolve_latest_package(handler_id).await?;
        self.download_package_with_progress_and_size(
            &pkg_id,
            &resolved_handler_id,
            ver,
            expected_size,
            on_progress,
        )
        .await
    }

    /// Extracts an in-memory zip archive (downloaded .nc package) into the target directory,
    /// isolating handlers and renaming 'handler.js' to avoid collisions.
    pub fn extract_package_isolated(
        archive_bytes: &[u8],
        destination_dir: &Path,
        fallback_id: Option<&str>,
    ) -> NucleusResult<Vec<PathBuf>> {
        file_util::ensure_dir(destination_dir)?;

        let mut archive = ZipArchive::new(Cursor::new(archive_bytes))
            .map_err(|e| NucleusError::Other(format!("Invalid zip archive: {}", e)))?;

        // 1. Inspect archive to find handler.js and detect Game.GameName
        let mut detected_name: Option<String> = None;

        for i in 0..archive.len() {
            if let Ok(mut entry) = archive.by_index(i) {
                let name = entry.name().to_string();
                if name.eq_ignore_ascii_case("handler.js") || name.ends_with("/handler.js") || name.ends_with("\\handler.js") {
                    let mut script_content = String::new();
                    use std::io::Read;
                    if entry.read_to_string(&mut script_content).is_ok() {
                        for line in script_content.lines() {
                            let trimmed = line.trim();
                            if trimmed.to_lowercase().starts_with("game.gamename") {
                                if let Some(start) = trimmed.find('"') {
                                    if let Some(end) = trimmed[start + 1..].rfind('"') {
                                        let raw_name = &trimmed[start + 1..start + 1 + end];
                                        if !raw_name.trim().is_empty() {
                                            detected_name = Some(raw_name.trim().to_string());
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Determine sanitized title
        let raw_title = detected_name
            .as_deref()
            .or(fallback_id)
            .unwrap_or("handler");

        // Remove invalid filename characters
        let safe_title: String = raw_title
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ => c,
            })
            .collect();
        let safe_title = safe_title.trim();
        let safe_title = if safe_title.is_empty() { "handler" } else { safe_title };

        let mut extracted = Vec::new();

        // 2. Extract entries
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| NucleusError::Other(format!("Failed to read archive entry: {}", e)))?;

            let enclosed_name = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let name_str = enclosed_name.to_string_lossy().to_string();

            if file.is_dir() {
                let outpath = if safe_title == "handler" {
                    destination_dir.join(&enclosed_name)
                } else {
                    destination_dir.join(safe_title).join(&enclosed_name)
                };
                file_util::ensure_dir(&outpath)?;
            } else {
                let outpath = if name_str.eq_ignore_ascii_case("handler.js") {
                    // Main handler script saved directly as <GameName>.js in handlers dir!
                    destination_dir.join(format!("{}.js", safe_title))
                } else if safe_title != "handler" && !enclosed_name.components().next().map_or(false, |c| c.as_os_str() == safe_title) {
                    // Other auxiliary assets isolated into handlers/<GameName>/
                    destination_dir.join(safe_title).join(&enclosed_name)
                } else {
                    destination_dir.join(&enclosed_name)
                };

                if let Some(parent) = outpath.parent() {
                    file_util::ensure_dir(parent)?;
                }

                // If file already exists, check if modified and create backup before overwrite
                if outpath.exists() {
                    if let Ok(existing_bytes) = std::fs::read(&outpath) {
                        let mut new_bytes = Vec::new();
                        use std::io::Read;
                        if file.read_to_end(&mut new_bytes).is_ok() {
                            if existing_bytes != new_bytes {
                                let bak_path = outpath.with_extension("js.bak");
                                let _ = std::fs::write(&bak_path, &existing_bytes);
                                tracing::info!("Preserved backup of modified local handler: {:?}", bak_path);
                            }
                            let mut outfile = std::fs::File::create(&outpath)?;
                            std::io::copy(&mut std::io::Cursor::new(new_bytes), &mut outfile)?;
                            extracted.push(outpath);
                            continue;
                        }
                    }
                }

                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
                extracted.push(outpath);
            }
        }

        Ok(extracted)
    }

    /// Extracts an in-memory zip archive (downloaded .nc package) into the target directory.
    pub fn extract_package(
        archive_bytes: &[u8],
        destination_dir: &Path,
    ) -> NucleusResult<Vec<PathBuf>> {
        Self::extract_package_isolated(archive_bytes, destination_dir, None)
    }

    /// Persists `.hubmeta.json` sidecar files alongside extracted `.js` handlers and companion folders.
    fn write_hubmeta_sidecars(
        destination_dir: &Path,
        extracted: &[PathBuf],
        sidecar: &HubMetaSidecar,
    ) -> NucleusResult<()> {
        let json = serde_json::to_string_pretty(sidecar)?;
        let mut wrote_js_sidecar = false;

        for path in extracted {
            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .map_or(false, |ext| ext.eq_ignore_ascii_case("js"))
            {
                let sidecar_path = path.with_extension("hubmeta.json");
                std::fs::write(&sidecar_path, &json)?;
                wrote_js_sidecar = true;

                if let (Some(parent), Some(stem)) = (path.parent(), path.file_stem()) {
                    let companion_dir = parent.join(stem);
                    if companion_dir.is_dir() {
                        std::fs::write(companion_dir.join(".hubmeta.json"), &json)?;
                    }
                }
            }
        }

        if !wrote_js_sidecar {
            let fallback_path = destination_dir.join(format!("{}.hubmeta.json", sidecar.handler_id));
            std::fs::write(fallback_path, &json)?;
        }

        Ok(())
    }

    /// Downloads and extracts a specific package version with progress reporting, writing `.hubmeta.json` sidecars.
    pub async fn download_and_extract_package_version_with_progress<F>(
        &self,
        handler_id: &str,
        package_id: &str,
        version: i32,
        expected_size: Option<u64>,
        destination_dir: &Path,
        on_progress: F,
    ) -> NucleusResult<Vec<PathBuf>>
    where
        F: FnMut(f32) + Send + 'static,
    {
        let bytes = self
            .download_package_with_progress_and_size(
                package_id,
                handler_id,
                version,
                expected_size,
                on_progress,
            )
            .await?;
        let extracted = Self::extract_package_isolated(&bytes, destination_dir, Some(handler_id))?;
        let sidecar = HubMetaSidecar {
            handler_id: handler_id.to_string(),
            package_id: package_id.to_string(),
            installed_version: version,
        };
        Self::write_hubmeta_sidecars(destination_dir, &extracted, &sidecar)?;
        Ok(extracted)
    }

    /// Downloads and automatically extracts a handler archive into the specified destination folder.
    pub async fn download_and_extract_handler(
        &self,
        handler_id: &str,
        destination_dir: &Path,
    ) -> NucleusResult<Vec<PathBuf>> {
        self.download_and_extract_handler_with_progress(handler_id, destination_dir, |_| {}).await
    }

    /// Downloads and automatically extracts a handler archive with progress reporting.
    pub async fn download_and_extract_handler_with_progress<F>(
        &self,
        handler_id: &str,
        destination_dir: &Path,
        on_progress: F,
    ) -> NucleusResult<Vec<PathBuf>>
    where
        F: FnMut(f32) + Send + 'static,
    {
        // If handler_id corresponds to a specific package version, download and extract that specific version payload directly.
        if let Ok(pkg) = self.package_details(handler_id).await {
            let pkg_ver = pkg
                .meta
                .as_ref()
                .and_then(|m| crate::types::value_to_i32(&m.handler_version));
            if pkg.id == handler_id && pkg_ver.is_some() {
                let ver = pkg_ver.unwrap();
                let target_handler_id = pkg
                    .meta
                    .as_ref()
                    .and_then(|m| m.handler_id.as_deref())
                    .unwrap_or(handler_id);
                return self
                    .download_and_extract_package_version_with_progress(
                        target_handler_id,
                        &pkg.id,
                        ver,
                        pkg.size,
                        destination_dir,
                        on_progress,
                    )
                    .await;
            }
        }

        // Fall back to resolve_latest_package if package_details fails (i.e. when a handler GUID or game slug is passed).
        let (resolved_handler_id, pkg_id, ver, expected_size) =
            self.resolve_latest_package(handler_id).await?;
        self.download_and_extract_package_version_with_progress(
            &resolved_handler_id,
            &pkg_id,
            ver,
            expected_size,
            destination_dir,
            on_progress,
        )
        .await
    }
}

impl Default for HubClient {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    #[test]
    fn test_hub_client_default_construction() {
        let client = HubClient::default();
        assert_eq!(client.base_url(), "https://hub.splitscreen.me/api/v1");
        assert_eq!(client.cdn_base_url(), "https://hub.splitscreen.me");

        let custom = HubClient::new(Some("http://localhost:8080/api/v1/"));
        assert_eq!(custom.base_url(), "http://localhost:8080/api/v1");
        assert_eq!(custom.cdn_base_url(), "http://localhost:8080");
    }

    #[test]
    fn test_extract_package_in_memory() {
        let mut buffer = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut buffer));
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            zip.start_file("handler.js", options).unwrap();
            zip.write_all(b"Game.ExecutableName = 'game.exe';").unwrap();

            zip.add_directory("subfolder", options).unwrap();
            zip.start_file("subfolder/config.txt", options).unwrap();
            zip.write_all(b"key=value").unwrap();

            zip.finish().unwrap();
        }

        let temp_dir = std::env::temp_dir().join(format!("nucleus_zip_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let extracted = HubClient::extract_package(&buffer, &temp_dir).unwrap();
        assert_eq!(extracted.len(), 2);

        let js_path = temp_dir.join("handler.js");
        assert!(js_path.exists());
        let js_content = file_util::read_text_file(&js_path).unwrap();
        assert_eq!(js_content, "Game.ExecutableName = 'game.exe';");

        let cfg_path = temp_dir.join("subfolder").join("config.txt");
        assert!(cfg_path.exists());
        let cfg_content = file_util::read_text_file(&cfg_path).unwrap();
        assert_eq!(cfg_content, "key=value");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_live_splitscreen_hub_api() {
        let client = HubClient::default();
        let search_res = client.search_raw_handlers("halo").await;
        if let Ok(handlers) = search_res {
            assert!(!handlers.is_empty(), "Expected at least one Halo handler from SplitScreen.Me Hub");
            let first = &handlers[0];
            assert!(!first.id.is_empty());
            assert!(first.display_title().to_lowercase().contains("halo"));

            // Test fetching handler details
            let detail = client.get_handler_details(&first.id).await;
            assert!(detail.is_ok(), "Failed to get handler detail: {:?}", detail.err());
        }
    }

    fn build_test_nc_archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut buffer));
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            for (name, contents) in entries {
                zip.start_file(*name, options).unwrap();
                zip.write_all(contents).unwrap();
            }
            zip.finish().unwrap();
        }
        buffer
    }

    async fn spawn_mock_cdn_server(
        zip_bytes: Vec<u8>,
    ) -> (
        std::net::SocketAddr,
        std::sync::Arc<std::sync::Mutex<Vec<String>>>,
        tokio::task::JoinHandle<()>,
    ) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let requested_paths = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let paths_clone = requested_paths.clone();

        let handle = tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let mut req_buf = Vec::new();
                let mut chunk = [0u8; 1024];
                loop {
                    match socket.read(&mut chunk).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            req_buf.extend_from_slice(&chunk[..n]);
                            if req_buf.windows(4).any(|w| w == b"\r\n\r\n") {
                                break;
                            }
                        }
                    }
                }
                let req_text = String::from_utf8_lossy(&req_buf);
                if let Some(first_line) = req_text.lines().next() {
                    if let Some(path) = first_line.split_whitespace().nth(1) {
                        paths_clone.lock().unwrap().push(path.to_string());
                    }
                }

                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
                    zip_bytes.len()
                );
                let _ = socket.write_all(header.as_bytes()).await;
                let _ = socket.write_all(&zip_bytes).await;
                let _ = socket.flush().await;
                let _ = socket.shutdown().await;
            }
        });

        (addr, requested_paths, handle)
    }

    #[tokio::test]
    async fn test_download_and_extract_package_version_with_progress_writes_sidecar() {
        let zip_bytes = build_test_nc_archive(&[
            (
                "handler.js",
                b"Game.GameName = \"Old Version Game\";\nGame.ExecutableName = \"oldgame.exe\";\n",
            ),
            ("extras/readme.txt", b"old version asset"),
        ]);

        let (addr, requested_paths, server_handle) = spawn_mock_cdn_server(zip_bytes).await;
        let client = HubClient::with_base_url(&format!("http://{}", addr));

        let dest_dir = std::env::temp_dir().join(format!(
            "nucleus_hub_ver_test_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dest_dir);

        let progress_events = std::sync::Arc::new(std::sync::Mutex::new(Vec::<f32>::new()));
        let progress_clone = progress_events.clone();

        let extracted = client
            .download_and_extract_package_version_with_progress(
                "hid_123",
                "pkg_old_v2",
                2,
                None,
                &dest_dir,
                move |p| {
                    progress_clone.lock().unwrap().push(p);
                },
            )
            .await
            .unwrap();

        server_handle.abort();

        let paths = requested_paths.lock().unwrap().clone();
        assert_eq!(
            paths,
            vec!["/cdn/storage/packages/pkg_old_v2/original/handler-hid_123-v2.nc?download=true"]
        );

        let js_path = dest_dir.join("Old Version Game.js");
        assert!(js_path.exists(), "Expected extracted handler JS at {:?}", js_path);
        assert!(extracted.contains(&js_path));

        let sidecar_path = dest_dir.join("Old Version Game.hubmeta.json");
        assert!(
            sidecar_path.exists(),
            "Expected sidecar at {:?}",
            sidecar_path
        );
        let sidecar_raw = std::fs::read_to_string(&sidecar_path).unwrap();
        let sidecar: HubMetaSidecar = serde_json::from_str(&sidecar_raw).unwrap();
        assert_eq!(
            sidecar,
            HubMetaSidecar {
                handler_id: "hid_123".to_string(),
                package_id: "pkg_old_v2".to_string(),
                installed_version: 2,
            }
        );

        let companion_sidecar_path = dest_dir.join("Old Version Game").join(".hubmeta.json");
        assert!(
            companion_sidecar_path.exists(),
            "Expected companion subfolder sidecar at {:?}",
            companion_sidecar_path
        );
        let companion_sidecar: HubMetaSidecar =
            serde_json::from_str(&std::fs::read_to_string(&companion_sidecar_path).unwrap())
                .unwrap();
        assert_eq!(companion_sidecar, sidecar);

        let recorded = progress_events.lock().unwrap();
        assert!(!recorded.is_empty());
        assert_eq!(recorded.last().copied(), Some(1.0));

        let _ = std::fs::remove_dir_all(&dest_dir);
    }

    #[tokio::test]
    async fn test_download_and_extract_package_version_without_game_name_uses_handler_id() {
        let zip_bytes = build_test_nc_archive(&[(
            "handler.js",
            b"Game.ExecutableName = 'legacy.exe';\n",
        )]);

        let (addr, requested_paths, server_handle) = spawn_mock_cdn_server(zip_bytes).await;
        let client = HubClient::with_base_url(&format!("http://{}", addr));

        let dest_dir = std::env::temp_dir().join(format!(
            "nucleus_hub_ver_fallback_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dest_dir);

        let _extracted = client
            .download_and_extract_package_version_with_progress(
                "hid_123",
                "pkg_old_v2",
                2,
                None,
                &dest_dir,
                |_| {},
            )
            .await
            .unwrap();

        server_handle.abort();

        let paths = requested_paths.lock().unwrap().clone();
        assert_eq!(
            paths,
            vec!["/cdn/storage/packages/pkg_old_v2/original/handler-hid_123-v2.nc?download=true"]
        );

        let js_path = dest_dir.join("hid_123.js");
        assert!(js_path.exists());

        let sidecar_path = dest_dir.join("hid_123.hubmeta.json");
        assert!(sidecar_path.exists());
        let sidecar: HubMetaSidecar =
            serde_json::from_str(&std::fs::read_to_string(&sidecar_path).unwrap()).unwrap();
        assert_eq!(sidecar.handler_id, "hid_123");
        assert_eq!(sidecar.package_id, "pkg_old_v2");
        assert_eq!(sidecar.installed_version, 2);

        let _ = std::fs::remove_dir_all(&dest_dir);
    }

    #[tokio::test]
    async fn test_download_and_extract_handler_with_specific_package_id() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let zip_bytes = build_test_nc_archive(&[(
            "handler.js",
            b"Game.GameName = \"Old Historical Version\";\n",
        )]);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let requested_paths = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let paths_clone = requested_paths.clone();

        let server_handle = tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let mut req_buf = Vec::new();
                let mut chunk = [0u8; 1024];
                loop {
                    match socket.read(&mut chunk).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            req_buf.extend_from_slice(&chunk[..n]);
                            if req_buf.windows(4).any(|w| w == b"\r\n\r\n") {
                                break;
                            }
                        }
                    }
                }
                let req_text = String::from_utf8_lossy(&req_buf);
                let path = req_text
                    .lines()
                    .next()
                    .and_then(|l| l.split_whitespace().nth(1))
                    .unwrap_or("")
                    .to_string();

                paths_clone.lock().unwrap().push(path.clone());

                if path.contains("/package/") {
                    let body = serde_json::json!({
                        "package": {
                            "_id": "pkg_historical_v2",
                            "meta": {
                                "handlerId": "hid_legacy",
                                "handlerVersion": 2
                            }
                        }
                    })
                    .to_string();
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(resp.as_bytes()).await;
                } else {
                    let header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
                        zip_bytes.len()
                    );
                    let _ = socket.write_all(header.as_bytes()).await;
                    let _ = socket.write_all(&zip_bytes).await;
                }
                let _ = socket.flush().await;
                let _ = socket.shutdown().await;
            }
        });

        let client = HubClient::with_base_url(&format!("http://{}", addr));
        let dest_dir = std::env::temp_dir().join(format!(
            "nucleus_hub_hist_test_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dest_dir);

        let extracted = client
            .download_and_extract_handler_with_progress("pkg_historical_v2", &dest_dir, |_| {})
            .await
            .unwrap();

        server_handle.abort();

        let paths = requested_paths.lock().unwrap().clone();
        assert_eq!(
            paths,
            vec![
                "/api/v1/package/pkg_historical_v2",
                "/cdn/storage/packages/pkg_historical_v2/original/handler-hid_legacy-v2.nc?download=true"
            ]
        );

        let js_path = dest_dir.join("Old Historical Version.js");
        assert!(js_path.exists());
        assert!(extracted.contains(&js_path));

        let sidecar_path = dest_dir.join("Old Historical Version.hubmeta.json");
        assert!(sidecar_path.exists());
        let sidecar: HubMetaSidecar =
            serde_json::from_str(&std::fs::read_to_string(&sidecar_path).unwrap()).unwrap();
        assert_eq!(sidecar.handler_id, "hid_legacy");
        assert_eq!(sidecar.package_id, "pkg_historical_v2");
        assert_eq!(sidecar.installed_version, 2);

        let _ = std::fs::remove_dir_all(&dest_dir);
    }
}

