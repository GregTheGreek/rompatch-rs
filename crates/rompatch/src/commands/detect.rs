use std::fs;
use std::path::Path;

use rompatch_core::format;

use super::CommandError;

pub fn run(patch_path: &Path) -> Result<(), CommandError> {
    let bytes = fs::read(patch_path)?;
    match format::detect(&bytes) {
        Some(kind) => {
            println!("{}", kind.name());
            Ok(())
        }
        None => Err(CommandError::UnknownFormat),
    }
}
