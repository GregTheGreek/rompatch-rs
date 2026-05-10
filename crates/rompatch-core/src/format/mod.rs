pub mod aps;
pub mod bps;
pub mod ips;
pub mod pmsr;
pub mod ppf;
pub mod rup;
pub mod ups;

use crate::error::{PatchError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatKind {
    Ips,
    Ups,
    Bps,
    Pmsr,
    ApsGba,
    ApsN64,
    Ppf,
    Rup,
}

impl FormatKind {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Ips => "IPS",
            Self::Ups => "UPS",
            Self::Bps => "BPS",
            Self::Pmsr => "PMSR",
            Self::ApsGba => "APS-GBA",
            Self::ApsN64 => "APS-N64",
            Self::Ppf => "PPF",
            Self::Rup => "RUP",
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
    } else if patch.starts_with(b"PMSR") {
        Some(FormatKind::Pmsr)
    } else if aps::is_n64(patch) {
        Some(FormatKind::ApsN64)
    } else if aps::is_gba(patch) {
        Some(FormatKind::ApsGba)
    } else if patch.starts_with(b"PPF") {
        Some(FormatKind::Ppf)
    } else if patch.starts_with(b"NINJA2") {
        Some(FormatKind::Rup)
    } else {
        None
    }
}

pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    match detect(patch) {
        Some(FormatKind::Ips) => ips::apply(patch, rom),
        Some(FormatKind::Ups) => ups::apply(patch, rom),
        Some(FormatKind::Bps) => bps::apply(patch, rom),
        Some(FormatKind::Pmsr) => pmsr::apply(patch, rom),
        Some(FormatKind::ApsGba) => aps::apply_gba(patch, rom),
        Some(FormatKind::ApsN64) => aps::apply_n64(patch, rom),
        Some(FormatKind::Ppf) => ppf::apply(patch, rom),
        Some(FormatKind::Rup) => rup::apply(patch, rom),
        None => Err(PatchError::InvalidMagic),
    }
}
