//! Package inspection, validation, and safe extraction for `.nucplugin` and `.nucaddon` archives.

use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use crate::manifest::PluginManifest;
use nucleus_common::error::{NucleusError, NucleusResult};

/// Result of inspecting a plugin package archive without extracting it.
#[derive(Debug, Clone)]
pub struct PackageInspection {
    /// Calculated SHA-256 hex digest of the package archive.
    pub sha256: String,
    /// Parsed manifest found inside the root or first-level directory.
    pub manifest: PluginManifest,
    /// List of file entries inside the package.
    pub entries: Vec<String>,
}

/// Unpacks and validates a plugin or addon zip package safely to a target directory.
pub struct PluginPackager;

impl PluginPackager {
    /// Calculates the SHA-256 hash of an archive file.
    pub fn compute_archive_hash(archive_path: &Path) -> NucleusResult<String> {
        let mut file = File::open(archive_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 16384];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(hex::encode(hasher.finalize()))
    }

    /// Inspects a `.nucplugin`, `.nucaddon`, or `.zip` archive without extracting all files.
    /// Finds and parses the manifest (`plugin.toml` or `addon.toml`).
    pub fn inspect_package(archive_path: &Path) -> NucleusResult<PackageInspection> {
        let sha256 = Self::compute_archive_hash(archive_path)?;
        let file = File::open(archive_path)?;
        let mut zip = ZipArchive::new(file)
            .map_err(|e| NucleusError::Plugin(format!("Invalid zip archive: {e}")))?;

        let mut manifest_content: Option<String> = None;
        let mut entries = Vec::new();

        for i in 0..zip.len() {
            let mut entry = zip
                .by_index(i)
                .map_err(|e| NucleusError::Plugin(format!("Failed reading zip entry: {e}")))?;
            let raw_name = entry.name().to_string();
            entries.push(raw_name.clone());

            let clean_name = raw_name.replace('\\', "/");
            let file_name = clean_name.split('/').last().unwrap_or("");
            if (file_name == "plugin.toml" || file_name == "addon.toml")
                && manifest_content.is_none()
            {
                let mut buf = String::new();
                entry.read_to_string(&mut buf)?;
                manifest_content = Some(buf);
            }
        }

        let raw_manifest = manifest_content.ok_or_else(|| {
            NucleusError::Plugin(
                "Package does not contain a valid 'plugin.toml' or 'addon.toml' manifest"
                    .to_string(),
            )
        })?;

        let manifest = PluginManifest::parse_toml(&raw_manifest)?;

        Ok(PackageInspection {
            sha256,
            manifest,
            entries,
        })
    }

    /// Safely extracts an archive into `destination_dir` with strict zip-slip protection.
    pub fn extract_package(
        archive_path: &Path,
        destination_dir: &Path,
    ) -> NucleusResult<PackageInspection> {
        let inspection = Self::inspect_package(archive_path)?;
        fs::create_dir_all(destination_dir)?;
        let canonical_target = destination_dir
            .canonicalize()
            .map_err(|e| NucleusError::Io(e))?;

        let file = File::open(archive_path)?;
        let mut zip = ZipArchive::new(file)
            .map_err(|e| NucleusError::Plugin(format!("Failed to open zip archive: {e}")))?;

        for i in 0..zip.len() {
            let mut entry = zip
                .by_index(i)
                .map_err(|e| NucleusError::Plugin(format!("Failed reading zip index {i}: {e}")))?;

            let raw_name = entry.name().replace('\\', "/");
            let relative_path = PathBuf::from(&raw_name);

            // Zip-slip security verification: reject absolute paths or parent directory traversals
            if relative_path.is_absolute() {
                return Err(NucleusError::Plugin(format!(
                    "Malicious package detected: absolute path inside zip: {raw_name}"
                )));
            }

            for component in relative_path.components() {
                if let std::path::Component::ParentDir = component {
                    return Err(NucleusError::Plugin(format!(
                        "Zip-slip attack detected: path traversal '..' in {raw_name}"
                    )));
                }
            }

            let outpath = destination_dir.join(&relative_path);

            if entry.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut entry, &mut outfile)?;
            }

            // Verify final written path is inside destination_dir
            if let Ok(canon_out) = outpath.canonicalize() {
                if !canon_out.starts_with(&canonical_target) {
                    let _ = fs::remove_file(&canon_out);
                    return Err(NucleusError::Plugin(format!(
                        "Zip-slip attack detected: extracted path escaped root directory: {}",
                        canon_out.display()
                    )));
                }
            }
        }

        Ok(inspection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::{SimpleFileOptions, ZipWriter};

    fn create_test_zip(manifest_str: &str, files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut buf);
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

            writer.start_file("plugin.toml", options).unwrap();
            writer.write_all(manifest_str.as_bytes()).unwrap();

            for (path, content) in files {
                writer.start_file(*path, options).unwrap();
                writer.write_all(content).unwrap();
            }
            writer.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn test_inspect_and_extract_valid_package() {
        let manifest = r#"
id = "test-packager"
name = "Test Packager"
js_namespace = "testPackager"
version = "1.0.0"
kind = "js-addon"
entrypoint = "main.js"
"#;
        let test_zip = create_test_zip(manifest, &[("main.js", b"console.log('hi');")]);

        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_pkg_test_{}", std::process::id()));
        let zip_path = temp_dir.join("test.nucplugin");
        let extract_dir = temp_dir.join("extracted");

        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(&zip_path, test_zip).unwrap();

        let inspection = PluginPackager::inspect_package(&zip_path).unwrap();
        assert_eq!(inspection.manifest.id, "test-packager");
        assert!(!inspection.sha256.is_empty());

        let extracted = PluginPackager::extract_package(&zip_path, &extract_dir).unwrap();
        assert_eq!(extracted.manifest.id, "test-packager");
        assert!(extract_dir.join("plugin.toml").exists());
        assert!(extract_dir.join("main.js").exists());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_zip_slip_rejection() {
        let manifest = r#"
id = "evil-packager"
name = "Evil"
js_namespace = "evilPackager"
version = "1.0.0"
kind = "js-addon"
entrypoint = "evil.js"
"#;
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut buf);
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

            writer.start_file("plugin.toml", options).unwrap();
            writer.write_all(manifest.as_bytes()).unwrap();

            writer.start_file("../evil.txt", options).unwrap();
            writer.write_all(b"escaped!").unwrap();

            writer.finish().unwrap();
        }
        let malicious_zip = buf.into_inner();

        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_pkg_slip_{}", std::process::id()));
        let zip_path = temp_dir.join("evil.nucplugin");
        let extract_dir = temp_dir.join("extracted");

        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(&zip_path, malicious_zip).unwrap();

        let result = PluginPackager::extract_package(&zip_path, &extract_dir);
        assert!(result.is_err(), "Extraction must reject zip-slip path");

        let _ = fs::remove_dir_all(temp_dir);
    }
}
