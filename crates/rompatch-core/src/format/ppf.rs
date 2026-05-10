//! PPF (`PlayStation` Patch File) v1/v2/v3.
//!
//! Header is byte-oriented; record offsets are little-endian (32-bit for v1
//! and v2, 64-bit for v3).
//!
//! v3 may carry an optional 1024-byte source `blockCheck` snapshot, an
//! `undoData` flag (each record is followed by a copy of the original bytes),
//! and a trailing `@BEGIN_FILE_ID.DIZ` block. The trailer is detected by
//! peeking four bytes at each record boundary for the marker `@BEG`.
//!
//! No source/output hash verification is performed (the JS reference
//! implementation does not verify the blockCheck data either).

use crate::bin_file::BinReader;
use crate::error::{PatchError, Result};

const MAGIC: &[u8] = b"PPF";
const FILE_ID_MARKER: &[u8] = b"@BEG";
const DESCRIPTION_LEN: usize = 50;
const BLOCK_CHECK_LEN: usize = 1024;

#[derive(Debug, Clone, Copy)]
enum Version {
    V1,
    V2,
    V3,
}

pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    let mut r = BinReader::new(patch);
    if r.read_bytes(MAGIC.len())? != MAGIC {
        return Err(PatchError::InvalidMagic);
    }

    let version_text = r.read_bytes(2)?;
    let version = match version_text {
        b"10" => Version::V1,
        b"20" => Version::V2,
        b"30" => Version::V3,
        _ => return Err(PatchError::InvalidEncoding),
    };
    let encoded_version = r.read_u8()?;
    let expected_byte = match version {
        Version::V1 => 0u8,
        Version::V2 => 1u8,
        Version::V3 => 2u8,
    };
    if encoded_version != expected_byte {
        return Err(PatchError::InvalidEncoding);
    }
    let _description = r.read_bytes(DESCRIPTION_LEN)?;

    let (block_check, undo_data) = match version {
        Version::V1 => (false, false),
        Version::V2 => {
            let _input_size = r.read_u32_be()?;
            (true, false)
        }
        Version::V3 => {
            let _image_type = r.read_u8()?;
            let bc = r.read_u8()? != 0;
            let undo = r.read_u8()? != 0;
            let _dummy = r.read_u8()?;
            (bc, undo)
        }
    };

    if block_check {
        let _ = r.read_bytes(BLOCK_CHECK_LEN)?;
    }

    let mut output = rom.to_vec();
    while r.pos() < patch.len() {
        if r.remaining() >= 4 && r.peek(4)? == FILE_ID_MARKER {
            break;
        }
        let offset = match version {
            Version::V3 => {
                let raw = r.read_u64_le()?;
                usize::try_from(raw).map_err(|_| PatchError::OffsetOutOfRange {
                    offset: raw,
                    max: usize::MAX as u64,
                })?
            }
            _ => r.read_u32_le()? as usize,
        };
        let length = r.read_u8()? as usize;
        let data = r.read_bytes(length)?.to_vec();
        if undo_data {
            let _ = r.read_bytes(length)?;
        }

        let end = offset
            .checked_add(length)
            .ok_or(PatchError::OffsetOutOfRange {
                offset: offset as u64,
                max: u64::MAX,
            })?;
        if end > output.len() {
            output.resize(end, 0);
        }
        output[offset..end].copy_from_slice(&data);
    }

    Ok(output)
}
