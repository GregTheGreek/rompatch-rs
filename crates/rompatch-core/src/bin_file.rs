use crate::error::{PatchError, Result};

pub struct BinReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> BinReader<'a> {
    #[must_use]
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    #[must_use]
    pub fn pos(&self) -> usize {
        self.pos
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    #[must_use]
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn seek(&mut self, pos: usize) -> Result<()> {
        if pos > self.buf.len() {
            return Err(PatchError::Truncated);
        }
        self.pos = pos;
        Ok(())
    }

    pub fn peek(&self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or(PatchError::Truncated)?;
        if end > self.buf.len() {
            return Err(PatchError::Truncated);
        }
        Ok(&self.buf[self.pos..end])
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let b = *self.buf.get(self.pos).ok_or(PatchError::Truncated)?;
        self.pos += 1;
        Ok(b)
    }

    pub fn read_u16_be(&mut self) -> Result<u16> {
        let b = self.read_bytes(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    pub fn read_u24_be(&mut self) -> Result<u32> {
        let b = self.read_bytes(3)?;
        Ok(u32::from_be_bytes([0, b[0], b[1], b[2]]))
    }

    pub fn read_u32_le(&mut self) -> Result<u32> {
        let b = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or(PatchError::Truncated)?;
        if end > self.buf.len() {
            return Err(PatchError::Truncated);
        }
        let slice = &self.buf[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    /// byuu's variable-length value encoding (UPS/BPS/RUP).
    pub fn read_vlv(&mut self) -> Result<u64> {
        let mut value: u64 = 0;
        let mut shift: u64 = 1;
        loop {
            let byte = self.read_u8()?;
            let chunk = u64::from(byte & 0x7f)
                .checked_mul(shift)
                .ok_or(PatchError::InvalidEncoding)?;
            value = value
                .checked_add(chunk)
                .ok_or(PatchError::InvalidEncoding)?;
            if byte & 0x80 != 0 {
                break;
            }
            shift = shift.checked_shl(7).ok_or(PatchError::InvalidEncoding)?;
            value = value
                .checked_add(shift)
                .ok_or(PatchError::InvalidEncoding)?;
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_vlv(out: &mut Vec<u8>, mut value: u64) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                out.push(byte | 0x80);
                return;
            }
            out.push(byte);
            value -= 1;
        }
    }

    #[test]
    fn vlv_roundtrips_known_values() {
        let cases: &[u64] = &[
            0,
            1,
            0x7f,
            0x80,
            0x100,
            0x1234,
            0xdead_beef,
            u64::from(u32::MAX),
        ];
        for &v in cases {
            let mut buf = Vec::new();
            write_vlv(&mut buf, v);
            let mut r = BinReader::new(&buf);
            assert_eq!(r.read_vlv().unwrap(), v, "value {v} did not roundtrip");
        }
    }

    #[test]
    fn read_past_end_returns_truncated() {
        let mut r = BinReader::new(&[1, 2]);
        assert_eq!(r.read_u32_le(), Err(PatchError::Truncated));
    }

    #[test]
    fn peek_does_not_advance() {
        let r = BinReader::new(&[1, 2, 3, 4]);
        assert_eq!(r.peek(2).unwrap(), &[1, 2]);
        assert_eq!(r.pos(), 0);
    }
}
