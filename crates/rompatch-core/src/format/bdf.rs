//! BDF / bsdiff (BSDIFF40 magic).
//!
//! 32-byte header: 8-byte ASCII magic "BSDIFF40", three 8-byte signed
//! integers carrying the compressed-byte sizes of the control and diff
//! blocks plus the target ROM size.
//!
//! Body: three concatenated bzip2 streams (control, diff, extra). Sizes for
//! control and diff are given by the header; extra runs to end-of-file.
//!
//! bsdiff's 8-byte signed integer encoding stores magnitude in little-endian
//! bytes 0..7, with the top bit of byte 7 acting as a sign flag (set =
//! negative). The encoding is symmetric (no two's-complement for the
//! magnitude).

use std::io::Read;

use bzip2_rs::DecoderReader;

use crate::bin_file::BinReader;
use crate::error::{PatchError, Result};

const MAGIC: &[u8] = b"BSDIFF40";
const HEADER_LEN: usize = 32;
const RECORD_LEN: usize = 24;

#[allow(clippy::too_many_lines, clippy::many_single_char_names)]
pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    if patch.len() < HEADER_LEN {
        return Err(PatchError::Truncated);
    }
    let mut r = BinReader::new(patch);
    if r.read_bytes(MAGIC.len())? != MAGIC {
        return Err(PatchError::InvalidMagic);
    }
    let ctrl_signed = read_offset(&mut r)?;
    let diff_signed = read_offset(&mut r)?;
    let target_signed = read_offset(&mut r)?;

    if ctrl_signed < 0 || diff_signed < 0 || target_signed < 0 {
        return Err(PatchError::InvalidEncoding);
    }
    let ctrl_compressed_len =
        usize::try_from(ctrl_signed).map_err(|_| PatchError::OffsetOutOfRange {
            offset: ctrl_signed as u64,
            max: usize::MAX as u64,
        })?;
    let diff_compressed_len =
        usize::try_from(diff_signed).map_err(|_| PatchError::OffsetOutOfRange {
            offset: diff_signed as u64,
            max: usize::MAX as u64,
        })?;
    let target_size = usize::try_from(target_signed).map_err(|_| PatchError::OffsetOutOfRange {
        offset: target_signed as u64,
        max: usize::MAX as u64,
    })?;

    let ctrl_end = HEADER_LEN
        .checked_add(ctrl_compressed_len)
        .ok_or(PatchError::Truncated)?;
    let diff_end = ctrl_end
        .checked_add(diff_compressed_len)
        .ok_or(PatchError::Truncated)?;
    if diff_end > patch.len() {
        return Err(PatchError::Truncated);
    }

    let ctrl = bz_decompress(&patch[HEADER_LEN..ctrl_end])?;
    let diff = bz_decompress(&patch[ctrl_end..diff_end])?;
    let extra = bz_decompress(&patch[diff_end..])?;

    let mut output = vec![0u8; target_size];
    let mut output_pos: usize = 0;
    let mut source_pos: i64 = 0;
    let mut diff_pos: usize = 0;
    let mut extra_pos: usize = 0;

    let mut c = BinReader::new(&ctrl);
    while c.remaining() > 0 {
        if c.remaining() < RECORD_LEN {
            return Err(PatchError::InvalidEncoding);
        }
        let x_signed = read_offset(&mut c)?;
        let y_signed = read_offset(&mut c)?;
        let z = read_offset(&mut c)?;

        if x_signed < 0 || y_signed < 0 {
            return Err(PatchError::InvalidEncoding);
        }
        let x = usize::try_from(x_signed).map_err(|_| PatchError::OffsetOutOfRange {
            offset: x_signed as u64,
            max: target_size as u64,
        })?;
        let y = usize::try_from(y_signed).map_err(|_| PatchError::OffsetOutOfRange {
            offset: y_signed as u64,
            max: target_size as u64,
        })?;

        // Diff segment: output[output_pos..+x] = source[source_pos..+x] + diff[diff_pos..+x],
        // where source bytes outside [0, rom.len()) contribute 0 (canonical bsdiff behaviour).
        let new_output = output_pos
            .checked_add(x)
            .ok_or(PatchError::OffsetOutOfRange {
                offset: output_pos as u64,
                max: target_size as u64,
            })?;
        if new_output > target_size {
            return Err(PatchError::OffsetOutOfRange {
                offset: new_output as u64,
                max: target_size as u64,
            });
        }
        let new_diff = diff_pos.checked_add(x).ok_or(PatchError::Truncated)?;
        if new_diff > diff.len() {
            return Err(PatchError::Truncated);
        }
        for i in 0..x {
            let abs = source_pos.wrapping_add(i as i64);
            let src = if abs >= 0 && (abs as u64) < rom.len() as u64 {
                rom[abs as usize]
            } else {
                0
            };
            output[output_pos + i] = src.wrapping_add(diff[diff_pos + i]);
        }
        output_pos = new_output;
        diff_pos = new_diff;
        source_pos = source_pos
            .checked_add(x as i64)
            .ok_or(PatchError::InvalidEncoding)?;

        // Extra segment: copy `y` bytes from the extra stream into output.
        let new_output_extra = output_pos
            .checked_add(y)
            .ok_or(PatchError::OffsetOutOfRange {
                offset: output_pos as u64,
                max: target_size as u64,
            })?;
        if new_output_extra > target_size {
            return Err(PatchError::OffsetOutOfRange {
                offset: new_output_extra as u64,
                max: target_size as u64,
            });
        }
        let new_extra = extra_pos.checked_add(y).ok_or(PatchError::Truncated)?;
        if new_extra > extra.len() {
            return Err(PatchError::Truncated);
        }
        output[output_pos..new_output_extra].copy_from_slice(&extra[extra_pos..new_extra]);
        output_pos = new_output_extra;
        extra_pos = new_extra;

        // Source seek: signed.
        source_pos = source_pos
            .checked_add(z)
            .ok_or(PatchError::InvalidEncoding)?;
    }

    if output_pos != target_size {
        return Err(PatchError::Truncated);
    }

    Ok(output)
}

fn read_offset(r: &mut BinReader<'_>) -> Result<i64> {
    let bytes = r.read_bytes(8)?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(bytes);
    Ok(decode_signed_i64(buf))
}

fn decode_signed_i64(bytes: [u8; 8]) -> i64 {
    let mut tmp = bytes;
    let neg = tmp[7] & 0x80 != 0;
    tmp[7] &= 0x7F;
    let magnitude = u64::from_le_bytes(tmp);
    if magnitude > i64::MAX as u64 {
        return i64::MAX;
    }
    let mag = magnitude as i64;
    if neg {
        mag.checked_neg().unwrap_or(i64::MIN)
    } else {
        mag
    }
}

fn bz_decompress(input: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    DecoderReader::new(input)
        .read_to_end(&mut out)
        .map_err(|_| PatchError::InvalidEncoding)?;
    Ok(out)
}
