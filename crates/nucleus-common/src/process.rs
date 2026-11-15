//! Process and binary inspection utility functions.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::error::{NucleusError, NucleusResult};

/// Machine architecture constants from PE specifications.
const IMAGE_FILE_MACHINE_I386: u16 = 0x014c;
const IMAGE_FILE_MACHINE_IA64: u16 = 0x0200;
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_FILE_MACHINE_ARM64: u16 = 0xaa64;

/// Inspects an executable's binary header (PE format) to determine if it is a 64-bit binary.
pub fn is_64bit_executable(exe_path: &Path) -> NucleusResult<bool> {
    let mut file = File::open(exe_path)?;

    // Check DOS "MZ" magic
    let mut dos_header = [0u8; 64];
    file.read_exact(&mut dos_header)?;
    if dos_header[0] != b'M' || dos_header[1] != b'Z' {
        return Err(NucleusError::Process(format!(
            "Not a valid PE executable (invalid DOS header): {}",
            exe_path.display()
        )));
    }

    // Offset 0x3C contains the 32-bit file pointer to the PE header
    let pe_offset = u32::from_le_bytes([
        dos_header[0x3C],
        dos_header[0x3D],
        dos_header[0x3E],
        dos_header[0x3F],
    ]) as u64;

    file.seek(SeekFrom::Start(pe_offset))?;

    // Verify "PE\0\0" signature
    let mut pe_signature = [0u8; 4];
    file.read_exact(&mut pe_signature)?;
    if &pe_signature != b"PE\0\0" {
        return Err(NucleusError::Process(format!(
            "Missing PE signature in executable: {}",
            exe_path.display()
        )));
    }

    // Read COFF File Header: first 2 bytes is Machine type
    let mut machine_bytes = [0u8; 2];
    file.read_exact(&mut machine_bytes)?;
    let machine = u16::from_le_bytes(machine_bytes);

    match machine {
        IMAGE_FILE_MACHINE_AMD64 | IMAGE_FILE_MACHINE_IA64 | IMAGE_FILE_MACHINE_ARM64 => Ok(true),
        IMAGE_FILE_MACHINE_I386 => Ok(false),
        other => Err(NucleusError::Process(format!(
            "Unknown or unsupported PE machine architecture: 0x{:04X}",
            other
        ))),
    }
}

/// Detects and returns the target architecture of an executable as a string ("x86" or "x64").
pub fn get_exe_architecture(exe_path: &Path) -> NucleusResult<String> {
    let is_64 = is_64bit_executable(exe_path)?;
    if is_64 {
        Ok("x64".to_string())
    } else {
        Ok("x86".to_string())
    }
}
