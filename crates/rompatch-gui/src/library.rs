//! Local content-addressed ROM/patch library.
//!
//! Stores three buckets keyed by SHA-256:
//!   `<root>/roms/<hash>.bin`
//!   `<root>/patches/<hash>.<ext>`
//!   `<root>/outputs/<hash>.bin`
//!
//! `library.json` at the root holds entries linking the three by hash plus
//! human metadata (original filename, apply options, timestamp). The file
//! is rewritten atomically (tempfile + rename) so a crash mid-write can't
//! corrupt the index.

use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rompatch_core::apply::ApplyOptions;
use rompatch_core::{ChecksumFamily, FormatKind, HeaderKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{GuiError, GuiResult};

pub const LIBRARY_INDEX_VERSION: u32 = 1;
const INDEX_FILENAME: &str = "library.json";
const ROMS_DIR: &str = "roms";
const PATCHES_DIR: &str = "patches";
const OUTPUTS_DIR: &str = "outputs";
const SETTINGS_FILENAME: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub id: String,
    pub source_rom_hash: String,
    pub source_rom_name: String,
    pub source_rom_size: u64,
    pub patch_hash: String,
    pub patch_name: String,
    pub patch_format: FormatKind,
    pub output_hash: String,
    pub output_name: String,
    pub output_size: u64,
    pub header: Option<HeaderKind>,
    pub fixed_checksum: Option<ChecksumFamily>,
    pub applied_at: String,
    pub apply_options: ApplyOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRomEntry {
    pub id: String,
    pub rom_hash: String,
    pub rom_name: String,
    pub rom_size: u64,
    pub header: Option<HeaderKind>,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryIndex {
    pub version: u32,
    pub root: PathBuf,
    #[serde(default)]
    pub roms: Vec<LibraryRomEntry>,
    pub entries: Vec<LibraryEntry>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VerifyStatus {
    Match,
    Mismatch,
    Missing,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RevealTarget {
    Source,
    Patch,
    Output,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Settings {
    library_root: Option<PathBuf>,
}

// ---------- root resolution ----------

/// Read the persisted library root, falling back to `<app_data_dir>/library`
/// if no setting is recorded yet.
pub fn current_root(app_data_dir: &Path, app_config_dir: &Path) -> PathBuf {
    let settings_path = app_config_dir.join(SETTINGS_FILENAME);
    if let Ok(bytes) = fs::read(&settings_path) {
        if let Ok(settings) = serde_json::from_slice::<Settings>(&bytes) {
            if let Some(custom) = settings.library_root {
                return custom;
            }
        }
    }
    app_data_dir.join("library")
}

/// Persist a new library root. Caller is responsible for moving existing
/// content if desired.
pub fn set_root(app_config_dir: &Path, new_root: &Path) -> GuiResult<()> {
    fs::create_dir_all(app_config_dir)?;
    let settings = Settings {
        library_root: Some(new_root.to_path_buf()),
    };
    let bytes = serde_json::to_vec_pretty(&settings)?;
    let settings_path = app_config_dir.join(SETTINGS_FILENAME);
    atomic_write(&settings_path, &bytes)?;
    Ok(())
}

// ---------- index I/O ----------

fn index_path(root: &Path) -> PathBuf {
    root.join(INDEX_FILENAME)
}

pub fn load_index(root: &Path) -> GuiResult<LibraryIndex> {
    let path = index_path(root);
    if !path.exists() {
        return Ok(LibraryIndex {
            version: LIBRARY_INDEX_VERSION,
            root: root.to_path_buf(),
            roms: Vec::new(),
            entries: Vec::new(),
        });
    }
    let bytes = fs::read(&path)?;
    let mut index: LibraryIndex = serde_json::from_slice(&bytes)?;
    // Refresh root in case the user moved the library on disk.
    index.root = root.to_path_buf();
    Ok(index)
}

pub fn save_index(root: &Path, index: &LibraryIndex) -> GuiResult<()> {
    fs::create_dir_all(root)?;
    let bytes = serde_json::to_vec_pretty(index)?;
    atomic_write(&index_path(root), &bytes)?;
    Ok(())
}

fn atomic_write(target: &Path, bytes: &[u8]) -> GuiResult<()> {
    let parent = target
        .parent()
        .ok_or_else(|| GuiError::Library(format!("invalid path: {}", target.display())))?;
    fs::create_dir_all(parent)?;
    let tmp = parent.join(format!(
        ".{}.tmp",
        target
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("write")
    ));
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, target)?;
    Ok(())
}

// ---------- hashing + import ----------

pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(&hasher.finalize()))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex(&hasher.finalize())
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

/// Path to a content-addressed file in the library, given its bucket, hash,
/// and original extension. ROMs and outputs always use `.bin`; patches keep
/// their original extension so the format detector still works on round-trip.
fn content_path(root: &Path, bucket: &str, hash: &str, ext: Option<&str>) -> PathBuf {
    let filename = match ext {
        Some(e) if !e.is_empty() => format!("{hash}.{e}"),
        _ => format!("{hash}.bin"),
    };
    root.join(bucket).join(filename)
}

/// Copy `src` into the library under `<bucket>/<hash>.<ext>` if not already
/// present. Returns the destination path. Stream-copies; safe for large files.
fn import_file(
    root: &Path,
    bucket: &str,
    src: &Path,
    hash: &str,
    ext: Option<&str>,
) -> GuiResult<PathBuf> {
    let dest = content_path(root, bucket, hash, ext);
    if dest.exists() {
        return Ok(dest);
    }
    let parent = dest
        .parent()
        .ok_or_else(|| GuiError::Library(format!("invalid dest: {}", dest.display())))?;
    fs::create_dir_all(parent)?;
    let tmp = parent.join(format!(".{hash}.tmp"));
    fs::copy(src, &tmp)?;
    fs::rename(&tmp, &dest)?;
    Ok(dest)
}

fn extension_of(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|s| s.to_str())
        .map(str::to_ascii_lowercase)
}

fn filename_of(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

fn now_iso() -> String {
    // SystemTime -> seconds since epoch -> rough ISO. We don't pull chrono.
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_unix_seconds_utc(secs)
}

/// Minimal UTC ISO-8601 formatter for `YYYY-MM-DDTHH:MM:SSZ`. Avoids pulling
/// `chrono` for a single helper.
fn format_unix_seconds_utc(secs: u64) -> String {
    // Days since 1970-01-01.
    let days = secs / 86_400;
    let seconds_in_day = secs % 86_400;
    let hh = seconds_in_day / 3600;
    let mm = (seconds_in_day % 3600) / 60;
    let ss = seconds_in_day % 60;

    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

/// Howard Hinnant's date algorithm: days since 1970-01-01 → (year, month, day).
#[allow(clippy::similar_names)]
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// ---------- record / list / verify / reapply ----------

/// Inputs to record a successful apply into the library.
pub struct RecordInput<'a> {
    pub root: &'a Path,
    pub source_path: &'a Path,
    pub patch_path: &'a Path,
    pub output_path: &'a Path,
    pub format: FormatKind,
    pub header: Option<HeaderKind>,
    pub fixed_checksum: Option<ChecksumFamily>,
    pub apply_options: ApplyOptions,
}

pub fn record(input: RecordInput<'_>) -> GuiResult<LibraryEntry> {
    let RecordInput {
        root,
        source_path,
        patch_path,
        output_path,
        format,
        header,
        fixed_checksum,
        apply_options,
    } = input;

    fs::create_dir_all(root)?;

    let source_hash = sha256_file(source_path)?;
    let patch_hash = sha256_file(patch_path)?;
    let output_hash = sha256_file(output_path)?;

    let source_size = fs::metadata(source_path)?.len();
    let patch_size = fs::metadata(patch_path)?.len();
    let _ = patch_size; // not stored, but reading metadata also acts as an existence check
    let output_size = fs::metadata(output_path)?.len();

    let patch_ext = extension_of(patch_path);

    import_file(root, ROMS_DIR, source_path, &source_hash, None)?;
    import_file(
        root,
        PATCHES_DIR,
        patch_path,
        &patch_hash,
        patch_ext.as_deref(),
    )?;
    import_file(root, OUTPUTS_DIR, output_path, &output_hash, None)?;

    let mut index = load_index(root)?;

    // Register the source ROM as a first-class library item if it isn't already.
    if !index.roms.iter().any(|r| r.rom_hash == source_hash) {
        index.roms.push(LibraryRomEntry {
            id: Uuid::new_v4().to_string(),
            rom_hash: source_hash.clone(),
            rom_name: filename_of(source_path),
            rom_size: source_size,
            header,
            added_at: now_iso(),
        });
    }

    // De-dup at the entry level: same source+patch+output triple -> bump timestamp on existing.
    if let Some(existing) = index.entries.iter_mut().find(|e| {
        e.source_rom_hash == source_hash
            && e.patch_hash == patch_hash
            && e.output_hash == output_hash
    }) {
        existing.applied_at = now_iso();
        let cloned = existing.clone();
        save_index(root, &index)?;
        return Ok(cloned);
    }

    let entry = LibraryEntry {
        id: Uuid::new_v4().to_string(),
        source_rom_hash: source_hash,
        source_rom_name: filename_of(source_path),
        source_rom_size: source_size,
        patch_hash,
        patch_name: filename_of(patch_path),
        patch_format: format,
        output_hash,
        output_name: filename_of(output_path),
        output_size,
        header,
        fixed_checksum,
        applied_at: now_iso(),
        apply_options,
    };

    index.entries.push(entry.clone());
    save_index(root, &index)?;
    Ok(entry)
}

pub fn list(root: &Path) -> GuiResult<Vec<LibraryEntry>> {
    let mut index = load_index(root)?;
    index
        .entries
        .sort_by(|a, b| b.applied_at.cmp(&a.applied_at));
    Ok(index.entries)
}

/// Absolute path to the content-addressed copy of a base ROM. Returns an
/// error if the file is missing on disk (the entry was recorded but the file
/// was deleted out-of-band).
pub fn rom_path_for(root: &Path, rom_hash: &str) -> GuiResult<PathBuf> {
    let path = content_path(root, ROMS_DIR, rom_hash, None);
    if !path.exists() {
        return Err(GuiError::Library(format!(
            "rom file missing for hash {rom_hash}"
        )));
    }
    Ok(path)
}

pub fn list_roms(root: &Path) -> GuiResult<Vec<LibraryRomEntry>> {
    let mut index = load_index(root)?;
    index.roms.sort_by(|a, b| b.added_at.cmp(&a.added_at));
    Ok(index.roms)
}

/// Read just enough of `path` to run `format::detect` on the magic bytes.
/// 64 bytes is overkill for every supported magic but keeps the read cheap.
fn read_magic_prefix(path: &Path) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let mut f = fs::File::open(path)?;
    let mut buf = vec![0u8; 64];
    let n = f.read(&mut buf)?;
    buf.truncate(n);
    Ok(buf)
}

/// Import a bare ROM into the library without a patch. Deduplicated by
/// SHA-256 - re-importing the same file is a no-op that returns the existing
/// entry. Refuses files whose magic bytes match a known patch format, since
/// patching a patch is never the intent. Returns the entry (existing or new).
pub fn import_rom(
    root: &Path,
    rom_path: &Path,
    header: Option<HeaderKind>,
) -> GuiResult<LibraryRomEntry> {
    let magic = read_magic_prefix(rom_path)?;
    if let Some(format) = rompatch_core::format::detect(&magic) {
        return Err(GuiError::Library(format!(
            "{} looks like a {} patch, not a ROM. Patches can't be imported as ROMs - apply them to a ROM from the Patch page instead.",
            filename_of(rom_path),
            format.name()
        )));
    }

    fs::create_dir_all(root)?;
    let hash = sha256_file(rom_path)?;
    let size = fs::metadata(rom_path)?.len();
    let name = filename_of(rom_path);

    import_file(root, ROMS_DIR, rom_path, &hash, None)?;

    let mut index = load_index(root)?;
    if let Some(existing) = index.roms.iter().find(|r| r.rom_hash == hash) {
        return Ok(existing.clone());
    }

    let entry = LibraryRomEntry {
        id: Uuid::new_v4().to_string(),
        rom_hash: hash,
        rom_name: name,
        rom_size: size,
        header,
        added_at: now_iso(),
    };
    index.roms.push(entry.clone());
    save_index(root, &index)?;
    Ok(entry)
}

pub fn entry_paths(root: &Path, entry: &LibraryEntry) -> (PathBuf, PathBuf, PathBuf) {
    let patch_ext = extension_of(Path::new(&entry.patch_name));
    (
        content_path(root, ROMS_DIR, &entry.source_rom_hash, None),
        content_path(root, PATCHES_DIR, &entry.patch_hash, patch_ext.as_deref()),
        content_path(root, OUTPUTS_DIR, &entry.output_hash, None),
    )
}

pub fn verify(root: &Path, entry_id: &str) -> GuiResult<VerifyStatus> {
    let index = load_index(root)?;
    let entry = index
        .entries
        .iter()
        .find(|e| e.id == entry_id)
        .ok_or_else(|| GuiError::Library(format!("entry not found: {entry_id}")))?;
    let (_source, _patch, output) = entry_paths(root, entry);
    if !output.exists() {
        return Ok(VerifyStatus::Missing);
    }
    let observed = sha256_file(&output)?;
    Ok(if observed == entry.output_hash {
        VerifyStatus::Match
    } else {
        VerifyStatus::Mismatch
    })
}

pub fn reapply(root: &Path, entry_id: &str) -> GuiResult<VerifyStatus> {
    let index = load_index(root)?;
    let entry = index
        .entries
        .iter()
        .find(|e| e.id == entry_id)
        .ok_or_else(|| GuiError::Library(format!("entry not found: {entry_id}")))?;
    let (source_path, patch_path, output_path) = entry_paths(root, entry);

    let source_bytes = fs::read(&source_path)?;
    let patch_bytes = fs::read(&patch_path)?;
    let outcome = rompatch_core::apply::run(&source_bytes, &patch_bytes, &entry.apply_options)?;

    // Re-write the output deterministically into the library bucket.
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, &outcome.output)?;

    let observed = sha256_bytes(&outcome.output);
    Ok(if observed == entry.output_hash {
        VerifyStatus::Match
    } else {
        VerifyStatus::Mismatch
    })
}

/// Write a playable copy of an entry's output to `dest`. Fast path is a
/// `fs::copy` from the content-addressed library blob; if that blob is
/// missing on disk we regenerate the bytes from the stored source + patch
/// using the original `ApplyOptions`, then write to `dest` and restore the
/// library blob so subsequent exports stay fast.
pub fn export(root: &Path, entry_id: &str, dest: &Path) -> GuiResult<()> {
    let index = load_index(root)?;
    let entry = index
        .entries
        .iter()
        .find(|e| e.id == entry_id)
        .ok_or_else(|| GuiError::Library(format!("entry not found: {entry_id}")))?;

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    let (source_path, patch_path, library_output) = entry_paths(root, entry);

    if library_output.exists() {
        fs::copy(&library_output, dest)?;
        return Ok(());
    }

    // Out-of-band delete of the library blob: regenerate deterministically.
    let source_bytes = fs::read(&source_path)?;
    let patch_bytes = fs::read(&patch_path)?;
    let outcome = rompatch_core::apply::run(&source_bytes, &patch_bytes, &entry.apply_options)?;
    fs::write(dest, &outcome.output)?;
    if let Some(parent) = library_output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&library_output, &outcome.output)?;
    Ok(())
}

pub fn reveal_path(root: &Path, entry_id: &str, target: RevealTarget) -> GuiResult<PathBuf> {
    let index = load_index(root)?;
    let entry = index
        .entries
        .iter()
        .find(|e| e.id == entry_id)
        .ok_or_else(|| GuiError::Library(format!("entry not found: {entry_id}")))?;
    let (source, patch, output) = entry_paths(root, entry);
    Ok(match target {
        RevealTarget::Source => source,
        RevealTarget::Patch => patch,
        RevealTarget::Output => output,
    })
}

/// On macOS, `open -R <path>` highlights the file in Finder. On other
/// platforms, fall back to opening the parent directory.
#[cfg(target_os = "macos")]
pub fn reveal_in_finder(path: &Path) -> GuiResult<()> {
    std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .status()?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn reveal_in_finder(path: &Path) -> GuiResult<()> {
    if let Some(parent) = path.parent() {
        std::process::Command::new("explorer")
            .arg(parent)
            .status()?;
    }
    Ok(())
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
pub fn reveal_in_finder(path: &Path) -> GuiResult<()> {
    if let Some(parent) = path.parent() {
        std::process::Command::new("xdg-open")
            .arg(parent)
            .status()?;
    }
    Ok(())
}

/// Remove a source ROM along with every patch application that references
/// it. Patch and output blobs that were unique to those applications are
/// unlinked; blobs still referenced by surviving entries (e.g. the same
/// patch hash applied against a different ROM) are kept.
pub fn delete_rom(root: &Path, rom_hash: &str) -> GuiResult<()> {
    let mut index = load_index(root)?;

    let rom_pos = index.roms.iter().position(|r| r.rom_hash == rom_hash);
    let cascaded: Vec<LibraryEntry> = index
        .entries
        .iter()
        .filter(|e| e.source_rom_hash == rom_hash)
        .cloned()
        .collect();

    if rom_pos.is_none() && cascaded.is_empty() {
        return Err(GuiError::Library(format!("rom not found: {rom_hash}")));
    }

    if let Some(p) = rom_pos {
        index.roms.remove(p);
    }
    index.entries.retain(|e| e.source_rom_hash != rom_hash);

    let rom_path = content_path(root, ROMS_DIR, rom_hash, None);
    if rom_path.exists() {
        fs::remove_file(&rom_path)?;
    }

    // For each cascaded entry, drop its patch/output blob only when nothing
    // in the remaining index references the same hash. `path.exists()`
    // tolerates two cascaded entries sharing a blob.
    for removed in &cascaded {
        let patch_still_referenced = index
            .entries
            .iter()
            .any(|e| e.patch_hash == removed.patch_hash);
        if !patch_still_referenced {
            let patch_ext = extension_of(Path::new(&removed.patch_name));
            let patch_path =
                content_path(root, PATCHES_DIR, &removed.patch_hash, patch_ext.as_deref());
            if patch_path.exists() {
                fs::remove_file(&patch_path)?;
            }
        }
        let output_still_referenced = index
            .entries
            .iter()
            .any(|e| e.output_hash == removed.output_hash);
        if !output_still_referenced {
            let output_path = content_path(root, OUTPUTS_DIR, &removed.output_hash, None);
            if output_path.exists() {
                fs::remove_file(&output_path)?;
            }
        }
    }

    save_index(root, &index)?;
    Ok(())
}

/// Remove a patch application from the index. The source ROM is preserved
/// (other entries may reference it, and the user can delete it separately
/// via the ROM list). The patch and output blobs are unlinked only when no
/// other remaining entry references the same hash, so shared content is
/// kept intact.
pub fn delete_entry(root: &Path, entry_id: &str) -> GuiResult<()> {
    let mut index = load_index(root)?;
    let pos = index
        .entries
        .iter()
        .position(|e| e.id == entry_id)
        .ok_or_else(|| GuiError::Library(format!("entry not found: {entry_id}")))?;
    let removed = index.entries.remove(pos);

    let patch_still_referenced = index
        .entries
        .iter()
        .any(|e| e.patch_hash == removed.patch_hash);
    if !patch_still_referenced {
        let patch_ext = extension_of(Path::new(&removed.patch_name));
        let patch_path = content_path(root, PATCHES_DIR, &removed.patch_hash, patch_ext.as_deref());
        if patch_path.exists() {
            fs::remove_file(&patch_path)?;
        }
    }

    let output_still_referenced = index
        .entries
        .iter()
        .any(|e| e.output_hash == removed.output_hash);
    if !output_still_referenced {
        let output_path = content_path(root, OUTPUTS_DIR, &removed.output_hash, None);
        if output_path.exists() {
            fs::remove_file(&output_path)?;
        }
    }

    save_index(root, &index)?;
    Ok(())
}

pub fn lookup_by_patch_hash(root: &Path, patch_path: &Path) -> GuiResult<Vec<LibraryEntry>> {
    let hash = sha256_file(patch_path)?;
    let mut matches: Vec<LibraryEntry> = load_index(root)?
        .entries
        .into_iter()
        .filter(|e| e.patch_hash == hash)
        .collect();
    matches.sort_by(|a, b| b.applied_at.cmp(&a.applied_at));
    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_tmp(dir: &Path, name: &str, contents: &[u8]) -> PathBuf {
        let p = dir.join(name);
        let mut f = fs::File::create(&p).unwrap();
        f.write_all(contents).unwrap();
        p
    }

    #[test]
    fn sha256_file_matches_sha256_bytes() {
        let dir = TempDir::new().unwrap();
        let p = write_tmp(dir.path(), "x.bin", b"hello world");
        let a = sha256_file(&p).unwrap();
        let b = sha256_bytes(b"hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn import_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("lib");
        let src = write_tmp(dir.path(), "rom.bin", b"AAAA");
        let h = sha256_file(&src).unwrap();
        let p1 = import_file(&root, ROMS_DIR, &src, &h, None).unwrap();
        let p2 = import_file(&root, ROMS_DIR, &src, &h, None).unwrap();
        assert_eq!(p1, p2);
        assert!(p1.exists());
    }

    #[test]
    fn atomic_write_replaces_existing() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("library.json");
        atomic_write(&target, b"first").unwrap();
        atomic_write(&target, b"second").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"second");
    }

    #[test]
    fn delete_entry_removes_orphan_blobs_but_keeps_shared() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("lib");

        // Prime the library with two entries sharing a source ROM but with
        // distinct patches/outputs, plus a third entry that shares the second
        // entry's patch hash so we can prove the shared blob is preserved.
        let source = write_tmp(dir.path(), "rom.bin", b"SOURCE");
        let patch_a = write_tmp(dir.path(), "a.ips", b"PATCH-A");
        let patch_b = write_tmp(dir.path(), "b.ips", b"PATCH-B");
        let out_a = write_tmp(dir.path(), "out-a.bin", b"OUT-A");
        let out_b = write_tmp(dir.path(), "out-b.bin", b"OUT-B");

        let opts = ApplyOptions::default();
        let entry_a = record(RecordInput {
            root: &root,
            source_path: &source,
            patch_path: &patch_a,
            output_path: &out_a,
            format: FormatKind::Ips,
            header: None,
            fixed_checksum: None,
            apply_options: opts.clone(),
        })
        .unwrap();
        let _entry_b = record(RecordInput {
            root: &root,
            source_path: &source,
            patch_path: &patch_b,
            output_path: &out_b,
            format: FormatKind::Ips,
            header: None,
            fixed_checksum: None,
            apply_options: opts.clone(),
        })
        .unwrap();

        // Hand-craft a third entry that reuses patch_b's hash but a distinct
        // output, so deleting entry_b leaves the patch blob in place via the
        // share-detection path.
        let mut index = load_index(&root).unwrap();
        let shared_patch_hash = index
            .entries
            .iter()
            .find(|e| e.patch_name == "b.ips")
            .unwrap()
            .patch_hash
            .clone();
        let extra_output_hash = sha256_bytes(b"OUT-C");
        let extra_output_path = content_path(&root, OUTPUTS_DIR, &extra_output_hash, None);
        fs::create_dir_all(extra_output_path.parent().unwrap()).unwrap();
        fs::write(&extra_output_path, b"OUT-C").unwrap();
        index.entries.push(LibraryEntry {
            id: Uuid::new_v4().to_string(),
            source_rom_hash: index.entries[0].source_rom_hash.clone(),
            source_rom_name: "rom.bin".into(),
            source_rom_size: 6,
            patch_hash: shared_patch_hash.clone(),
            patch_name: "b.ips".into(),
            patch_format: FormatKind::Ips,
            output_hash: extra_output_hash.clone(),
            output_name: "out-c.bin".into(),
            output_size: 5,
            header: None,
            fixed_checksum: None,
            applied_at: now_iso(),
            apply_options: opts,
        });
        save_index(&root, &index).unwrap();

        // Snapshot pre-delete state for entry_a.
        let (source_a, patch_a_dest, output_a_dest) = entry_paths(&root, &entry_a);
        assert!(patch_a_dest.exists());
        assert!(output_a_dest.exists());
        let shared_patch_path = content_path(&root, PATCHES_DIR, &shared_patch_hash, Some("ips"));
        assert!(shared_patch_path.exists());

        // Deleting entry_a removes its orphan patch + output, but the source
        // ROM stays (still referenced) and the shared patch blob is untouched.
        delete_entry(&root, &entry_a.id).unwrap();

        let post = load_index(&root).unwrap();
        assert!(post.entries.iter().all(|e| e.id != entry_a.id));
        assert!(!patch_a_dest.exists(), "orphan patch should be unlinked");
        assert!(!output_a_dest.exists(), "orphan output should be unlinked");
        assert!(source_a.exists(), "source ROM must be preserved");
        assert!(shared_patch_path.exists(), "shared patch must be preserved");
    }

    #[test]
    fn delete_entry_unknown_id_errors() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("lib");
        fs::create_dir_all(&root).unwrap();
        save_index(
            &root,
            &LibraryIndex {
                version: LIBRARY_INDEX_VERSION,
                root: root.clone(),
                roms: Vec::new(),
                entries: Vec::new(),
            },
        )
        .unwrap();
        let err = delete_entry(&root, "does-not-exist").unwrap_err();
        assert!(matches!(err, GuiError::Library(_)));
    }

    fn make_entry_for_export(dir: &Path, root: &Path) -> LibraryEntry {
        // Hand-built IPS patch: replace 4 bytes at offset 8 of a 16-byte ROM.
        // Real format so the regenerate-via-apply path is exercised end-to-end.
        let source_bytes = vec![0u8; 16];
        let mut output_bytes = source_bytes.clone();
        output_bytes[8..12].copy_from_slice(b"\xDE\xAD\xBE\xEF");

        let mut patch_bytes = b"PATCH".to_vec();
        // record: offset=0x000008, size=0x0004, data=DEADBEEF
        patch_bytes.extend_from_slice(&[0x00, 0x00, 0x08, 0x00, 0x04]);
        patch_bytes.extend_from_slice(b"\xDE\xAD\xBE\xEF");
        patch_bytes.extend_from_slice(b"EOF");

        let source = write_tmp(dir, "rom.bin", &source_bytes);
        let output = write_tmp(dir, "out.bin", &output_bytes);
        let patch = write_tmp(dir, "p.ips", &patch_bytes);

        record(RecordInput {
            root,
            source_path: &source,
            patch_path: &patch,
            output_path: &output,
            format: FormatKind::Ips,
            header: None,
            fixed_checksum: None,
            apply_options: ApplyOptions::default(),
        })
        .unwrap()
    }

    #[test]
    fn export_uses_library_copy_when_present() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("lib");
        let entry = make_entry_for_export(dir.path(), &root);

        let dest = dir.path().join("exported.bin");
        export(&root, &entry.id, &dest).unwrap();

        let exported = fs::read(&dest).unwrap();
        assert_eq!(sha256_bytes(&exported), entry.output_hash);
    }

    #[test]
    fn export_regenerates_when_library_blob_missing() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("lib");
        let entry = make_entry_for_export(dir.path(), &root);

        // Simulate an out-of-band delete of the library's output blob.
        let (_src, _patch, library_output) = entry_paths(&root, &entry);
        fs::remove_file(&library_output).unwrap();
        assert!(!library_output.exists());

        let dest = dir.path().join("exported.bin");
        export(&root, &entry.id, &dest).unwrap();

        let exported = fs::read(&dest).unwrap();
        assert_eq!(sha256_bytes(&exported), entry.output_hash);
        // Library blob should be restored for future fast-path exports.
        assert!(library_output.exists());
        assert_eq!(sha256_file(&library_output).unwrap(), entry.output_hash);
    }

    #[test]
    fn export_unknown_id_errors() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("lib");
        fs::create_dir_all(&root).unwrap();
        save_index(
            &root,
            &LibraryIndex {
                version: LIBRARY_INDEX_VERSION,
                root: root.clone(),
                roms: Vec::new(),
                entries: Vec::new(),
            },
        )
        .unwrap();
        let err = export(&root, "missing", &dir.path().join("x.bin")).unwrap_err();
        assert!(matches!(err, GuiError::Library(_)));
    }

    #[test]
    fn import_rom_rejects_files_with_patch_magic() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("lib");

        // Real IPS magic header followed by an EOF marker.
        let mut ips = b"PATCH".to_vec();
        ips.extend_from_slice(b"EOF");
        let p = write_tmp(dir.path(), "hack.ips", &ips);

        let err = import_rom(&root, &p, None).unwrap_err();
        match err {
            GuiError::Library(msg) => {
                assert!(msg.contains("IPS"), "error should name the format: {msg}");
                assert!(msg.contains("patch"), "error should call it a patch: {msg}");
            }
            _ => panic!("expected GuiError::Library, got {err:?}"),
        }
        // Reject means we never created the index.
        assert!(!index_path(&root).exists());
    }

    #[test]
    fn import_rom_accepts_files_without_patch_magic() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("lib");
        let p = write_tmp(dir.path(), "rom.bin", b"\x00\x01\x02\x03not a patch");
        let entry = import_rom(&root, &p, None).unwrap();
        assert_eq!(entry.rom_name, "rom.bin");
    }

    #[test]
    fn delete_rom_cascades_entries_and_preserves_shared_blobs() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("lib");

        // ROM A gets a patch. ROM B gets a different patch whose blob
        // happens to share a hash with ROM A's patch (we synthesize this
        // by writing the same bytes), so deleting ROM A must NOT remove
        // the shared patch blob.
        let rom_a = write_tmp(dir.path(), "a.bin", b"ROM-A");
        let rom_b = write_tmp(dir.path(), "b.bin", b"ROM-B");
        let shared_patch = write_tmp(dir.path(), "shared.ips", b"PATCH-SHARED");
        let shared_patch_dup = write_tmp(dir.path(), "shared-dup.ips", b"PATCH-SHARED");
        let out_a = write_tmp(dir.path(), "out-a.bin", b"OUT-A");
        let out_b = write_tmp(dir.path(), "out-b.bin", b"OUT-B");

        let opts = ApplyOptions::default();
        let entry_a = record(RecordInput {
            root: &root,
            source_path: &rom_a,
            patch_path: &shared_patch,
            output_path: &out_a,
            format: FormatKind::Ips,
            header: None,
            fixed_checksum: None,
            apply_options: opts.clone(),
        })
        .unwrap();
        let entry_b = record(RecordInput {
            root: &root,
            source_path: &rom_b,
            patch_path: &shared_patch_dup,
            output_path: &out_b,
            format: FormatKind::Ips,
            header: None,
            fixed_checksum: None,
            apply_options: opts,
        })
        .unwrap();
        assert_eq!(entry_a.patch_hash, entry_b.patch_hash);

        let (source_a_path, _, output_a_path) = entry_paths(&root, &entry_a);
        let shared_patch_path = content_path(&root, PATCHES_DIR, &entry_a.patch_hash, Some("ips"));
        let source_b_path = content_path(&root, ROMS_DIR, &entry_b.source_rom_hash, None);

        delete_rom(&root, &entry_a.source_rom_hash).unwrap();

        let post = load_index(&root).unwrap();
        assert!(post
            .roms
            .iter()
            .all(|r| r.rom_hash != entry_a.source_rom_hash));
        assert!(post.entries.iter().all(|e| e.id != entry_a.id));
        assert!(
            post.entries.iter().any(|e| e.id == entry_b.id),
            "unrelated ROM B's entry must survive"
        );
        assert!(!source_a_path.exists(), "deleted ROM file should be gone");
        assert!(
            !output_a_path.exists(),
            "cascaded orphan output should be gone"
        );
        assert!(
            shared_patch_path.exists(),
            "patch blob shared with ROM B must remain"
        );
        assert!(source_b_path.exists(), "ROM B file must be untouched");
    }

    #[test]
    fn delete_rom_bare_entry_removes_record_and_file() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("lib");
        let rom = write_tmp(dir.path(), "rom.bin", b"BARE");
        let imported = import_rom(&root, &rom, None).unwrap();
        let rom_path = content_path(&root, ROMS_DIR, &imported.rom_hash, None);
        assert!(rom_path.exists());

        delete_rom(&root, &imported.rom_hash).unwrap();

        let post = load_index(&root).unwrap();
        assert!(post.roms.is_empty());
        assert!(!rom_path.exists());
    }

    #[test]
    fn delete_rom_unknown_hash_errors() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("lib");
        fs::create_dir_all(&root).unwrap();
        save_index(
            &root,
            &LibraryIndex {
                version: LIBRARY_INDEX_VERSION,
                root: root.clone(),
                roms: Vec::new(),
                entries: Vec::new(),
            },
        )
        .unwrap();
        let err = delete_rom(&root, "deadbeef").unwrap_err();
        assert!(matches!(err, GuiError::Library(_)));
    }

    #[test]
    fn iso_format_known_epoch() {
        assert_eq!(format_unix_seconds_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_unix_seconds_utc(86_400), "1970-01-02T00:00:00Z");
        // 2024-01-01 00:00:00 UTC
        assert_eq!(
            format_unix_seconds_utc(1_704_067_200),
            "2024-01-01T00:00:00Z"
        );
        // 2024-02-29 (leap day) 12:34:56 UTC
        assert_eq!(
            format_unix_seconds_utc(1_709_210_096),
            "2024-02-29T12:34:56Z"
        );
    }
}
