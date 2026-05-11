//! High-level orchestration: take a ROM and a patch, optionally strip the
//! ROM header, verify input hashes, apply the patch, fix cartridge
//! checksums, verify output hashes, and re-attach the header.
//!
//! [`run`] is byte-oriented: it does no file I/O. CLI and GUI front-ends
//! read paths, call [`run`], and write the resulting bytes themselves.

use core::fmt;

use crate::checksum_fix::{self, ChecksumFamily};
use crate::error::PatchError;
use crate::format::{self, FormatKind};
use crate::hash;
use crate::header::{self, HeaderKind};

/// One of the four hash algorithms accepted in a [`HashSpec`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HashAlgo {
    Crc32,
    Md5,
    Sha1,
    Adler32,
}

impl HashAlgo {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Crc32 => "crc32",
            Self::Md5 => "md5",
            Self::Sha1 => "sha1",
            Self::Adler32 => "adler32",
        }
    }

    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "crc32" => Some(Self::Crc32),
            "md5" => Some(Self::Md5),
            "sha1" => Some(Self::Sha1),
            "adler32" => Some(Self::Adler32),
            _ => None,
        }
    }

    /// Compute the hash of `bytes` and return its canonical lowercase
    /// hex representation.
    #[must_use]
    pub fn compute_hex(self, bytes: &[u8]) -> String {
        match self {
            Self::Crc32 => format!("{:08x}", hash::crc32(bytes)),
            Self::Md5 => hash::hex(&hash::md5(bytes)),
            Self::Sha1 => hash::hex(&hash::sha1(bytes)),
            Self::Adler32 => format!("{:08x}", hash::adler32(bytes)),
        }
    }
}

/// Expected hash for one side of a patch operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HashSpec {
    pub algo: HashAlgo,
    /// Lowercase hex string. Constructors normalize on the way in.
    pub expected_hex: String,
}

impl HashSpec {
    /// Parse the CLI form `algo:hex` (e.g. `sha1:da39a3ee...`).
    pub fn parse(spec: &str) -> core::result::Result<Self, ApplyError> {
        let (algo, expected) = spec
            .split_once(':')
            .ok_or_else(|| ApplyError::InvalidHashSpec(spec.to_string()))?;
        let algo =
            HashAlgo::parse(algo).ok_or_else(|| ApplyError::InvalidHashSpec(algo.to_string()))?;
        Ok(Self {
            algo,
            expected_hex: expected.trim().to_ascii_lowercase(),
        })
    }
}

/// Which side of the patch operation a [`HashSpec`] was checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HashCheckKind {
    Input,
    Output,
}

impl HashCheckKind {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
        }
    }
}

/// Inputs to [`run`].
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplyOptions {
    pub strip_header: bool,
    pub fix_checksum: bool,
    pub verify_input: Option<HashSpec>,
    pub verify_output: Option<HashSpec>,
    pub format_override: Option<FormatKind>,
}

/// Result of a successful [`run`] call.
#[derive(Debug, Clone)]
pub struct ApplyOutcome {
    pub format: FormatKind,
    /// Final ROM bytes (header re-attached if it was stripped).
    pub output: Vec<u8>,
    /// `Some(kind)` if a header was detected and stripped before patching.
    pub stripped_header: Option<HeaderKind>,
    /// `Some(family)` if [`ApplyOptions::fix_checksum`] was set and a
    /// known cartridge family was detected.
    pub fixed_checksum: Option<ChecksumFamily>,
}

/// Errors returned by [`run`].
#[derive(Debug, Clone)]
pub enum ApplyError {
    Patch(PatchError),
    UnknownFormat,
    UnknownFormatName(String),
    InvalidHashSpec(String),
    HashMismatch {
        kind: HashCheckKind,
        algo: HashAlgo,
        expected: String,
        actual: String,
    },
}

impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Patch(e) => write!(f, "{e}"),
            Self::UnknownFormat => f.write_str("unknown patch format (no recognized magic bytes)"),
            Self::UnknownFormatName(name) => write!(f, "unknown format name: {name}"),
            Self::InvalidHashSpec(s) => write!(f, "invalid hash spec: {s}"),
            Self::HashMismatch {
                kind,
                algo,
                expected,
                actual,
            } => write!(
                f,
                "{} {} hash mismatch: expected {expected}, got {actual}",
                kind.name(),
                algo.name()
            ),
        }
    }
}

impl std::error::Error for ApplyError {}

impl From<PatchError> for ApplyError {
    fn from(e: PatchError) -> Self {
        Self::Patch(e)
    }
}

/// Run the full apply pipeline against the given ROM and patch bytes.
pub fn run(
    rom: &[u8],
    patch: &[u8],
    opts: &ApplyOptions,
) -> core::result::Result<ApplyOutcome, ApplyError> {
    let (stripped_header, body): (Option<HeaderKind>, &[u8]) = if opts.strip_header {
        match header::detect(rom) {
            Some(kind) => {
                let (_h, b) = header::split(rom, kind);
                (Some(kind), b)
            }
            None => (None, rom),
        }
    } else {
        (None, rom)
    };

    if let Some(spec) = &opts.verify_input {
        verify(body, spec, HashCheckKind::Input)?;
    }

    let kind = opts
        .format_override
        .or_else(|| format::detect(patch))
        .ok_or(ApplyError::UnknownFormat)?;

    let mut output = kind.apply(patch, body)?;

    let fixed_checksum = if opts.fix_checksum {
        checksum_fix::sniff_and_fix(&mut output)
    } else {
        None
    };

    if let Some(spec) = &opts.verify_output {
        verify(&output, spec, HashCheckKind::Output)?;
    }

    let final_output = if let Some(kind) = stripped_header {
        let (h, _) = header::split(rom, kind);
        let mut joined = Vec::with_capacity(h.len() + output.len());
        joined.extend_from_slice(h);
        joined.extend_from_slice(&output);
        joined
    } else {
        output
    };

    Ok(ApplyOutcome {
        format: kind,
        output: final_output,
        stripped_header,
        fixed_checksum,
    })
}

fn verify(
    bytes: &[u8],
    spec: &HashSpec,
    kind: HashCheckKind,
) -> core::result::Result<(), ApplyError> {
    let actual = spec.algo.compute_hex(bytes);
    if actual != spec.expected_hex {
        return Err(ApplyError::HashMismatch {
            kind,
            algo: spec.algo,
            expected: spec.expected_hex.clone(),
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ips_identity_patch() -> Vec<u8> {
        // "PATCH" + "EOF" - a valid IPS patch with no records.
        let mut p = Vec::new();
        p.extend_from_slice(b"PATCH");
        p.extend_from_slice(b"EOF");
        p
    }

    #[test]
    fn empty_ips_patch_is_identity() {
        let rom = vec![0xAB; 1024];
        let patch = ips_identity_patch();
        let outcome = run(&rom, &patch, &ApplyOptions::default()).unwrap();
        assert_eq!(outcome.format, FormatKind::Ips);
        assert_eq!(outcome.output, rom);
        assert!(outcome.stripped_header.is_none());
        assert!(outcome.fixed_checksum.is_none());
    }

    #[test]
    fn unknown_magic_returns_unknown_format() {
        let rom = vec![0u8; 16];
        let patch = b"\xDE\xAD\xBE\xEFhello".to_vec();
        let err = run(&rom, &patch, &ApplyOptions::default()).unwrap_err();
        assert!(matches!(err, ApplyError::UnknownFormat));
    }

    #[test]
    fn format_override_bypasses_detection() {
        let rom = vec![0xAB; 1024];
        let patch = ips_identity_patch();
        let opts = ApplyOptions {
            format_override: Some(FormatKind::Ips),
            ..ApplyOptions::default()
        };
        let outcome = run(&rom, &patch, &opts).unwrap();
        assert_eq!(outcome.format, FormatKind::Ips);
    }

    #[test]
    fn verify_input_mismatch_returns_error() {
        let rom = vec![0xAB; 1024];
        let patch = ips_identity_patch();
        let opts = ApplyOptions {
            verify_input: Some(HashSpec {
                algo: HashAlgo::Crc32,
                expected_hex: "deadbeef".into(),
            }),
            ..ApplyOptions::default()
        };
        let err = run(&rom, &patch, &opts).unwrap_err();
        assert!(matches!(
            err,
            ApplyError::HashMismatch {
                kind: HashCheckKind::Input,
                algo: HashAlgo::Crc32,
                ..
            }
        ));
    }

    #[test]
    fn verify_input_match_succeeds() {
        let rom = vec![0xAB; 1024];
        let patch = ips_identity_patch();
        let crc = format!("{:08x}", hash::crc32(&rom));
        let opts = ApplyOptions {
            verify_input: Some(HashSpec {
                algo: HashAlgo::Crc32,
                expected_hex: crc,
            }),
            ..ApplyOptions::default()
        };
        run(&rom, &patch, &opts).unwrap();
    }

    #[test]
    fn hash_spec_parse_roundtrip() {
        let spec = HashSpec::parse("SHA1:DA39A3EE5E6B4B0D3255BFEF95601890AFD80709").unwrap();
        assert_eq!(spec.algo, HashAlgo::Sha1);
        assert_eq!(
            spec.expected_hex,
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
    }

    #[test]
    fn hash_spec_parse_rejects_missing_colon() {
        let err = HashSpec::parse("sha1deadbeef").unwrap_err();
        assert!(matches!(err, ApplyError::InvalidHashSpec(_)));
    }

    #[test]
    fn format_kind_from_name_accepts_aliases() {
        assert_eq!(FormatKind::from_name("ips"), Some(FormatKind::Ips));
        assert_eq!(FormatKind::from_name("BSDIFF"), Some(FormatKind::Bdf));
        assert_eq!(FormatKind::from_name("aps_gba"), Some(FormatKind::ApsGba));
        assert_eq!(FormatKind::from_name("zzz"), None);
    }
}
