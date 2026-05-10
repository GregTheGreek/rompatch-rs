use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use rompatch_core::format;

use super::CommandError;

pub fn run(rom_path: &Path, patch_path: &Path, out: Option<PathBuf>) -> Result<(), CommandError> {
    let rom = fs::read(rom_path)?;
    let patch = fs::read(patch_path)?;

    let kind = format::detect(&patch).ok_or(CommandError::UnknownFormat)?;
    let output = format::apply(&patch, &rom)?;

    let out_path = out.unwrap_or_else(|| default_output_path(rom_path));
    fs::write(&out_path, &output)?;

    eprintln!(
        "applied {} patch ({} bytes) -> {} ({} bytes)",
        kind.name(),
        patch.len(),
        out_path.display(),
        output.len(),
    );
    Ok(())
}

fn default_output_path(rom_path: &Path) -> PathBuf {
    let stem = rom_path
        .file_stem()
        .map_or_else(OsString::new, OsString::from);
    let ext = rom_path.extension();
    let mut name = stem;
    name.push(".patched");
    if let Some(e) = ext {
        name.push(".");
        name.push(e);
    }
    let mut out = rom_path.to_path_buf();
    out.set_file_name(name);
    out
}
