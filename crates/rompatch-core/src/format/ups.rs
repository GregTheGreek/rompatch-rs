use crate::bin_file::BinReader;
use crate::error::{PatchError, Result};
use crate::hash;
use crate::MAX_PATCH_OUTPUT_SIZE;

const MAGIC: &[u8] = b"UPS1";
const FOOTER_LEN: usize = 12;

pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    if patch.len() < MAGIC.len() + FOOTER_LEN {
        return Err(PatchError::Truncated);
    }

    let mut r = BinReader::new(patch);
    if r.read_bytes(MAGIC.len())? != MAGIC {
        return Err(PatchError::InvalidMagic);
    }

    let input_size = r.read_vlv()?;
    let output_size = r.read_vlv()?;

    if rom.len() as u64 != input_size {
        return Err(PatchError::InputSizeMismatch {
            expected: input_size,
            actual: rom.len() as u64,
        });
    }

    let body_end = patch.len() - FOOTER_LEN;
    if r.pos() > body_end {
        return Err(PatchError::Truncated);
    }

    let output_size_usize =
        usize::try_from(output_size).map_err(|_| PatchError::OffsetOutOfRange {
            offset: output_size,
            max: usize::MAX as u64,
        })?;
    if output_size_usize > MAX_PATCH_OUTPUT_SIZE {
        return Err(PatchError::OutputTooLarge {
            declared: output_size,
            max: MAX_PATCH_OUTPUT_SIZE as u64,
        });
    }

    let mut output = vec![0u8; output_size_usize];
    let copy_len = rom.len().min(output_size_usize);
    output[..copy_len].copy_from_slice(&rom[..copy_len]);

    let mut pos: usize = 0;
    while r.pos() < body_end {
        let skip = r.read_vlv()?;
        if r.pos() > body_end {
            return Err(PatchError::Truncated);
        }
        let skip = usize::try_from(skip).map_err(|_| PatchError::OffsetOutOfRange {
            offset: skip,
            max: output_size,
        })?;
        pos = pos.checked_add(skip).ok_or(PatchError::OffsetOutOfRange {
            offset: 0,
            max: output_size,
        })?;

        loop {
            if r.pos() >= body_end {
                return Err(PatchError::Truncated);
            }
            let b = r.read_u8()?;
            if b == 0 {
                pos = pos.checked_add(1).ok_or(PatchError::OffsetOutOfRange {
                    offset: 0,
                    max: output_size,
                })?;
                break;
            }
            if pos >= output_size_usize {
                return Err(PatchError::OffsetOutOfRange {
                    offset: pos as u64,
                    max: output_size,
                });
            }
            let src = if pos < rom.len() { rom[pos] } else { 0 };
            output[pos] = src ^ b;
            pos += 1;
        }
    }

    let input_crc_expected = u32::from_le_bytes(patch[body_end..body_end + 4].try_into().unwrap());
    let output_crc_expected =
        u32::from_le_bytes(patch[body_end + 4..body_end + 8].try_into().unwrap());
    let patch_crc_expected =
        u32::from_le_bytes(patch[body_end + 8..body_end + 12].try_into().unwrap());

    let input_crc_actual = hash::crc32(rom);
    if input_crc_actual != input_crc_expected {
        return Err(PatchError::InputHashMismatch {
            expected: input_crc_expected,
            actual: input_crc_actual,
        });
    }

    let output_crc_actual = hash::crc32(&output);
    if output_crc_actual != output_crc_expected {
        return Err(PatchError::OutputHashMismatch {
            expected: output_crc_expected,
            actual: output_crc_actual,
        });
    }

    let patch_crc_actual = hash::crc32(&patch[..patch.len() - 4]);
    if patch_crc_actual != patch_crc_expected {
        return Err(PatchError::PatchHashMismatch {
            expected: patch_crc_expected,
            actual: patch_crc_actual,
        });
    }

    Ok(output)
}
