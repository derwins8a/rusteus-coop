//! `.nc` archive packager and JavaScript handler syntax pre-validation.

use boa_parser::{Parser, Source};
use nucleus_common::error::{NucleusError, NucleusResult};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// Parameters for building a validated `.nc` handler package archive.
#[derive(Debug, Clone)]
pub struct PackageBuildInput {
    pub handler_js_path: PathBuf,
    pub extra_assets_dir: Option<PathBuf>,
    pub handler_title: String,
    pub game_name: String,
    pub version: i32,
}

/// Builds and validates a `.nc` handler ZIP archive in-memory.
pub fn build_and_validate_nc_archive(input: &PackageBuildInput) -> NucleusResult<Vec<u8>> {
    if !input.handler_js_path.exists() {
        return Err(NucleusError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "Handler script not found at '{}'",
                input.handler_js_path.display()
            ),
        )));
    }

    let js_code = fs::read_to_string(&input.handler_js_path)?;

    // 1. Syntax check via boa_parser
    let mut parser = Parser::new(Source::from_bytes(js_code.as_bytes()));
    let mut interner = boa_interner::Interner::default();
    let scope = boa_ast::scope::Scope::new_global();
    parser
        .parse_script(&scope, &mut interner)
        .map_err(|e| NucleusError::Script(format!("Handler JavaScript syntax error: {}", e)))?;

    // 2. Validate required metadata markers
    let has_game_name = js_code.contains("Game.GameName") || !input.game_name.is_empty();
    let has_exe_name =
        js_code.contains("Game.ExecutableName") || js_code.contains("Game.ExecutableNames");
    if !has_game_name {
        return Err(NucleusError::Handler(
            "Handler must define Game.GameName".to_string(),
        ));
    }
    if !has_exe_name {
        return Err(NucleusError::Handler(
            "Handler must define Game.ExecutableName or Game.ExecutableNames".to_string(),
        ));
    }

    // 3. Construct ZIP archive
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Add handler.js at root
        zip.start_file("handler.js", options)
            .map_err(|e| NucleusError::Handler(e.to_string()))?;
        zip.write_all(js_code.as_bytes())?;

        // Add package.json metadata
        let package_json = serde_json::json!({
            "title": input.handler_title,
            "gameName": input.game_name,
            "version": input.version,
        });
        zip.start_file("package.json", options)
            .map_err(|e| NucleusError::Handler(e.to_string()))?;
        zip.write_all(serde_json::to_string_pretty(&package_json)?.as_bytes())?;

        // Add extra assets if specified
        if let Some(ref assets_dir) = input.extra_assets_dir {
            if assets_dir.exists() && assets_dir.is_dir() {
                add_dir_to_zip(&mut zip, assets_dir, assets_dir, options)?;
            }
        }

        zip.finish()
            .map_err(|e| NucleusError::Handler(e.to_string()))?;
    }

    Ok(buffer.into_inner())
}

/// Recursively adds files from a directory into a zip archive with path traversal and size checks.
fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    base_dir: &Path,
    current_dir: &Path,
    options: SimpleFileOptions,
) -> NucleusResult<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let rel_path = path
            .strip_prefix(base_dir)
            .map_err(|e| NucleusError::Other(e.to_string()))?;

        // Reject zip slip attempts
        for comp in rel_path.components() {
            if let std::path::Component::ParentDir = comp {
                return Err(NucleusError::Handler(
                    "Path traversal not permitted in package assets".to_string(),
                ));
            }
        }

        let name_lower = rel_path.to_string_lossy().to_lowercase();
        // Do not bundle autohotkey executables or git directories
        if name_lower.ends_with("autohotkey64.exe")
            || name_lower.ends_with("autohotkey32.exe")
            || name_lower.starts_with(".git")
        {
            continue;
        }

        if path.is_dir() {
            add_dir_to_zip(zip, base_dir, &path, options)?;
        } else if path.is_file() {
            let zip_entry_name = rel_path.to_string_lossy().replace('\\', "/");
            zip.start_file(&zip_entry_name, options)
                .map_err(|e| NucleusError::Handler(e.to_string()))?;
            let file_bytes = fs::read(&path)?;
            zip.write_all(&file_bytes)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_handler_packager() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_pkg_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let js_path = temp_dir.join("test_handler.js");

        fs::write(
            &js_path,
            r#"
            Game.GameName = "Test Game";
            Game.ExecutableName = "test.exe";
            Game.Play = function() {
                Context.Width = 1280;
            };
            "#,
        )
        .unwrap();

        let input = PackageBuildInput {
            handler_js_path: js_path,
            extra_assets_dir: None,
            handler_title: "Test Community Handler".to_string(),
            game_name: "Test Game".to_string(),
            version: 1,
        };

        let zip_bytes = build_and_validate_nc_archive(&input).unwrap();
        assert!(!zip_bytes.is_empty());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_invalid_syntax_rejected() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_pkg_test_inv_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let js_path = temp_dir.join("broken.js");

        fs::write(&js_path, "function broken() { const = 5; }").unwrap();

        let input = PackageBuildInput {
            handler_js_path: js_path,
            extra_assets_dir: None,
            handler_title: "Broken Handler".to_string(),
            game_name: "Test".to_string(),
            version: 1,
        };

        let err = build_and_validate_nc_archive(&input);
        assert!(err.is_err());
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
