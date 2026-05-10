//! RUP / NINJA2.
//!
//! 2 KiB header at offset 0 with fixed-width string fields followed by a
//! command stream starting at offset 0x800.
//!
//! Commands:
//! - `0x01` `OPEN_NEW_FILE` — opens a per-file scope with metadata, sizes,
//!   and 16-byte MD5s. Multi-file patches concatenate file scopes; we pick
//!   the one whose source MD5 matches the input ROM.
//! - `0x02` `XOR_RECORD` — applies an XOR delta at an offset.
//! - `0x00` END — terminates a file scope, or (at top level) the patch.
//!
//! RUP uses its own variable-length value encoding (one byte count followed
//! by little-endian magnitude bytes), distinct from byuu's VLV used by
//! BPS/UPS.
//!
//! Undo (matching against the target MD5) and overflow mode `M` (target
//! smaller than source) are not implemented.

use crate::bin_file::BinReader;
use crate::error::{PatchError, Result};
use crate::hash;
use crate::MAX_PATCH_OUTPUT_SIZE;

const MAGIC: &[u8] = b"NINJA2";
const HEADER_LEN: usize = 0x800;

const CMD_END: u8 = 0x00;
const CMD_OPEN_FILE: u8 = 0x01;
const CMD_XOR_RECORD: u8 = 0x02;

const MD5_LEN: usize = 16;

pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    if patch.len() < HEADER_LEN {
        return Err(PatchError::Truncated);
    }
    let mut r = BinReader::new(patch);
    if r.read_bytes(MAGIC.len())? != MAGIC {
        return Err(PatchError::InvalidMagic);
    }
    r.seek(HEADER_LEN)?;

    let rom_md5 = hash::md5(rom);

    loop {
        let cmd = r.read_u8()?;
        if cmd == CMD_END {
            return Err(PatchError::NoMatchingFile);
        }
        if cmd != CMD_OPEN_FILE {
            return Err(PatchError::InvalidEncoding);
        }
        let file = read_file_header(&mut r)?;

        if file.source_md5 == rom_md5 {
            return apply_file(&mut r, &file, rom);
        }
        // Skip this file's records (cmd=0x02 ... 0x00).
        loop {
            let sub = r.read_u8()?;
            if sub == CMD_END {
                break;
            }
            if sub != CMD_XOR_RECORD {
                return Err(PatchError::InvalidEncoding);
            }
            let _offset = read_rup_vlv(&mut r)?;
            let xor_len = read_rup_vlv(&mut r)?;
            let xor_len_usize =
                usize::try_from(xor_len).map_err(|_| PatchError::OffsetOutOfRange {
                    offset: xor_len,
                    max: usize::MAX as u64,
                })?;
            let _ = r.read_bytes(xor_len_usize)?;
        }
    }
}

struct FileHeader {
    source_size: u64,
    target_size: u64,
    source_md5: [u8; MD5_LEN],
    target_md5: [u8; MD5_LEN],
    overflow_mode: Option<u8>,
    overflow_data: Vec<u8>,
}

fn read_file_header(r: &mut BinReader<'_>) -> Result<FileHeader> {
    let filename_len = read_rup_vlv(r)?;
    let filename_len = usize::try_from(filename_len).map_err(|_| PatchError::OffsetOutOfRange {
        offset: filename_len,
        max: usize::MAX as u64,
    })?;
    let _filename = r.read_bytes(filename_len)?;
    let _rom_type = r.read_u8()?;
    let source_size = read_rup_vlv(r)?;
    let target_size = read_rup_vlv(r)?;

    let source_md5_slice = r.read_bytes(MD5_LEN)?;
    let mut source_md5 = [0u8; MD5_LEN];
    source_md5.copy_from_slice(source_md5_slice);
    let target_md5_slice = r.read_bytes(MD5_LEN)?;
    let mut target_md5 = [0u8; MD5_LEN];
    target_md5.copy_from_slice(target_md5_slice);

    let (overflow_mode, overflow_data) = if source_size == target_size {
        (None, Vec::new())
    } else {
        let mode = r.read_u8()?;
        let len = read_rup_vlv(r)?;
        let len = usize::try_from(len).map_err(|_| PatchError::OffsetOutOfRange {
            offset: len,
            max: usize::MAX as u64,
        })?;
        let raw = r.read_bytes(len)?;
        let decoded: Vec<u8> = raw.iter().map(|b| b ^ 0xFF).collect();
        (Some(mode), decoded)
    };

    Ok(FileHeader {
        source_size,
        target_size,
        source_md5,
        target_md5,
        overflow_mode,
        overflow_data,
    })
}

fn apply_file(r: &mut BinReader<'_>, file: &FileHeader, rom: &[u8]) -> Result<Vec<u8>> {
    let target_size =
        usize::try_from(file.target_size).map_err(|_| PatchError::OffsetOutOfRange {
            offset: file.target_size,
            max: usize::MAX as u64,
        })?;
    if target_size > MAX_PATCH_OUTPUT_SIZE {
        return Err(PatchError::OutputTooLarge {
            declared: file.target_size,
            max: MAX_PATCH_OUTPUT_SIZE as u64,
        });
    }
    let source_size =
        usize::try_from(file.source_size).map_err(|_| PatchError::OffsetOutOfRange {
            offset: file.source_size,
            max: usize::MAX as u64,
        })?;

    let mut output = vec![0u8; target_size];
    let copy_len = source_size.min(target_size).min(rom.len());
    output[..copy_len].copy_from_slice(&rom[..copy_len]);

    if let Some(mode) = file.overflow_mode {
        match mode {
            b'A' => {
                if file.target_size <= file.source_size {
                    return Err(PatchError::InvalidEncoding);
                }
                let start = source_size;
                let end = start
                    .saturating_add(file.overflow_data.len())
                    .min(target_size);
                let take = end.saturating_sub(start);
                output[start..start + take].copy_from_slice(&file.overflow_data[..take]);
            }
            b'M' => {
                return Err(PatchError::UnsupportedFeature(
                    "RUP overflow mode 'M' (undo)",
                ));
            }
            _ => return Err(PatchError::InvalidEncoding),
        }
    }

    loop {
        let sub = r.read_u8()?;
        if sub == CMD_END {
            break;
        }
        if sub != CMD_XOR_RECORD {
            return Err(PatchError::InvalidEncoding);
        }
        let offset = read_rup_vlv(r)?;
        let xor_len = read_rup_vlv(r)?;
        let offset = usize::try_from(offset).map_err(|_| PatchError::OffsetOutOfRange {
            offset,
            max: target_size as u64,
        })?;
        let xor_len = usize::try_from(xor_len).map_err(|_| PatchError::OffsetOutOfRange {
            offset: xor_len,
            max: target_size as u64,
        })?;
        let xor = r.read_bytes(xor_len)?;
        let end = offset
            .checked_add(xor_len)
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
        for i in 0..xor_len {
            let src = if offset + i < rom.len() {
                rom[offset + i]
            } else {
                0
            };
            output[offset + i] = src ^ xor[i];
        }
    }

    let actual = hash::md5(&output);
    if actual != file.target_md5 {
        return Err(PatchError::OutputMd5Mismatch {
            expected: file.target_md5,
            actual,
        });
    }

    Ok(output)
}

/// RUP's VLV: a u8 byte-count, then `n` little-endian magnitude bytes.
fn read_rup_vlv(r: &mut BinReader<'_>) -> Result<u64> {
    let n = r.read_u8()? as usize;
    if n > 8 {
        return Err(PatchError::InvalidEncoding);
    }
    let mut value: u64 = 0;
    for i in 0..n {
        let b = r.read_u8()?;
        value |= u64::from(b) << (i * 8);
    }
    Ok(value)
}
