pub mod apply;
pub mod bin_file;
pub mod checksum_fix;
pub mod error;
pub mod format;
pub mod hash;
pub mod header;
pub mod info;

pub use apply::{ApplyError, ApplyOptions, ApplyOutcome, HashAlgo, HashCheckKind, HashSpec};
pub use checksum_fix::ChecksumFamily;
pub use error::{PatchError, Result};
pub use format::FormatKind;
pub use header::HeaderKind;

/// Upper bound on a declared output ROM size, in bytes. Any header that asks
/// for a larger allocation is rejected with [`PatchError::OutputTooLarge`].
///
/// Set well above the largest plausible cartridge ROM (64 MB N64) so real
/// patches are unaffected, while cheaply rejecting bogus or adversarial
/// headers that would otherwise trigger a multi-GB allocation.
pub const MAX_PATCH_OUTPUT_SIZE: usize = 256 * 1024 * 1024;
