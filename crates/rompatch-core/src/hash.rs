use md5::{Digest, Md5};
use sha1::Sha1;

#[must_use]
pub fn crc32(data: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

#[must_use]
pub fn md5(data: &[u8]) -> [u8; 16] {
    let mut hasher = Md5::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[must_use]
pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[must_use]
pub fn adler32(data: &[u8]) -> u32 {
    let mut hasher = adler2::Adler32::new();
    hasher.write_slice(data);
    hasher.checksum()
}

#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    use core::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}
