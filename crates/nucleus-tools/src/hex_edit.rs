//! Hex editing utilities for binary patching game executables and libraries.

use std::fs;
use std::path::{Path, PathBuf};
use nucleus_common::error::{NucleusError, NucleusResult};

/// Tool providing byte pattern search and replacement in game executables and dynamic libraries
/// to modify hardcoded window sizes, mutex names, networking ports, or memory addresses.
#[derive(Debug, Default, Clone)]
pub struct HexEditTool;

impl HexEditTool {
    /// Reads `file_path`, writes `new_bytes` at the given `offset`, and saves it back.
    pub fn edit_at_offset(file_path: &Path, offset: usize, new_bytes: &[u8]) -> NucleusResult<()> {
        let mut data = fs::read(file_path)?;
        if offset + new_bytes.len() > data.len() {
            return Err(NucleusError::Other(format!(
                "Patch offset {} + len {} exceeds file size {}",
                offset,
                new_bytes.len(),
                data.len()
            )));
        }

        data[offset..offset + new_bytes.len()].copy_from_slice(new_bytes);
        fs::write(file_path, data)?;
        Ok(())
    }

    /// Finds all occurrences of `pattern` in `file_path` and replaces them with `replacement`.
    /// `replacement` must be the same length as `pattern`.
    pub fn replace_pattern(
        file_path: &Path,
        pattern: &[u8],
        replacement: &[u8],
    ) -> NucleusResult<usize> {
        if pattern.is_empty() {
            return Err(NucleusError::Other("Pattern cannot be empty".to_string()));
        }
        if pattern.len() != replacement.len() {
            return Err(NucleusError::Other(
                "Replacement must be equal in length to the pattern for in-place patch".to_string(),
            ));
        }

        let mut data = fs::read(file_path)?;
        let mut count = 0;
        let mut i = 0;

        while i + pattern.len() <= data.len() {
            if &data[i..i + pattern.len()] == pattern {
                data[i..i + replacement.len()].copy_from_slice(replacement);
                count += 1;
                i += pattern.len();
            } else {
                i += 1;
            }
        }

        if count > 0 {
            fs::write(file_path, data)?;
        }

        Ok(count)
    }

    /// Helper to convert a hex string (e.g., "48 89 5C 24" or "48895C24") into bytes.
    pub fn hex_to_bytes(hex_str: &str) -> NucleusResult<Vec<u8>> {
        let cleaned: String = hex_str.chars().filter(|c| !c.is_whitespace()).collect();
        if cleaned.len() % 2 != 0 {
            return Err(NucleusError::Other("Hex string has odd length".to_string()));
        }

        let mut bytes = Vec::with_capacity(cleaned.len() / 2);
        for i in (0..cleaned.len()).step_by(2) {
            let byte_str = &cleaned[i..i + 2];
            let byte = u8::from_str_radix(byte_str, 16).map_err(|e| {
                NucleusError::Other(format!("Invalid hex byte '{}': {}", byte_str, e))
            })?;
            bytes.push(byte);
        }

        Ok(bytes)
    }

    /// Copies `src` to `dst` (if paths differ), then finds `find_bytes` and replaces with `replace_bytes`.
    /// If `find_all` is true, replaces all matching occurrences; otherwise only the first occurrence.
    pub fn patch_file(
        src: &Path,
        dst: &Path,
        find_bytes: &[u8],
        replace_bytes: &[u8],
        find_all: bool,
    ) -> NucleusResult<usize> {
        if find_bytes.is_empty() {
            return Err(NucleusError::Other("Search pattern cannot be empty".to_string()));
        }
        if find_bytes.len() != replace_bytes.len() {
            return Err(NucleusError::Other("Search and replace patterns must be equal length".to_string()));
        }

        let mut data = fs::read(src)?;
        let mut count = 0;
        let mut i = 0;

        while i + find_bytes.len() <= data.len() {
            if &data[i..i + find_bytes.len()] == find_bytes {
                data[i..i + replace_bytes.len()].copy_from_slice(replace_bytes);
                count += 1;
                i += find_bytes.len();
                if !find_all {
                    break;
                }
            } else {
                i += 1;
            }
        }

        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(dst, data)?;
        Ok(count)
    }

    /// Reads `src`, parses `orig_pattern` and `patch_pattern` from hex strings,
    /// patches `orig_pattern` with `patch_pattern`, and writes to `dst`.
    pub fn patch_file_find_pattern(
        src: &Path,
        dst: &Path,
        orig_pattern: &str,
        patch_pattern: &str,
        patch_all: bool,
    ) -> NucleusResult<usize> {
        let find_bytes = Self::hex_to_bytes(orig_pattern)?;
        let replace_bytes = Self::hex_to_bytes(patch_pattern)?;
        Self::patch_file(src, dst, &find_bytes, &replace_bytes, patch_all)
    }

    /// Searches for `orig_pattern` in `src`, replaces bytes at 1-based `offset` with
    /// 16-bit little-endian integer `insert`, and writes result to `dst`.
    pub fn patch_file_find_pattern_insert(
        src: &Path,
        dst: &Path,
        orig_pattern: &str,
        insert: i32,
        offset: usize,
        patch_all: bool,
    ) -> NucleusResult<usize> {
        let orig_bytes = Self::hex_to_bytes(orig_pattern)?;
        let mut patch_bytes = orig_bytes.clone();
        let insert_bytes = (insert as u16).to_le_bytes();
        let start_idx = if offset > 0 { offset - 1 } else { 0 };
        for (i, b) in insert_bytes.iter().enumerate() {
            if start_idx + i < patch_bytes.len() {
                patch_bytes[start_idx + i] = *b;
            }
        }
        Self::patch_file(src, dst, &orig_bytes, &patch_bytes, patch_all)
    }

    /// Executes batch hex address patching on the target executable.
    /// Supports both:
    /// - `"<instance_1_based>|<hex_address>|<hex_bytes>"` (applied only when instance index matches `i + 1`)
    /// - `"<hex_address>|<hex_bytes>"` (applied to all instances)
    ///
    /// Preserves a `<stem>_NUCLEUS_BACKUP.<ext>` copy alongside the executable if not already present.
    pub fn hex_edit_exe_address(
        instance_index: usize,
        exe_path: &Path,
        hex_split_lines: &[String],
    ) -> NucleusResult<Vec<PathBuf>> {
        if !exe_path.exists() {
            return Err(NucleusError::Other(format!(
                "Executable not found for hex edit: {}",
                exe_path.display()
            )));
        }

        let mut backups_created = Vec::new();

        if let Some(parent) = exe_path.parent() {
            let stem = exe_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let ext = exe_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let backup_name = if ext.is_empty() {
                format!("{}_NUCLEUS_BACKUP", stem)
            } else {
                format!("{}_NUCLEUS_BACKUP.{}", stem, ext)
            };
            let backup_path = parent.join(backup_name);
            if !backup_path.exists() {
                let _ = fs::copy(exe_path, &backup_path);
                backups_created.push(backup_path);
            }
        }

        for hex_line in hex_split_lines {
            let parts: Vec<&str> = hex_line.split('|').map(|s| s.trim()).collect();
            let (addr_str, bytes_str) = match parts.len() {
                3 => {
                    if let Ok(target_inst) = parts[0].parse::<usize>() {
                        if target_inst != instance_index + 1 {
                            continue;
                        }
                    } else {
                        continue;
                    }
                    (parts[1], parts[2])
                }
                2 => (parts[0], parts[1]),
                _ => continue,
            };

            let clean_addr = addr_str.trim_start_matches("0x").trim_start_matches("0X");
            let offset = usize::from_str_radix(clean_addr, 16).map_err(|e| {
                NucleusError::Other(format!("Invalid hex address '{}': {}", addr_str, e))
            })?;
            let patch_bytes = Self::hex_to_bytes(bytes_str)?;

            Self::edit_at_offset(exe_path, offset, &patch_bytes)?;
        }

        Ok(backups_created)
    }

    /// Executes batch hex address patching on arbitrary files within `link_folder`.
    /// Supports both:
    /// - `"<instance_1_based>|<rel_file_path>|<hex_address>|<hex_bytes>"` (applied only when instance index matches `i + 1`)
    /// - `"<rel_file_path>|<hex_address>|<hex_bytes>"` (applied to all instances)
    ///
    /// Preserves a `<stem>_NUCLEUS_BACKUP.<ext>` copy alongside the file if not already present.
    pub fn hex_edit_file_address(
        instance_index: usize,
        link_folder: &Path,
        hex_split_lines: &[String],
    ) -> NucleusResult<Vec<PathBuf>> {
        let mut backups_created = Vec::new();

        for hex_line in hex_split_lines {
            let parts: Vec<&str> = hex_line.split('|').map(|s| s.trim()).collect();
            let (rel_path_str, addr_str, bytes_str) = match parts.len() {
                4 => {
                    if let Ok(target_inst) = parts[0].parse::<usize>() {
                        if target_inst != instance_index + 1 {
                            continue;
                        }
                    } else {
                        continue;
                    }
                    (parts[1], parts[2], parts[3])
                }
                3 => (parts[0], parts[1], parts[2]),
                _ => continue,
            };

            let full_file_path = link_folder.join(rel_path_str);
            if !full_file_path.exists() {
                continue;
            }

            if let Some(parent) = full_file_path.parent() {
                let stem = full_file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let ext = full_file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let backup_name = if ext.is_empty() {
                    format!("{}_NUCLEUS_BACKUP", stem)
                } else {
                    format!("{}_NUCLEUS_BACKUP.{}", stem, ext)
                };
                let backup_path = parent.join(backup_name);
                if !backup_path.exists() {
                    let _ = fs::copy(&full_file_path, &backup_path);
                    backups_created.push(backup_path);
                }
            }

            let clean_addr = addr_str.trim_start_matches("0x").trim_start_matches("0X");
            let offset = usize::from_str_radix(clean_addr, 16).map_err(|e| {
                NucleusError::Other(format!("Invalid hex address '{}': {}", addr_str, e))
            })?;
            let patch_bytes = Self::hex_to_bytes(bytes_str)?;

            Self::edit_at_offset(&full_file_path, offset, &patch_bytes)?;
        }

        Ok(backups_created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_hex_to_bytes() {
        let bytes = HexEditTool::hex_to_bytes("48 89 5C 24 08").unwrap();
        assert_eq!(bytes, vec![0x48, 0x89, 0x5C, 0x24, 0x08]);

        let bytes_no_spaces = HexEditTool::hex_to_bytes("DEADBEEF").unwrap();
        assert_eq!(bytes_no_bytes_hex(bytes_no_spaces), [0xDE, 0xAD, 0xBE, 0xEF]);
    }

    fn bytes_no_bytes_hex(b: Vec<u8>) -> [u8; 4] {
        [b[0], b[1], b[2], b[3]]
    }

    #[test]
    fn test_replace_pattern() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_hex_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test_bin.bin");

        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(b"Hello WORLD! Hello Earth!").unwrap();
        drop(f);

        let count = HexEditTool::replace_pattern(&file_path, b"Hello", b"Howdy").unwrap();
        assert_eq!(count, 2);

        let patched = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(patched, "Howdy WORLD! Howdy Earth!");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_edit_at_offset() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_offset_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test_offset.bin");

        std::fs::write(&file_path, b"AAAA").unwrap();
        HexEditTool::edit_at_offset(&file_path, 1, b"BC").unwrap();

        let patched = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(patched, "ABCA");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_patch_file_find_pattern() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_patch_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let src_file = temp_dir.join("src.bin");
        let dst_file = temp_dir.join("dst.bin");

        std::fs::write(&src_file, &[0x76, 0x1D, 0xC7, 0x05, 0xAA, 0xBB]).unwrap();
        let patched_count = HexEditTool::patch_file_find_pattern(
            &src_file,
            &dst_file,
            "76 1D C7 05",
            "EB 1D C7 05",
            false,
        ).unwrap();
        assert_eq!(patched_count, 1);

        let dst_bytes = std::fs::read(&dst_file).unwrap();
        assert_eq!(dst_bytes, vec![0xEB, 0x1D, 0xC7, 0x05, 0xAA, 0xBB]);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_patch_file_find_pattern_insert() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_insert_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let src_file = temp_dir.join("src.bin");
        let dst_file = temp_dir.join("dst.bin");

        // Orig pattern: 0x10, 0x20, 0x00, 0x00
        std::fs::write(&src_file, &[0xAA, 0x10, 0x20, 0x00, 0x00, 0xBB]).unwrap();
        // Insert 0x1234 (little endian 0x34, 0x12) at 1-based offset 3 (index 2)
        let count = HexEditTool::patch_file_find_pattern_insert(
            &src_file,
            &dst_file,
            "10 20 00 00",
            0x1234,
            3,
            false,
        ).unwrap();
        assert_eq!(count, 1);

        let dst_bytes = std::fs::read(&dst_file).unwrap();
        assert_eq!(dst_bytes, vec![0xAA, 0x10, 0x20, 0x34, 0x12, 0xBB]);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_hex_edit_exe_address_and_backup() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_exe_addr_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let exe_path = temp_dir.join("game.exe");
        std::fs::write(&exe_path, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();

        let lines = vec![
            "1|0x02|AABB".to_string(), // Player 0 (instance_index 0 + 1 == 1)
            "2|0x02|CCDD".to_string(), // Player 1 (skipped for player 0)
            "0x00|FF".to_string(),     // Applies to all
        ];

        let backups = HexEditTool::hex_edit_exe_address(0, &exe_path, &lines).unwrap();
        assert_eq!(backups.len(), 1);
        let backup_path = temp_dir.join("game_NUCLEUS_BACKUP.exe");
        assert!(backup_path.exists());
        assert_eq!(std::fs::read(&backup_path).unwrap(), vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        let patched_bytes = std::fs::read(&exe_path).unwrap();
        assert_eq!(patched_bytes, vec![0xFF, 0x00, 0xAA, 0xBB, 0x00, 0x00]);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_hex_edit_file_address() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_file_addr_{}", std::process::id()));
        let sub = temp_dir.join("data");
        std::fs::create_dir_all(&sub).unwrap();
        let target_file = sub.join("config.bin");
        std::fs::write(&target_file, &[0x11, 0x22, 0x33, 0x44]).unwrap();

        let lines = vec![
            "data/config.bin|0x01|FFFF".to_string(),
        ];

        let backups = HexEditTool::hex_edit_file_address(0, &temp_dir, &lines).unwrap();
        assert_eq!(backups.len(), 1);
        assert!(sub.join("config_NUCLEUS_BACKUP.bin").exists());

        let patched = std::fs::read(&target_file).unwrap();
        assert_eq!(patched, vec![0x11, 0xFF, 0xFF, 0x44]);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
