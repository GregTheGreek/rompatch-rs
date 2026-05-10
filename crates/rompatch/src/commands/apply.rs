use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use rompatch_core::{checksum_fix, format, hash, header};

use super::CommandError;

pub struct Args<'a> {
    pub rom_path: &'a Path,
    pub patch_path: &'a Path,
    pub out: Option<PathBuf>,
    pub strip_header: bool,
    pub fix_checksum: bool,
    pub verify_input: Option<String>,
    pub verify_output: Option<String>,
}

pub fn run(args: Args<'_>) -> Result<(), CommandError> {
    let rom = fs::read(args.rom_path)?;
    let patch = fs::read(args.patch_path)?;

    let (preserved_header, body) = if args.strip_header {
        match header::detect(&rom) {
            Some(kind) => {
                let (h, b) = header::split(&rom, kind);
                eprintln!("stripped {} header ({} bytes)", kind.name(), h.len());
                (Some(h.to_vec()), b.to_vec())
            }
            None => (None, rom),
        }
    } else {
        (None, rom)
    };

    if let Some(spec) = &args.verify_input {
        verify_hash(&body, spec, "input")?;
    }

    let kind = format::detect(&patch).ok_or(CommandError::UnknownFormat)?;
    let mut output = format::apply(&patch, &body)?;

    if args.fix_checksum {
        if let Some(name) = checksum_fix_for(&mut output) {
            eprintln!("fixed {name} checksum");
        }
    }

    if let Some(spec) = &args.verify_output {
        verify_hash(&output, spec, "output")?;
    }

    let final_bytes = if let Some(h) = preserved_header {
        let mut joined = h;
        joined.extend_from_slice(&output);
        joined
    } else {
        output
    };

    let out_path = args
        .out
        .unwrap_or_else(|| default_output_path(args.rom_path));
    fs::write(&out_path, &final_bytes)?;

    eprintln!(
        "applied {} patch ({} bytes) -> {} ({} bytes)",
        kind.name(),
        patch.len(),
        out_path.display(),
        final_bytes.len(),
    );
    Ok(())
}

fn checksum_fix_for(rom: &mut [u8]) -> Option<&'static str> {
    if rom.len() >= 0x150 && rom.len() >= 0x134 + GB_LOGO.len() && rom[0x104..0x134] == GB_LOGO {
        checksum_fix::fix_game_boy(rom);
        return Some("Game Boy");
    }
    if rom.len() >= 0x200
        && (&rom[0x100..0x110] == b"SEGA MEGA DRIVE " || &rom[0x100..0x110] == b"SEGA GENESIS    ")
    {
        checksum_fix::fix_mega_drive(rom);
        return Some("Mega Drive");
    }
    None
}

const GB_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

fn verify_hash(bytes: &[u8], spec: &str, kind: &'static str) -> Result<(), CommandError> {
    let (algo, expected) = spec
        .split_once(':')
        .ok_or_else(|| CommandError::InvalidHashSpec(spec.to_string()))?;
    let expected_norm = expected.trim().to_ascii_lowercase();
    let actual = match algo.to_ascii_lowercase().as_str() {
        "crc32" => format!("{:08x}", hash::crc32(bytes)),
        "md5" => {
            use core::fmt::Write;
            let digest = hash::md5(bytes);
            let mut s = String::with_capacity(32);
            for b in digest {
                let _ = write!(&mut s, "{b:02x}");
            }
            s
        }
        other => return Err(CommandError::InvalidHashSpec(other.to_string())),
    };
    if actual != expected_norm {
        return Err(CommandError::HashMismatch {
            kind,
            expected: expected_norm,
            actual,
        });
    }
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
