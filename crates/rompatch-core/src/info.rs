//! Format-agnostic "describe a patch" helpers powering the `rompatch info`
//! subcommand.
//!
//! Each [`describe`] result is best-effort: we parse the header only, never
//! decompress block bodies, and report whatever metadata is cheap to extract.
//! Anything below the detect-magic level is left to the format-specific
//! `apply` functions.

use crate::bin_file::BinReader;
use crate::error::{PatchError, Result};
use crate::format::{self, FormatKind};
use crate::hash;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatchInfo {
    pub format: FormatKind,
    pub patch_size: u64,
    pub fields: Vec<(String, String)>,
}

pub fn describe(patch: &[u8]) -> Result<PatchInfo> {
    let kind = format::detect(patch).ok_or(PatchError::InvalidMagic)?;
    let mut fields = Vec::new();
    match kind {
        FormatKind::Ips => {}
        FormatKind::Ups => describe_ups(patch, &mut fields)?,
        FormatKind::Bps => describe_bps(patch, &mut fields)?,
        FormatKind::Pmsr => describe_pmsr(patch, &mut fields)?,
        FormatKind::ApsGba => describe_aps_gba(patch, &mut fields)?,
        FormatKind::ApsN64 => describe_aps_n64(patch, &mut fields)?,
        FormatKind::Ppf => describe_ppf(patch, &mut fields)?,
        FormatKind::Rup => describe_rup(patch, &mut fields)?,
        FormatKind::Bdf => describe_bdf(patch, &mut fields)?,
    }
    Ok(PatchInfo {
        format: kind,
        patch_size: patch.len() as u64,
        fields,
    })
}

fn describe_ups(patch: &[u8], out: &mut Vec<(String, String)>) -> Result<()> {
    if patch.len() < 4 + 12 {
        return Err(PatchError::Truncated);
    }
    let mut r = BinReader::new(patch);
    let _ = r.read_bytes(4)?;
    let input_size = r.read_vlv()?;
    let output_size = r.read_vlv()?;
    let footer = &patch[patch.len() - 12..];
    let in_crc = u32::from_le_bytes(footer[0..4].try_into().unwrap());
    let out_crc = u32::from_le_bytes(footer[4..8].try_into().unwrap());
    let patch_crc = u32::from_le_bytes(footer[8..12].try_into().unwrap());
    out.push(("input size".into(), input_size.to_string()));
    out.push(("output size".into(), output_size.to_string()));
    out.push(("input CRC32".into(), format!("{in_crc:08x}")));
    out.push(("output CRC32".into(), format!("{out_crc:08x}")));
    out.push(("patch CRC32".into(), format!("{patch_crc:08x}")));
    Ok(())
}

fn describe_bps(patch: &[u8], out: &mut Vec<(String, String)>) -> Result<()> {
    if patch.len() < 4 + 12 {
        return Err(PatchError::Truncated);
    }
    let mut r = BinReader::new(patch);
    let _ = r.read_bytes(4)?;
    let source_size = r.read_vlv()?;
    let target_size = r.read_vlv()?;
    let metadata_size = r.read_vlv()?;
    let metadata_len = usize::try_from(metadata_size).unwrap_or(0);
    let metadata = if metadata_len > 0 && r.pos().saturating_add(metadata_len) <= patch.len() - 12 {
        let bytes = r.read_bytes(metadata_len)?;
        String::from_utf8_lossy(bytes).into_owned()
    } else {
        String::new()
    };
    let footer = &patch[patch.len() - 12..];
    let src_crc = u32::from_le_bytes(footer[0..4].try_into().unwrap());
    let tgt_crc = u32::from_le_bytes(footer[4..8].try_into().unwrap());
    let patch_crc = u32::from_le_bytes(footer[8..12].try_into().unwrap());
    out.push(("source size".into(), source_size.to_string()));
    out.push(("target size".into(), target_size.to_string()));
    out.push(("metadata size".into(), metadata_size.to_string()));
    if !metadata.is_empty() {
        out.push(("metadata".into(), metadata));
    }
    out.push(("source CRC32".into(), format!("{src_crc:08x}")));
    out.push(("target CRC32".into(), format!("{tgt_crc:08x}")));
    out.push(("patch CRC32".into(), format!("{patch_crc:08x}")));
    Ok(())
}

fn describe_pmsr(patch: &[u8], out: &mut Vec<(String, String)>) -> Result<()> {
    if patch.len() < 8 {
        return Err(PatchError::Truncated);
    }
    let n_records = u32::from_be_bytes(patch[4..8].try_into().unwrap());
    out.push(("record count".into(), n_records.to_string()));
    out.push((
        "expected ROM".into(),
        "Paper Mario USA1.0 (41,943,040 bytes, CRC32 a7f5cd7e)".into(),
    ));
    Ok(())
}

fn describe_aps_gba(patch: &[u8], out: &mut Vec<(String, String)>) -> Result<()> {
    if patch.len() < 12 {
        return Err(PatchError::Truncated);
    }
    let source_size = u32::from_le_bytes(patch[4..8].try_into().unwrap());
    let target_size = u32::from_le_bytes(patch[8..12].try_into().unwrap());
    let body_len = patch.len() - 12;
    let record_count = body_len / (4 + 2 + 2 + 0x0001_0000);
    out.push(("source size".into(), source_size.to_string()));
    out.push(("target size".into(), target_size.to_string()));
    out.push(("64KiB blocks".into(), record_count.to_string()));
    Ok(())
}

fn describe_aps_n64(patch: &[u8], out: &mut Vec<(String, String)>) -> Result<()> {
    if patch.len() < 5 + 1 + 1 + 50 {
        return Err(PatchError::Truncated);
    }
    let header_type = patch[5];
    let encoding = patch[6];
    let desc = trimmed_string(&patch[7..57]);
    out.push(("header type".into(), format!("{header_type:#04x}")));
    out.push(("encoding method".into(), format!("{encoding:#04x}")));
    if !desc.is_empty() {
        out.push(("description".into(), desc));
    }
    if header_type == 0x01 && patch.len() >= 0x4A + 4 {
        let cart_id = &patch[0x39 + 1..0x39 + 4];
        let crc = &patch[0x3D..0x45];
        let size = u32::from_le_bytes(patch[0x4A..0x4E].try_into().unwrap());
        out.push(("N64 cart id".into(), hash::hex(cart_id)));
        out.push(("N64 CRC field".into(), hash::hex(crc)));
        out.push(("target size".into(), size.to_string()));
    } else if patch.len() >= 0x39 + 4 {
        let size = u32::from_le_bytes(patch[0x39..0x3D].try_into().unwrap());
        out.push(("target size".into(), size.to_string()));
    }
    Ok(())
}

fn describe_ppf(patch: &[u8], out: &mut Vec<(String, String)>) -> Result<()> {
    if patch.len() < 6 + 50 {
        return Err(PatchError::Truncated);
    }
    let version_text = &patch[3..5];
    let version = match version_text {
        b"10" => 1,
        b"20" => 2,
        b"30" => 3,
        _ => return Err(PatchError::InvalidEncoding),
    };
    let desc = trimmed_string(&patch[6..56]);
    out.push(("version".into(), version.to_string()));
    if !desc.is_empty() {
        out.push(("description".into(), desc));
    }
    if version == 3 && patch.len() >= 60 {
        let image_type = patch[56];
        let block_check = patch[57] != 0;
        let undo_data = patch[58] != 0;
        out.push((
            "image type".into(),
            if image_type == 0 { "BIN" } else { "GI" }.into(),
        ));
        out.push(("block check".into(), block_check.to_string()));
        out.push(("undo data".into(), undo_data.to_string()));
    }
    Ok(())
}

fn describe_rup(patch: &[u8], out: &mut Vec<(String, String)>) -> Result<()> {
    if patch.len() < 0x0800 {
        return Err(PatchError::Truncated);
    }
    let mut r = BinReader::new(patch);
    let _ = r.read_bytes(6)?;
    let _encoding = r.read_u8()?;
    let author = trimmed_string(r.read_bytes(84)?);
    let version = trimmed_string(r.read_bytes(11)?);
    let title = trimmed_string(r.read_bytes(256)?);
    if !author.is_empty() {
        out.push(("author".into(), author));
    }
    if !version.is_empty() {
        out.push(("version".into(), version));
    }
    if !title.is_empty() {
        out.push(("title".into(), title));
    }
    Ok(())
}

fn describe_bdf(patch: &[u8], out: &mut Vec<(String, String)>) -> Result<()> {
    if patch.len() < 32 {
        return Err(PatchError::Truncated);
    }
    let ctrl = decode_bdf_offset(patch[8..16].try_into().unwrap());
    let diff = decode_bdf_offset(patch[16..24].try_into().unwrap());
    let target = decode_bdf_offset(patch[24..32].try_into().unwrap());
    out.push(("ctrl block size".into(), ctrl.to_string()));
    out.push(("diff block size".into(), diff.to_string()));
    out.push(("target size".into(), target.to_string()));
    Ok(())
}

fn decode_bdf_offset(bytes: [u8; 8]) -> i64 {
    let mut tmp = bytes;
    let neg = tmp[7] & 0x80 != 0;
    tmp[7] &= 0x7F;
    let mag = u64::from_le_bytes(tmp) as i64;
    if neg {
        mag.checked_neg().unwrap_or(i64::MIN)
    } else {
        mag
    }
}

fn trimmed_string(bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes);
    s.trim_end_matches(['\0', ' ']).trim().to_string()
}
