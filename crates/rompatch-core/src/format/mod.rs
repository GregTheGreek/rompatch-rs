pub mod bps;
pub mod ips;
pub mod ups;

use crate::error::{PatchError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatKind {
    Ips,
    Ups,
    Bps,
}

impl FormatKind {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Ips => "IPS",
            Self::Ups => "UPS",
            Self::Bps => "BPS",
        }
    }
}

#[must_use]
pub fn detect(patch: &[u8]) -> Option<FormatKind> {
    if patch.starts_with(b"PATCH") {
        Some(FormatKind::Ips)
    } else if patch.starts_with(b"UPS1") {
        Some(FormatKind::Ups)
    } else if patch.starts_with(b"BPS1") {
        Some(FormatKind::Bps)
    } else {
        None
    }
}

pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    match detect(patch) {
        Some(FormatKind::Ips) => ips::apply(patch, rom),
        Some(FormatKind::Ups) => ups::apply(patch, rom),
        Some(FormatKind::Bps) => bps::apply(patch, rom),
        None => Err(PatchError::InvalidMagic),
    }
}
