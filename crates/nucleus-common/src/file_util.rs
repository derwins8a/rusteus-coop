//! File system operations and utilities.

use crate::error::{NucleusError, NucleusResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Copies all contents from source directory to destination directory.
pub fn copy_directory(src: &Path, dst: &Path, recursive: bool) -> NucleusResult<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            if recursive {
                copy_directory(&entry.path(), &dest_path, recursive)?;
            }
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }

    Ok(())
}

/// Copies a single file from `src` to `dst`, ensuring destination parent directories exist.
pub fn copy_file(src: &Path, dst: &Path, _overwrite: bool) -> NucleusResult<()> {
    if let Some(parent) = dst.parent() {
        ensure_dir(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}

/// Backs up a file into the designated backup directory.
pub fn backup_file(src: &Path, backup_dir: &Path) -> NucleusResult<PathBuf> {
    ensure_dir(backup_dir)?;
    let file_name = src.file_name().ok_or_else(|| {
        NucleusError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid source file name",
        ))
    })?;
    let dest_path = backup_dir.join(file_name);
    fs::copy(src, &dest_path)?;
    Ok(dest_path)
}

/// Restores a backed up file to its original target location.
pub fn restore_file(backup: &Path, original: &Path) -> NucleusResult<()> {
    if let Some(parent) = original.parent() {
        ensure_dir(parent)?;
    }
    fs::copy(backup, original)?;
    Ok(())
}

/// Creates a directory and any necessary parent directories if it does not already exist.
pub fn ensure_dir(path: &Path) -> NucleusResult<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Reads an entire text file into a string.
pub fn read_text_file(path: &Path) -> NucleusResult<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

/// Writes string content into a file, creating parent directories if needed.
pub fn write_text_file(path: &Path, content: &str) -> NucleusResult<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

/// Finds files within a directory matching the specified pattern.
pub fn find_files(root: &Path, pattern: &str, recursive: bool) -> NucleusResult<Vec<PathBuf>> {
    let mut results = Vec::new();
    if !root.exists() {
        return Ok(results);
    }

    let pattern_lower = pattern.to_lowercase();

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_dir() {
            if recursive {
                let mut sub_files = find_files(&path, pattern, recursive)?;
                results.append(&mut sub_files);
            }
        } else if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.to_lowercase().contains(&pattern_lower) {
                results.push(path);
            }
        }
    }

    Ok(results)
}

/// Finds the 1-based line number in `text` matching `pattern` according to `mode`.
/// Returns -1 if not found.
pub fn find_line_number(text: &str, pattern: &str, mode: crate::types::SearchMode) -> i32 {
    use crate::types::SearchMode;
    for (idx, line) in text.lines().enumerate() {
        let matched = match mode {
            SearchMode::StartsWith => line.starts_with(pattern),
            SearchMode::Contains => line.contains(pattern),
            SearchMode::EndsWith => line.ends_with(pattern),
            SearchMode::Exact => line == pattern,
        };
        if matched {
            return (idx + 1) as i32;
        }
    }
    -1
}

/// Finds the 1-based line number in a file matching `pattern` according to `mode`.
/// Returns -1 if not found.
pub fn find_line_number_in_file(
    path: &Path,
    pattern: &str,
    mode: crate::types::SearchMode,
) -> NucleusResult<i32> {
    let content = read_text_file(path)?;
    Ok(find_line_number(&content, pattern, mode))
}

/// Replaces specific lines in a file by 1-based line number.
pub fn replace_lines_in_file(path: &Path, replacements: &[(usize, String)]) -> NucleusResult<()> {
    let content = read_text_file(path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    for (line_num_1based, new_val) in replacements {
        if *line_num_1based > 0 && *line_num_1based <= lines.len() {
            lines[*line_num_1based - 1] = new_val.clone();
        }
    }

    let updated = lines.join("\r\n");
    write_text_file(path, &updated)?;
    Ok(())
}

/// Removes a line in a file by 1-based line number.
pub fn remove_line_in_file(path: &Path, line_index_1based: usize) -> NucleusResult<()> {
    let content = read_text_file(path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    if line_index_1based > 0 && line_index_1based <= lines.len() {
        lines.remove(line_index_1based - 1);
        let updated = lines.join("\r\n");
        write_text_file(path, &updated)?;
    }
    Ok(())
}

/// Removes all lines in a file matching `pattern` according to `mode`.
pub fn remove_lines_matching_pattern(
    path: &Path,
    pattern: &str,
    mode: crate::types::SearchMode,
) -> NucleusResult<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let content = read_text_file(path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let orig_len = lines.len();
    lines.retain(|line| {
        let matches = match mode {
            crate::types::SearchMode::Contains => line.contains(pattern),
            crate::types::SearchMode::Exact => line == pattern,
            crate::types::SearchMode::StartsWith => line.starts_with(pattern),
            crate::types::SearchMode::EndsWith => line.ends_with(pattern),
        };
        !matches
    });
    let changed = lines.len() != orig_len;
    let updated = lines.join("\r\n");
    write_text_file(path, &updated)?;
    Ok(changed)
}

/// Modifies or appends a key-value pair under a specific section in an INI file.
pub fn modify_ini_value(
    file_path: &Path,
    section: &str,
    key: &str,
    value: &str,
) -> NucleusResult<()> {
    let content = if file_path.exists() {
        read_text_file(file_path)?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let target_section = format!("[{}]", section.trim());
    let mut in_section = false;
    let mut section_found = false;
    let mut key_updated = false;
    let mut section_end_index = lines.len();

    let key_prefix = format!("{}=", key.trim());
    let key_prefix_spaced = format!("{} =", key.trim());

    for (i, line) in lines.iter_mut().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_section {
                // We reached the next section
                section_end_index = i;
                break;
            }
            if trimmed.eq_ignore_ascii_case(&target_section) {
                in_section = true;
                section_found = true;
            }
        } else if in_section {
            if trimmed.starts_with(&key_prefix) || trimmed.starts_with(&key_prefix_spaced) {
                *line = format!("{}={}", key.trim(), value.trim());
                key_updated = true;
                break;
            }
        }
    }

    if !section_found {
        lines.push(target_section);
        lines.push(format!("{}={}", key.trim(), value.trim()));
    } else if !key_updated {
        // Insert under the section
        lines.insert(
            section_end_index,
            format!("{}={}", key.trim(), value.trim()),
        );
    }

    let updated = lines.join("\r\n");
    write_text_file(file_path, &updated)?;
    Ok(())
}

/// Modifies or appends a key-value pair in a Source Engine configuration file (*.cfg).
/// Lines are formatted as `key "value"` or `"key" "value"`.
pub fn modify_cfg_value(
    file_path: &Path,
    section: &str,
    key: &str,
    value: &str,
) -> NucleusResult<()> {
    let content = if file_path.exists() {
        read_text_file(file_path)?
    } else {
        String::new()
    };

    let key_clean = key.trim().trim_matches('"');
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut key_updated = false;

    for line in lines.iter_mut() {
        let trimmed = line.trim();
        // Check if line starts with key or "key" followed by whitespace
        let is_match = if let Some(rest) = trimmed.strip_prefix(key_clean) {
            rest.starts_with(' ') || rest.starts_with('\t')
        } else if let Some(rest) = trimmed.strip_prefix(&format!("\"{}\"", key_clean)) {
            rest.starts_with(' ') || rest.starts_with('\t')
        } else {
            false
        };

        if is_match {
            let leading_ws: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            if trimmed.starts_with('"') {
                *line = format!("{}\"{}\" \"{}\"", leading_ws, key_clean, value);
            } else {
                *line = format!("{}{} \"{}\"", leading_ws, key_clean, value);
            }
            key_updated = true;
            break;
        }
    }

    if !key_updated {
        // If a section is specified and exists, insert inside section
        if !section.is_empty() {
            let section_clean = section.trim().trim_matches('"');
            let mut section_end = None;
            let mut in_section = false;

            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed == format!("\"{}\"", section_clean) || trimmed == section_clean {
                    in_section = true;
                } else if in_section && trimmed == "}" {
                    section_end = Some(i);
                    break;
                }
            }

            if let Some(end) = section_end {
                lines.insert(end, format!("\t\"{}\" \"{}\"", key_clean, value));
            } else {
                lines.push(format!("\"{}\"", section_clean));
                lines.push("{".to_string());
                lines.push(format!("\t\"{}\" \"{}\"", key_clean, value));
                lines.push("}".to_string());
            }
        } else {
            // Flat CFG file (e.g. autoexec.cfg / config.cfg)
            lines.push(format!("{} \"{}\"", key_clean, value));
        }
    }

    let updated = lines.join("\r\n");
    write_text_file(file_path, &updated)?;
    Ok(())
}

/// Expands `%VAR%` environment variables in a path or string.
pub fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();
    while let Some(start) = result.find('%') {
        if let Some(end) = result[start + 1..].find('%') {
            let var_name = &result[start + 1..start + 1 + end];
            let val = std::env::var(var_name).unwrap_or_default();
            result.replace_range(start..start + 1 + end + 1, &val);
        } else {
            break;
        }
    }
    result
}

/// Modifies or inserts an XML attribute value matching the specified XPath/tag name.
pub fn change_xml_attribute_value(
    file_path: &Path,
    xpath_or_tag: &str,
    attr_name: &str,
    attr_value: &str,
) -> NucleusResult<()> {
    let content = read_text_file(file_path)?;
    let tag_name = xpath_or_tag
        .trim_start_matches('/')
        .split('/')
        .last()
        .unwrap_or(xpath_or_tag)
        .trim();

    let open_pattern = format!("<{}", tag_name);
    let mut new_content = String::new();
    let mut last_idx = 0;

    let lower_content = content.to_ascii_lowercase();
    let lower_open = open_pattern.to_ascii_lowercase();

    while let Some(pos) = lower_content[last_idx..].find(&lower_open) {
        let abs_start = last_idx + pos;
        new_content.push_str(&content[last_idx..abs_start]);

        // Find end of tag '>'
        if let Some(tag_end_rel) = content[abs_start..].find('>') {
            let abs_end = abs_start + tag_end_rel;
            let tag_body = &content[abs_start..abs_end];

            // Look for attr_name="..."
            let lower_tag = tag_body.to_ascii_lowercase();
            let attr_pattern = format!("{}=", attr_name.to_ascii_lowercase());

            let updated_tag = if let Some(attr_pos) = lower_tag.find(&attr_pattern) {
                let quote_start = abs_start + attr_pos + attr_pattern.len();
                let quote_char = content.chars().nth(quote_start).unwrap_or('"');
                if let Some(quote_end_rel) = content[quote_start + 1..abs_end].find(quote_char) {
                    let quote_end = quote_start + 1 + quote_end_rel;
                    format!(
                        "{}{}{}{}",
                        &content[abs_start..quote_start + 1],
                        attr_value,
                        quote_char,
                        &content[quote_end + 1..abs_end]
                    )
                } else {
                    tag_body.to_string()
                }
            } else {
                // Insert attribute before closing slash or bracket
                let trimmed = tag_body.trim_end();
                if trimmed.ends_with('/') {
                    format!(
                        "{} {}=\"{}\" /",
                        &trimmed[..trimmed.len() - 1].trim_end(),
                        attr_name,
                        attr_value
                    )
                } else {
                    format!("{} {}=\"{}\"", trimmed, attr_name, attr_value)
                }
            };

            new_content.push_str(&updated_tag);
            last_idx = abs_end;
        } else {
            new_content.push_str(&content[abs_start..]);
            last_idx = content.len();
            break;
        }
    }

    new_content.push_str(&content[last_idx..]);
    write_text_file(file_path, &new_content)?;
    Ok(())
}

/// Modifies an XML element text or node value.
pub fn change_xml_node_value(
    file_path: &Path,
    xpath_or_tag: &str,
    node_value: &str,
) -> NucleusResult<()> {
    let content = read_text_file(file_path)?;
    let tag_name = xpath_or_tag
        .trim_start_matches('/')
        .split('/')
        .last()
        .unwrap_or(xpath_or_tag)
        .trim();

    let open_tag = format!("<{}", tag_name);
    let close_tag = format!("</{}>", tag_name);

    if let (Some(open_pos), Some(close_pos)) = (
        content
            .to_ascii_lowercase()
            .find(&open_tag.to_ascii_lowercase()),
        content
            .to_ascii_lowercase()
            .find(&close_tag.to_ascii_lowercase()),
    ) {
        if let Some(tag_body_end) = content[open_pos..].find('>') {
            let value_start = open_pos + tag_body_end + 1;
            if value_start <= close_pos {
                let mut updated = content[..value_start].to_string();
                updated.push_str(node_value);
                updated.push_str(&content[close_pos..]);
                write_text_file(file_path, &updated)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_write_read() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_common_test_{}", std::process::id()));
        ensure_dir(&temp_dir).unwrap();
        let file_path = temp_dir.join("test.txt");

        write_text_file(&file_path, "Hello Nucleus Coop!").unwrap();
        let content = read_text_file(&file_path).unwrap();
        assert_eq!(content, "Hello Nucleus Coop!");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_backup_and_restore() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_common_backup_{}", std::process::id()));
        let backup_dir = temp_dir.join("backups");
        ensure_dir(&temp_dir).unwrap();

        let orig_file = temp_dir.join("save.dat");
        write_text_file(&orig_file, "SaveSlot1Data").unwrap();

        let backed_up = backup_file(&orig_file, &backup_dir).unwrap();
        assert!(backed_up.exists());

        write_text_file(&orig_file, "CorruptedSaveSlot").unwrap();
        assert_eq!(read_text_file(&orig_file).unwrap(), "CorruptedSaveSlot");

        restore_file(&backed_up, &orig_file).unwrap();
        assert_eq!(read_text_file(&orig_file).unwrap(), "SaveSlot1Data");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_find_files_recursive() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_common_find_{}", std::process::id()));
        let sub_dir = temp_dir.join("subfolder");
        ensure_dir(&sub_dir).unwrap();

        let file1 = temp_dir.join("game.exe");
        let file2 = sub_dir.join("game_launcher.exe");
        let file3 = sub_dir.join("readme.txt");

        write_text_file(&file1, "dummy").unwrap();
        write_text_file(&file2, "dummy").unwrap();
        write_text_file(&file3, "dummy").unwrap();

        let found = find_files(&temp_dir, ".exe", true).unwrap();
        assert_eq!(found.len(), 2);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_find_and_replace_lines() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_common_lines_{}", std::process::id()));
        ensure_dir(&temp_dir).unwrap();
        let cfg_file = temp_dir.join("test.cfg");

        write_text_file(
            &cfg_file,
            "ScreenMode = 0\r\nScreenWidth = 800\r\nScreenHeight = 600",
        )
        .unwrap();

        let line_num = find_line_number_in_file(
            &cfg_file,
            "ScreenWidth =",
            crate::types::SearchMode::StartsWith,
        )
        .unwrap();
        assert_eq!(line_num, 2);

        replace_lines_in_file(&cfg_file, &[(2, "ScreenWidth = 1920".to_string())]).unwrap();
        let content = read_text_file(&cfg_file).unwrap();
        assert!(content.contains("ScreenWidth = 1920"));

        remove_line_in_file(&cfg_file, 1).unwrap();
        let content_after_del = read_text_file(&cfg_file).unwrap();
        assert!(!content_after_del.contains("ScreenMode = 0"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_modify_ini_value() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_common_ini_{}", std::process::id()));
        ensure_dir(&temp_dir).unwrap();
        let ini_file = temp_dir.join("config.ini");

        write_text_file(
            &ini_file,
            "[DISPLAY]\r\nResolution=800x600\r\nFullScreen=ON",
        )
        .unwrap();

        modify_ini_value(&ini_file, "DISPLAY", "Resolution", "1920x1080").unwrap();
        modify_ini_value(&ini_file, "DISPLAY", "FullScreen", "OFF").unwrap();
        modify_ini_value(&ini_file, "AUDIO", "Volume", "100").unwrap();

        let content = read_text_file(&ini_file).unwrap();
        assert!(content.contains("Resolution=1920x1080"));
        assert!(content.contains("FullScreen=OFF"));
        assert!(content.contains("[AUDIO]"));
        assert!(content.contains("Volume=100"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_expand_env_vars() {
        unsafe {
            std::env::set_var("NUCLEUS_TEST_ENV", "SplitScreenRocks");
        }
        let expanded = expand_env_vars("Path/%NUCLEUS_TEST_ENV%/Subdir");
        assert_eq!(expanded, "Path/SplitScreenRocks/Subdir");
    }

    #[test]
    fn test_change_xml_attribute_and_node_value() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_common_xml_{}", std::process::id()));
        ensure_dir(&temp_dir).unwrap();
        let xml_file = temp_dir.join("hardware_settings.xml");

        let initial_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<hardware_settings_config>
    <resolution width="800" height="600" fullscreen="true" />
    <title>Game</title>
</hardware_settings_config>"#;

        write_text_file(&xml_file, initial_xml).unwrap();

        change_xml_attribute_value(&xml_file, "//resolution", "width", "1920").unwrap();
        change_xml_attribute_value(&xml_file, "//resolution", "height", "1080").unwrap();
        change_xml_attribute_value(&xml_file, "//resolution", "fullscreen", "false").unwrap();
        change_xml_node_value(&xml_file, "//title", "Portal 2 Co-op").unwrap();

        let updated = read_text_file(&xml_file).unwrap();
        assert!(updated.contains("width=\"1920\""));
        assert!(updated.contains("height=\"1080\""));
        assert!(updated.contains("fullscreen=\"false\""));
        assert!(updated.contains("<title>Portal 2 Co-op</title>"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
