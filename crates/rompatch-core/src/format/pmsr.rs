//! PMSR (Paper Mario Star Rod) patches.
//!
//! Header is 8 bytes: 4-byte ASCII magic "PMSR" + big-endian u32 record count.
//! Each record is `(u32 BE offset, u32 BE length, length bytes data)`.
//! No patch trailer or per-record checksum.
//!
//! Source ROM is required to be the 40 MiB Paper Mario (USA 1.0) image. We
//! enforce this by size + CRC32 to match RomPatcher.js's behaviour.

use crate::bin_file::BinReader;
use crate::error::{PatchError, Result};
use crate::hash;

const MAGIC: &[u8] = b"PMSR";

const PAPER_MARIO_USA10_SIZE: usize = 41_943_040;
const PAPER_MARIO_USA10_CRC32: u32 = 0xa7f5_cd7e;

pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    if rom.len() != PAPER_MARIO_USA10_SIZE {
        return Err(PatchError::InputSizeMismatch {
            expected: PAPER_MARIO_USA10_SIZE as u64,
            actual: rom.len() as u64,
        });
    }
    let rom_crc = hash::crc32(rom);
    if rom_crc != PAPER_MARIO_USA10_CRC32 {
        return Err(PatchError::InputHashMismatch {
            expected: PAPER_MARIO_USA10_CRC32,
            actual: rom_crc,
        });
    }
    apply_records(patch, rom)
}

/// Apply PMSR records without the Paper Mario size/CRC validation. Exposed for
/// tests; the public `apply` is the user-facing entry point.
pub fn apply_records(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    let mut r = BinReader::new(patch);
    if r.read_bytes(MAGIC.len())? != MAGIC {
        return Err(PatchError::InvalidMagic);
    }

    let n_records = r.read_u32_be()?;
    let mut output = rom.to_vec();

    for _ in 0..n_records {
        let offset = r.read_u32_be()? as usize;
        let length = r.read_u32_be()? as usize;
        let bytes = r.read_bytes(length)?;

        let end = offset
            .checked_add(length)
            .ok_or(PatchError::OffsetOutOfRange {
                offset: offset as u64,
                max: u64::MAX,
            })?;
        if end > crate::MAX_PATCH_OUTPUT_SIZE {
            return Err(PatchError::OutputTooLarge {
                declared: end as u64,
                max: crate::MAX_PATCH_OUTPUT_SIZE as u64,
            });
        }
        if end > output.len() {
            output.resize(end, 0);
        }
        output[offset..end].copy_from_slice(bytes);
    }

    Ok(output)
}
