//! APS patches: GBA and N64 variants.
//!
//! Both share an "APS1..." prefix so detection probes for the longer N64 magic
//! ("APS10") first.
//!
//! # APS-GBA
//!
//! - 12-byte header: 4-byte magic "APS1" + u32 LE source size + u32 LE target
//!   size.
//! - Body: a sequence of fixed 65544-byte records. Each record is u32 LE
//!   offset + u16 LE source CRC-16 + u16 LE target CRC-16 + 65 536 XOR bytes
//!   applied to the source's 64 KiB block at `offset`.
//! - Per-block CRC-16 verification is not implemented (RomPatcher.js's exact
//!   polynomial is not documented in our spec); apply still produces correct
//!   output for a correctly-matched source.
//!
//! # APS-N64
//!
//! - Variable header: 5-byte magic "APS10" + 1-byte header type (0 = simple,
//!   1 = N64) + 1-byte encoding method + 50-byte description. If header type
//!   is N64: 1-byte source format flag + 3-byte cart id + 8-byte crc field +
//!   5-byte padding. Then u32 LE target size.
//! - Body: variable-length records. u32 LE offset + u8 length, followed by
//!   either `length` raw bytes (length > 0) or a 2-byte RLE pair (length == 0).
//! - For header type 1 we additionally require source bytes 0x3C..0x3F to
//!   match the cart id and source bytes 0x10..0x17 to match the crc field.

use crate::bin_file::BinReader;
use crate::error::{PatchError, Result};

const MAGIC_GBA: &[u8] = b"APS1";
const MAGIC_N64: &[u8] = b"APS10";

const GBA_BLOCK_SIZE: usize = 0x0001_0000;
const GBA_RECORD_SIZE: usize = 4 + 2 + 2 + GBA_BLOCK_SIZE;
const GBA_HEADER_LEN: usize = 12;

const N64_HEADER_TYPE_SIMPLE: u8 = 0x00;
const N64_HEADER_TYPE_N64: u8 = 0x01;

#[must_use]
pub fn is_n64(patch: &[u8]) -> bool {
    patch.starts_with(MAGIC_N64)
}

#[must_use]
pub fn is_gba(patch: &[u8]) -> bool {
    patch.starts_with(MAGIC_GBA) && !patch.starts_with(MAGIC_N64)
}

pub fn apply_gba(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    if patch.len() < GBA_HEADER_LEN {
        return Err(PatchError::Truncated);
    }
    let mut r = BinReader::new(patch);
    if r.read_bytes(MAGIC_GBA.len())? != MAGIC_GBA {
        return Err(PatchError::InvalidMagic);
    }

    let source_size = r.read_u32_le()? as usize;
    let target_size = r.read_u32_le()? as usize;

    if rom.len() != source_size {
        return Err(PatchError::InputSizeMismatch {
            expected: source_size as u64,
            actual: rom.len() as u64,
        });
    }

    let body_len = patch.len() - GBA_HEADER_LEN;
    if !body_len.is_multiple_of(GBA_RECORD_SIZE) {
        return Err(PatchError::InvalidEncoding);
    }

    let mut output = vec![0u8; target_size];
    let copy_len = source_size.min(target_size);
    output[..copy_len].copy_from_slice(&rom[..copy_len]);

    let n_records = body_len / GBA_RECORD_SIZE;
    for _ in 0..n_records {
        let offset = r.read_u32_le()? as usize;
        let _src_crc = r.read_u16_le()?;
        let _tgt_crc = r.read_u16_le()?;
        let xor = r.read_bytes(GBA_BLOCK_SIZE)?;

        let end = offset
            .checked_add(GBA_BLOCK_SIZE)
            .ok_or(PatchError::OffsetOutOfRange {
                offset: offset as u64,
                max: target_size as u64,
            })?;
        if end > target_size {
            return Err(PatchError::OffsetOutOfRange {
                offset: end as u64,
                max: target_size as u64,
            });
        }
        for i in 0..GBA_BLOCK_SIZE {
            let src = if offset + i < rom.len() {
                rom[offset + i]
            } else {
                0
            };
            output[offset + i] = src ^ xor[i];
        }
    }

    Ok(output)
}

pub fn apply_n64(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    let mut r = BinReader::new(patch);
    if r.read_bytes(MAGIC_N64.len())? != MAGIC_N64 {
        return Err(PatchError::InvalidMagic);
    }

    let header_type = r.read_u8()?;
    let _encoding_method = r.read_u8()?;
    let _description = r.read_bytes(50)?;

    if header_type == N64_HEADER_TYPE_N64 {
        let _original_format = r.read_u8()?;
        let cart_id = r.read_bytes(3)?.to_vec();
        let crc = r.read_bytes(8)?.to_vec();
        let _pad = r.read_bytes(5)?;

        if rom.len() < 0x40 {
            return Err(PatchError::InputSizeMismatch {
                expected: 0x40,
                actual: rom.len() as u64,
            });
        }
        if &rom[0x3C..0x3F] != cart_id.as_slice() {
            return Err(PatchError::InputHashMismatch {
                expected: u32::from_be_bytes([0, cart_id[0], cart_id[1], cart_id[2]]),
                actual: u32::from_be_bytes([0, rom[0x3C], rom[0x3D], rom[0x3E]]),
            });
        }
        if &rom[0x10..0x18] != crc.as_slice() {
            return Err(PatchError::InputHashMismatch {
                expected: u32::from_be_bytes([crc[0], crc[1], crc[2], crc[3]]),
                actual: u32::from_be_bytes([rom[0x10], rom[0x11], rom[0x12], rom[0x13]]),
            });
        }
    } else if header_type != N64_HEADER_TYPE_SIMPLE {
        return Err(PatchError::InvalidEncoding);
    }

    let target_size = r.read_u32_le()? as usize;
    let mut output = vec![0u8; target_size];
    let copy_len = rom.len().min(target_size);
    output[..copy_len].copy_from_slice(&rom[..copy_len]);

    while r.pos() < patch.len() {
        let offset = r.read_u32_le()? as usize;
        let length = r.read_u8()? as usize;
        if length == 0 {
            let byte = r.read_u8()?;
            let rle_len = r.read_u8()? as usize;
            let end = offset
                .checked_add(rle_len)
                .ok_or(PatchError::OffsetOutOfRange {
                    offset: offset as u64,
                    max: target_size as u64,
                })?;
            if end > target_size {
                return Err(PatchError::OffsetOutOfRange {
                    offset: end as u64,
                    max: target_size as u64,
                });
            }
            output[offset..end].fill(byte);
        } else {
            let bytes = r.read_bytes(length)?;
            let end = offset
                .checked_add(length)
                .ok_or(PatchError::OffsetOutOfRange {
                    offset: offset as u64,
                    max: target_size as u64,
                })?;
            if end > target_size {
                return Err(PatchError::OffsetOutOfRange {
                    offset: end as u64,
                    max: target_size as u64,
                });
            }
            output[offset..end].copy_from_slice(bytes);
        }
    }

    Ok(output)
}
