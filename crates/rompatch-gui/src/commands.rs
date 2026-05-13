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
use tauri::{AppHandle, Manager};

use crate::error::{GuiError, GuiResult};
use crate::library::{
    self, LibraryEntry, LibraryRomEntry, RecordInput, RevealTarget, VerifyStatus,
};

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

// ---------- library commands ----------

fn resolve_dirs(app: &AppHandle) -> GuiResult<(PathBuf, PathBuf)> {
    let data_dir = app.path().app_data_dir()?;
    let config_dir = app.path().app_config_dir()?;
    Ok((data_dir, config_dir))
}

fn resolve_root(app: &AppHandle) -> GuiResult<PathBuf> {
    let (data_dir, config_dir) = resolve_dirs(app)?;
    Ok(library::current_root(&data_dir, &config_dir))
}

#[tauri::command]
pub fn library_root(app: AppHandle) -> GuiResult<PathBuf> {
    resolve_root(&app)
}

#[tauri::command]
pub fn library_set_root(app: AppHandle, new_root: PathBuf) -> GuiResult<PathBuf> {
    let (_, config_dir) = resolve_dirs(&app)?;
    library::set_root(&config_dir, &new_root)?;
    Ok(new_root)
}

#[tauri::command]
pub fn library_list(app: AppHandle) -> GuiResult<Vec<LibraryEntry>> {
    let root = resolve_root(&app)?;
    library::list(&root)
}

#[tauri::command]
pub fn library_list_roms(app: AppHandle) -> GuiResult<Vec<LibraryRomEntry>> {
    let root = resolve_root(&app)?;
    library::list_roms(&root)
}

#[tauri::command]
pub fn library_import_rom(app: AppHandle, rom_path: PathBuf) -> GuiResult<LibraryRomEntry> {
    let root = resolve_root(&app)?;
    let bytes = std::fs::read(&rom_path)?;
    let header = rompatch_core::header::detect(&bytes);
    library::import_rom(&root, &rom_path, header)
}

#[tauri::command]
pub fn library_rom_path(app: AppHandle, rom_hash: String) -> GuiResult<PathBuf> {
    let root = resolve_root(&app)?;
    library::rom_path_for(&root, &rom_hash)
}

#[derive(Debug, serde::Deserialize)]
pub struct LibraryRecordArgs {
    pub source_path: PathBuf,
    pub patch_path: PathBuf,
    pub output_path: PathBuf,
    pub format: FormatKind,
    pub header: Option<HeaderKind>,
    pub fixed_checksum: Option<rompatch_core::ChecksumFamily>,
    pub apply_options: ApplyOptions,
}

#[tauri::command]
pub fn library_record(app: AppHandle, args: LibraryRecordArgs) -> GuiResult<LibraryEntry> {
    let root = resolve_root(&app)?;
    library::record(RecordInput {
        root: &root,
        source_path: &args.source_path,
        patch_path: &args.patch_path,
        output_path: &args.output_path,
        format: args.format,
        header: args.header,
        fixed_checksum: args.fixed_checksum,
        apply_options: args.apply_options,
    })
}

#[tauri::command]
pub fn library_verify(app: AppHandle, entry_id: String) -> GuiResult<VerifyStatus> {
    let root = resolve_root(&app)?;
    library::verify(&root, &entry_id)
}

#[tauri::command]
pub fn library_reapply(app: AppHandle, entry_id: String) -> GuiResult<VerifyStatus> {
    let root = resolve_root(&app)?;
    library::reapply(&root, &entry_id)
}

#[tauri::command]
pub fn library_reveal(app: AppHandle, entry_id: String, target: RevealTarget) -> GuiResult<()> {
    let root = resolve_root(&app)?;
    let path = library::reveal_path(&root, &entry_id, target)?;
    if !path.exists() {
        return Err(GuiError::Library(format!(
            "file no longer exists: {}",
            path.display()
        )));
    }
    library::reveal_in_finder(&path)
}

#[tauri::command]
pub fn library_delete_entry(app: AppHandle, entry_id: String) -> GuiResult<()> {
    let root = resolve_root(&app)?;
    library::delete_entry(&root, &entry_id)
}

#[tauri::command]
pub fn library_delete_rom(app: AppHandle, rom_hash: String) -> GuiResult<()> {
    let root = resolve_root(&app)?;
    library::delete_rom(&root, &rom_hash)
}

#[tauri::command]
pub fn library_export(app: AppHandle, entry_id: String, dest_path: PathBuf) -> GuiResult<()> {
    let root = resolve_root(&app)?;
    library::export(&root, &entry_id, &dest_path)
}

#[tauri::command]
pub fn library_lookup_by_patch_hash(
    app: AppHandle,
    patch_path: PathBuf,
) -> GuiResult<Vec<LibraryEntry>> {
    let root = resolve_root(&app)?;
    library::lookup_by_patch_hash(&root, &patch_path)
}
