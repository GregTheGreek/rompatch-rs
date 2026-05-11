use core::fmt;

use crate::cli::Command;

pub mod apply;
pub mod detect;
pub mod hash;
pub mod info;

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
            format_override,
        } => apply::run(apply::Args {
            rom_path: &rom,
            patch_path: &patch,
            out,
            strip_header,
            fix_checksum,
            verify_input,
            verify_output,
            format_override,
        }),
        Command::Detect { patch } => detect::run(&patch),
        Command::Info { patch } => info::run(&patch),
        Command::Hash { file, algo } => hash::run(&file, &algo),
    }
}

#[derive(Debug)]
pub enum CommandError {
    Io(std::io::Error),
    Apply(rompatch_core::ApplyError),
    Patch(rompatch_core::PatchError),
    UnknownFormat,
    InvalidHashAlgo(String),
}

impl CommandError {
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Io(_) => 3,
            Self::Apply(_) | Self::Patch(_) | Self::UnknownFormat | Self::InvalidHashAlgo(_) => 2,
        }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Apply(e) => write!(f, "{e}"),
            Self::Patch(e) => write!(f, "{e}"),
            Self::UnknownFormat => f.write_str("unknown patch format (no recognized magic bytes)"),
            Self::InvalidHashAlgo(s) => write!(f, "invalid hash algorithm: {s}"),
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

impl From<rompatch_core::ApplyError> for CommandError {
    fn from(e: rompatch_core::ApplyError) -> Self {
        Self::Apply(e)
    }
}
