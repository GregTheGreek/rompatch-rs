pub mod aps;
pub mod bdf;
pub mod bps;
pub mod ips;
pub mod pmsr;
pub mod ppf;
pub mod rup;
pub mod ups;

use crate::error::{PatchError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatKind {
    Ips,
    Ups,
    Bps,
    Pmsr,
    ApsGba,
    ApsN64,
    Ppf,
    Rup,
    Bdf,
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
            Self::Bdf => "BDF",
        }
    }

    /// Parse a user-facing format name into a [`FormatKind`].
    ///
    /// Accepts case-insensitive variants and a few common aliases
    /// (`aps-gba`/`aps_gba`/`apsgba`, `bsdiff` for `bdf`).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "ips" => Some(Self::Ips),
            "ups" => Some(Self::Ups),
            "bps" => Some(Self::Bps),
            "pmsr" => Some(Self::Pmsr),
            "aps-gba" | "aps_gba" | "apsgba" => Some(Self::ApsGba),
            "aps-n64" | "aps_n64" | "apsn64" => Some(Self::ApsN64),
            "ppf" => Some(Self::Ppf),
            "rup" => Some(Self::Rup),
            "bdf" | "bsdiff" => Some(Self::Bdf),
            _ => None,
        }
    }

    /// Apply a patch of this format to `rom`, returning the patched output.
    pub fn apply(self, patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
        match self {
            Self::Ips => ips::apply(patch, rom),
            Self::Ups => ups::apply(patch, rom),
            Self::Bps => bps::apply(patch, rom),
            Self::Pmsr => pmsr::apply(patch, rom),
            Self::ApsGba => aps::apply_gba(patch, rom),
            Self::ApsN64 => aps::apply_n64(patch, rom),
            Self::Ppf => ppf::apply(patch, rom),
            Self::Rup => rup::apply(patch, rom),
            Self::Bdf => bdf::apply(patch, rom),
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
    } else if patch.starts_with(b"BSDIFF40") {
        Some(FormatKind::Bdf)
    } else {
        None
    }
}

pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    detect(patch)
        .ok_or(PatchError::InvalidMagic)?
        .apply(patch, rom)
}
