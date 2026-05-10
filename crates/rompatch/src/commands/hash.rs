use std::fs;
use std::path::Path;

use rompatch_core::hash;

use super::CommandError;

pub fn run(file: &Path, algo: &str) -> Result<(), CommandError> {
    let bytes = fs::read(file)?;
    let out = match algo.to_ascii_lowercase().as_str() {
        "crc32" => format!("{:08x}", hash::crc32(&bytes)),
        "md5" => hash::hex(&hash::md5(&bytes)),
        "sha1" => hash::hex(&hash::sha1(&bytes)),
        "adler32" => format!("{:08x}", hash::adler32(&bytes)),
        other => return Err(CommandError::InvalidHashSpec(other.to_string())),
    };
    println!("{out}  {}", file.display());
    Ok(())
}
