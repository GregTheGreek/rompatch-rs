use std::fs;
use std::path::Path;

use rompatch_core::HashAlgo;

use super::CommandError;

pub fn run(file: &Path, algo: &str) -> Result<(), CommandError> {
    let algo =
        HashAlgo::parse(algo).ok_or_else(|| CommandError::InvalidHashAlgo(algo.to_string()))?;
    let bytes = fs::read(file)?;
    println!("{}  {}", algo.compute_hex(&bytes), file.display());
    Ok(())
}
