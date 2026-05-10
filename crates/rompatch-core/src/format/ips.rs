use crate::bin_file::BinReader;
use crate::error::{PatchError, Result};

const MAGIC: &[u8] = b"PATCH";
const EOF: &[u8] = b"EOF";

pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>> {
    let mut r = BinReader::new(patch);
    if r.read_bytes(MAGIC.len())? != MAGIC {
        return Err(PatchError::InvalidMagic);
    }

    let mut output = rom.to_vec();

    loop {
        if r.peek(EOF.len())? == EOF {
            r.read_bytes(EOF.len())?;
            break;
        }

        let offset = r.read_u24_be()? as usize;
        let size = r.read_u16_be()? as usize;

        if size == 0 {
            let rle_size = r.read_u16_be()? as usize;
            let fill = r.read_u8()?;
            let end = offset
                .checked_add(rle_size)
                .ok_or(PatchError::InvalidEncoding)?;
            grow_to(&mut output, end);
            for byte in &mut output[offset..end] {
                *byte = fill;
            }
        } else {
            let data = r.read_bytes(size)?;
            let end = offset
                .checked_add(size)
                .ok_or(PatchError::InvalidEncoding)?;
            grow_to(&mut output, end);
            output[offset..end].copy_from_slice(data);
        }
    }

    if r.remaining() >= 3 {
        let trunc = r.read_u24_be()? as usize;
        if trunc < output.len() {
            output.truncate(trunc);
        } else {
            grow_to(&mut output, trunc);
        }
    }

    Ok(output)
}

fn grow_to(buf: &mut Vec<u8>, len: usize) {
    if buf.len() < len {
        buf.resize(len, 0);
    }
}
