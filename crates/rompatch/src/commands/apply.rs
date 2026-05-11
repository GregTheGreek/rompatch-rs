use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use rompatch_core::apply::{self, ApplyError, ApplyOptions, HashSpec};
use rompatch_core::FormatKind;

use super::CommandError;

pub struct Args<'a> {
    pub rom_path: &'a Path,
    pub patch_path: &'a Path,
    pub out: Option<PathBuf>,
    pub strip_header: bool,
    pub fix_checksum: bool,
    pub verify_input: Option<String>,
    pub verify_output: Option<String>,
    pub format_override: Option<String>,
}

pub fn run(args: Args<'_>) -> Result<(), CommandError> {
    let rom = fs::read(args.rom_path)?;
    let patch = fs::read(args.patch_path)?;

    let format_override = match args.format_override {
        Some(name) => {
            Some(FormatKind::from_name(&name).ok_or(ApplyError::UnknownFormatName(name))?)
        }
        None => None,
    };

    let opts = ApplyOptions {
        strip_header: args.strip_header,
        fix_checksum: args.fix_checksum,
        verify_input: args
            .verify_input
            .as_deref()
            .map(HashSpec::parse)
            .transpose()?,
        verify_output: args
            .verify_output
            .as_deref()
            .map(HashSpec::parse)
            .transpose()?,
        format_override,
    };

    let outcome = apply::run(&rom, &patch, &opts)?;

    if let Some(kind) = outcome.stripped_header {
        eprintln!("stripped {} header", kind.name());
    }
    if let Some(family) = outcome.fixed_checksum {
        eprintln!("fixed {} checksum", family.name());
    }

    let out_path = args
        .out
        .unwrap_or_else(|| default_output_path(args.rom_path));
    fs::write(&out_path, &outcome.output)?;

    eprintln!(
        "applied {} patch ({} bytes) -> {} ({} bytes)",
        outcome.format.name(),
        patch.len(),
        out_path.display(),
        outcome.output.len(),
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
