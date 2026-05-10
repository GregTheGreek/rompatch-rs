//! Recompute and patch in-ROM checksums for Game Boy and Mega Drive headers.
//!
//! Many ROM hacks ship a patched body without recomputing the cartridge
//! checksums; the resulting ROM boots on lenient emulators but fails on
//! hardware. These helpers fix the relevant bytes in-place.

const GB_HEADER_CHECKSUM_OFFSET: usize = 0x14D;
const GB_GLOBAL_CHECKSUM_OFFSET: usize = 0x14E;
const GB_HEADER_RANGE: core::ops::RangeInclusive<usize> = 0x134..=0x14C;

const MD_CHECKSUM_OFFSET: usize = 0x18E;
const MD_BODY_START: usize = 0x200;

/// Recompute Game Boy header byte 0x14D and global checksum 0x14E-0x14F.
///
/// Returns `false` if the ROM is too small for a GB header.
pub fn fix_game_boy(rom: &mut [u8]) -> bool {
    if rom.len() < 0x150 {
        return false;
    }
    let mut header: u8 = 0;
    for i in GB_HEADER_RANGE {
        header = header.wrapping_sub(rom[i]).wrapping_sub(1);
    }
    rom[GB_HEADER_CHECKSUM_OFFSET] = header;

    let mut global: u16 = 0;
    for (i, &b) in rom.iter().enumerate() {
        if i == GB_GLOBAL_CHECKSUM_OFFSET || i == GB_GLOBAL_CHECKSUM_OFFSET + 1 {
            continue;
        }
        global = global.wrapping_add(u16::from(b));
    }
    let bytes = global.to_be_bytes();
    rom[GB_GLOBAL_CHECKSUM_OFFSET] = bytes[0];
    rom[GB_GLOBAL_CHECKSUM_OFFSET + 1] = bytes[1];
    true
}

/// Recompute Mega Drive header u16 BE checksum at offset 0x18E as the sum of
/// all u16 BE words from offset 0x200 to the end of the ROM (odd trailing
/// bytes are ignored, matching established tooling).
///
/// Returns `false` if the ROM is too small.
pub fn fix_mega_drive(rom: &mut [u8]) -> bool {
    if rom.len() < MD_BODY_START {
        return false;
    }
    let mut sum: u16 = 0;
    let body = &rom[MD_BODY_START..];
    let pairs = body.len() / 2;
    for i in 0..pairs {
        let word = u16::from_be_bytes([body[i * 2], body[i * 2 + 1]]);
        sum = sum.wrapping_add(word);
    }
    let bytes = sum.to_be_bytes();
    rom[MD_CHECKSUM_OFFSET] = bytes[0];
    rom[MD_CHECKSUM_OFFSET + 1] = bytes[1];
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gb_header_checksum_matches_logo_zeros() {
        // A blank 32 KiB ROM: header bytes 0x134..=0x14C are all zero.
        // Each iteration: chk = chk - 0 - 1 = -1 → wrapping to 0xFF, etc.
        // 25 iterations of -1 -> 0x100 - 25 = 0xE7.
        let mut rom = vec![0u8; 0x8000];
        assert!(fix_game_boy(&mut rom));
        assert_eq!(rom[GB_HEADER_CHECKSUM_OFFSET], 0xE7);
    }

    #[test]
    fn md_checksum_handles_short_rom() {
        let mut rom = vec![0u8; 0x100];
        assert!(!fix_mega_drive(&mut rom));
    }
}
