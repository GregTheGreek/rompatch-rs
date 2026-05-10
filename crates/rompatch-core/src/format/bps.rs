use crate::bin_file::BinReader;
use crate::error::{PatchError, Result};
use crate::hash;
use crate::MAX_PATCH_OUTPUT_SIZE;

const MAGIC: &[u8] = b"BPS1";
const FOOTER_LEN: usize = 12;

const ACTION_SOURCE_READ: u64 = 0;
const ACTION_TARGET_READ: u64 = 1;
const ACTION_SOURCE_COPY: u64 = 2;
const ACTION_TARGET_COPY: u64 = 3;

#[allow(clippy::too_many_lines)]
pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    if patch.len() < MAGIC.len() + FOOTER_LEN {
        return Err(PatchError::Truncated);
    }
    let mut r = BinReader::new(patch);
    if r.read_bytes(MAGIC.len())? != MAGIC {
        return Err(PatchError::InvalidMagic);
    }

    let source_size = r.read_vlv()?;
    let target_size = r.read_vlv()?;
    let metadata_size = r.read_vlv()?;

    if rom.len() as u64 != source_size {
        return Err(PatchError::InputSizeMismatch {
            expected: source_size,
            actual: rom.len() as u64,
        });
    }

    let body_end = patch.len() - FOOTER_LEN;
    let metadata_size_usize =
        usize::try_from(metadata_size).map_err(|_| PatchError::OffsetOutOfRange {
            offset: metadata_size,
            max: body_end as u64,
        })?;
    let after_metadata = r
        .pos()
        .checked_add(metadata_size_usize)
        .ok_or(PatchError::Truncated)?;
    if after_metadata > body_end {
        return Err(PatchError::Truncated);
    }
    let _ = r.read_bytes(metadata_size_usize)?;

    let target_size_usize =
        usize::try_from(target_size).map_err(|_| PatchError::OffsetOutOfRange {
            offset: target_size,
            max: usize::MAX as u64,
        })?;
    if target_size_usize > MAX_PATCH_OUTPUT_SIZE {
        return Err(PatchError::OutputTooLarge {
            declared: target_size,
            max: MAX_PATCH_OUTPUT_SIZE as u64,
        });
    }

    let mut output = vec![0u8; target_size_usize];
    let mut output_offset: usize = 0;
    let mut source_relative_offset: i64 = 0;
    let mut target_relative_offset: i64 = 0;

    while r.pos() < body_end {
        let command = r.read_vlv()?;
        let action = command & 3;
        let length_u64 = (command >> 2)
            .checked_add(1)
            .ok_or(PatchError::InvalidEncoding)?;
        let length = usize::try_from(length_u64).map_err(|_| PatchError::OffsetOutOfRange {
            offset: length_u64,
            max: target_size,
        })?;

        let end = output_offset
            .checked_add(length)
            .ok_or(PatchError::OffsetOutOfRange {
                offset: 0,
                max: target_size,
            })?;
        if end > target_size_usize {
            return Err(PatchError::OffsetOutOfRange {
                offset: end as u64,
                max: target_size,
            });
        }

        match action {
            ACTION_SOURCE_READ => {
                if end > rom.len() {
                    return Err(PatchError::OffsetOutOfRange {
                        offset: end as u64,
                        max: rom.len() as u64,
                    });
                }
                output[output_offset..end].copy_from_slice(&rom[output_offset..end]);
            }
            ACTION_TARGET_READ => {
                let next = r.pos().checked_add(length).ok_or(PatchError::Truncated)?;
                if next > body_end {
                    return Err(PatchError::Truncated);
                }
                let bytes = r.read_bytes(length)?;
                output[output_offset..end].copy_from_slice(bytes);
            }
            ACTION_SOURCE_COPY => {
                let delta = read_signed_vlv(&mut r)?;
                source_relative_offset = source_relative_offset
                    .checked_add(delta)
                    .ok_or(PatchError::InvalidEncoding)?;
                if source_relative_offset < 0 {
                    return Err(PatchError::OffsetOutOfRange {
                        offset: 0,
                        max: rom.len() as u64,
                    });
                }
                let start = usize::try_from(source_relative_offset).map_err(|_| {
                    PatchError::OffsetOutOfRange {
                        offset: source_relative_offset as u64,
                        max: rom.len() as u64,
                    }
                })?;
                let copy_end = start
                    .checked_add(length)
                    .ok_or(PatchError::OffsetOutOfRange {
                        offset: start as u64,
                        max: rom.len() as u64,
                    })?;
                if copy_end > rom.len() {
                    return Err(PatchError::OffsetOutOfRange {
                        offset: copy_end as u64,
                        max: rom.len() as u64,
                    });
                }
                output[output_offset..end].copy_from_slice(&rom[start..copy_end]);
                source_relative_offset = source_relative_offset
                    .checked_add(i64::try_from(length).map_err(|_| PatchError::InvalidEncoding)?)
                    .ok_or(PatchError::InvalidEncoding)?;
            }
            ACTION_TARGET_COPY => {
                let delta = read_signed_vlv(&mut r)?;
                target_relative_offset = target_relative_offset
                    .checked_add(delta)
                    .ok_or(PatchError::InvalidEncoding)?;
                if target_relative_offset < 0 {
                    return Err(PatchError::OffsetOutOfRange {
                        offset: 0,
                        max: target_size,
                    });
                }
                let start = usize::try_from(target_relative_offset).map_err(|_| {
                    PatchError::OffsetOutOfRange {
                        offset: target_relative_offset as u64,
                        max: target_size,
                    }
                })?;
                // Byte-by-byte to allow self-overlap (RLE pattern: delta = -k, length > k).
                for i in 0..length {
                    let src_idx = start.checked_add(i).ok_or(PatchError::OffsetOutOfRange {
                        offset: 0,
                        max: target_size,
                    })?;
                    if src_idx >= output_offset + i {
                        return Err(PatchError::OffsetOutOfRange {
                            offset: src_idx as u64,
                            max: target_size,
                        });
                    }
                    output[output_offset + i] = output[src_idx];
                }
                target_relative_offset = target_relative_offset
                    .checked_add(i64::try_from(length).map_err(|_| PatchError::InvalidEncoding)?)
                    .ok_or(PatchError::InvalidEncoding)?;
            }
            _ => unreachable!("action mask is two bits"),
        }

        output_offset = end;
    }

    if output_offset != target_size_usize {
        return Err(PatchError::Truncated);
    }

    let source_crc_expected = u32::from_le_bytes(patch[body_end..body_end + 4].try_into().unwrap());
    let target_crc_expected =
        u32::from_le_bytes(patch[body_end + 4..body_end + 8].try_into().unwrap());
    let patch_crc_expected =
        u32::from_le_bytes(patch[body_end + 8..body_end + 12].try_into().unwrap());

    let source_crc_actual = hash::crc32(rom);
    if source_crc_actual != source_crc_expected {
        return Err(PatchError::InputHashMismatch {
            expected: source_crc_expected,
            actual: source_crc_actual,
        });
    }

    let target_crc_actual = hash::crc32(&output);
    if target_crc_actual != target_crc_expected {
        return Err(PatchError::OutputHashMismatch {
            expected: target_crc_expected,
            actual: target_crc_actual,
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

fn read_signed_vlv(r: &mut BinReader<'_>) -> Result<i64> {
    let raw = r.read_vlv()?;
    let magnitude = (raw >> 1) as i64;
    if raw & 1 != 0 {
        Ok(-magnitude)
    } else {
        Ok(magnitude)
    }
}
