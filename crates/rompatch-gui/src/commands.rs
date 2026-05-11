//! Tauri IPC commands. Thin wrappers around `rompatch_core` returning
//! JSON-serializable data.
//!
//! Paths come from native open/save dialogs and are trusted. Any new
//! source of external paths (URL handlers, persisted "recents",
//! drag-and-drop) must add validation before reusing these commands.
#![allow(clippy::needless_pass_by_value)]

use std::path::PathBuf;

use rompatch_core::apply::{self, ApplyOptions};
use rompatch_core::info::PatchInfo;
use rompatch_core::{format, header, FormatKind, HashAlgo, HeaderKind};
use serde::Serialize;

use crate::error::GuiResult;

/// Detect a patch's format from its magic bytes.
#[tauri::command]
pub fn detect_patch_format(patch_path: PathBuf) -> GuiResult<Option<FormatKind>> {
    let bytes = std::fs::read(patch_path)?;
    Ok(format::detect(&bytes))
}

/// Read a patch and return its parsed metadata.
#[tauri::command]
pub fn describe_patch(patch_path: PathBuf) -> GuiResult<PatchInfo> {
    let bytes = std::fs::read(patch_path)?;
    Ok(rompatch_core::info::describe(&bytes)?)
}

/// Detect an SMC/iNES/FDS/LYNX header on a ROM, if present.
#[tauri::command]
pub fn detect_rom_header(rom_path: PathBuf) -> GuiResult<Option<HeaderKind>> {
    let bytes = std::fs::read(rom_path)?;
    Ok(header::detect(&bytes))
}

#[derive(Debug, Serialize)]
pub struct HashReport {
    pub crc32: String,
    pub md5: String,
    pub sha1: String,
    pub adler32: String,
    pub file_size: u64,
}

/// Compute CRC32, MD5, SHA1, and Adler32 of a file in one pass.
#[tauri::command]
pub fn compute_hashes(file_path: PathBuf) -> GuiResult<HashReport> {
    let bytes = std::fs::read(file_path)?;
    Ok(HashReport {
        crc32: HashAlgo::Crc32.compute_hex(&bytes),
        md5: HashAlgo::Md5.compute_hex(&bytes),
        sha1: HashAlgo::Sha1.compute_hex(&bytes),
        adler32: HashAlgo::Adler32.compute_hex(&bytes),
        file_size: bytes.len() as u64,
    })
}

#[derive(Debug, Serialize)]
pub struct ApplyReport {
    pub format: FormatKind,
    pub out_path: PathBuf,
    pub out_size: u64,
    pub stripped_header: Option<HeaderKind>,
    pub fixed_checksum: Option<rompatch_core::ChecksumFamily>,
}

/// Read ROM + patch, run the apply pipeline, write the output to disk,
/// and return a structured report.
#[tauri::command]
pub fn apply_patch(
    rom_path: PathBuf,
    patch_path: PathBuf,
    out_path: PathBuf,
    options: ApplyOptions,
) -> GuiResult<ApplyReport> {
    let rom = std::fs::read(&rom_path)?;
    let patch = std::fs::read(&patch_path)?;

    let outcome = apply::run(&rom, &patch, &options)?;
    std::fs::write(&out_path, &outcome.output)?;

    Ok(ApplyReport {
        format: outcome.format,
        out_size: outcome.output.len() as u64,
        out_path,
        stripped_header: outcome.stripped_header,
        fixed_checksum: outcome.fixed_checksum,
    })
}

/// Suggested default output path for the apply UI: `<rom>.patched.<ext>`.
#[tauri::command]
pub fn default_output_path(rom_path: PathBuf) -> PathBuf {
    apply::default_output_path(&rom_path)
}
