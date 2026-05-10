use core::fmt;

use crate::cli::Command;

pub mod apply;
pub mod detect;

pub fn dispatch(cmd: Command) -> Result<(), CommandError> {
    match cmd {
        Command::Apply {
            rom,
            patch,
            out,
            strip_header,
            fix_checksum,
            verify_input,
            verify_output,
        } => apply::run(apply::Args {
            rom_path: &rom,
            patch_path: &patch,
            out,
            strip_header,
            fix_checksum,
            verify_input,
            verify_output,
        }),
        Command::Detect { patch } => detect::run(&patch),
    }
}

#[derive(Debug)]
pub enum CommandError {
    Io(std::io::Error),
    Patch(rompatch_core::PatchError),
    UnknownFormat,
    HashMismatch {
        kind: &'static str,
        expected: String,
        actual: String,
    },
    InvalidHashSpec(String),
}

impl CommandError {
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Io(_) => 3,
            Self::Patch(_)
            | Self::UnknownFormat
            | Self::HashMismatch { .. }
            | Self::InvalidHashSpec(_) => 2,
        }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Patch(e) => write!(f, "{e}"),
            Self::UnknownFormat => f.write_str("unknown patch format (no recognized magic bytes)"),
            Self::HashMismatch {
                kind,
                expected,
                actual,
            } => write!(f, "{kind} hash mismatch: expected {expected}, got {actual}"),
            Self::InvalidHashSpec(s) => write!(f, "invalid hash spec: {s}"),
        }
    }
}

impl std::error::Error for CommandError {}

impl From<std::io::Error> for CommandError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<rompatch_core::PatchError> for CommandError {
    fn from(e: rompatch_core::PatchError) -> Self {
        Self::Patch(e)
    }
}
