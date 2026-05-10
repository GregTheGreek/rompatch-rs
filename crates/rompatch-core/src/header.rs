//! ROM header detection and stripping.
//!
//! Some patch formats hash the un-headered ROM, so a headered input would
//! fail integrity checks even when the patch is valid. This module detects
//! a few common headers and provides byte-range views with the header
//! stripped.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderKind {
    SmcSnes,
    INes,
    Fds,
    Lynx,
}

impl HeaderKind {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::SmcSnes => "SMC (SNES)",
            Self::INes => "iNES (NES)",
            Self::Fds => "FDS",
            Self::Lynx => "LYNX",
        }
    }

    #[must_use]
    pub fn header_size(self) -> usize {
        match self {
            Self::SmcSnes => 512,
            Self::INes | Self::Fds => 16,
            Self::Lynx => 64,
        }
    }
}

#[must_use]
pub fn detect(rom: &[u8]) -> Option<HeaderKind> {
    if rom.starts_with(b"NES\x1A") {
        return Some(HeaderKind::INes);
    }
    if rom.starts_with(b"FDS\x1A") {
        return Some(HeaderKind::Fds);
    }
    if rom.starts_with(b"LYNX") {
        return Some(HeaderKind::Lynx);
    }
    if rom.len() % 1024 == 512 {
        return Some(HeaderKind::SmcSnes);
    }
    None
}

#[must_use]
pub fn strip(rom: &[u8], kind: HeaderKind) -> &[u8] {
    let size = kind.header_size();
    if rom.len() < size {
        return rom;
    }
    &rom[size..]
}

#[must_use]
pub fn split(rom: &[u8], kind: HeaderKind) -> (&[u8], &[u8]) {
    let size = kind.header_size().min(rom.len());
    rom.split_at(size)
}
