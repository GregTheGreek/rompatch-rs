use std::fs;
use std::path::Path;

use rompatch_core::info;

use super::CommandError;

pub fn run(patch_path: &Path) -> Result<(), CommandError> {
    let bytes = fs::read(patch_path)?;
    let report = info::describe(&bytes)?;
    println!("format: {}", report.format.name());
    println!("patch size: {} bytes", report.patch_size);
    for (key, value) in &report.fields {
        println!("{key}: {value}");
    }
    Ok(())
}
