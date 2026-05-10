# rompatch-core

Library backing the `rompatch` CLI. All format parsers, hash wrappers, ROM
header detection, cartridge checksum fixes, and patch metadata extraction
live here. The CLI is a thin shell over this crate.

`#![forbid(unsafe_code)]` workspace-wide.

## Modules

| Module          | What it does                                                                  |
| --------------- | ----------------------------------------------------------------------------- |
| `bin_file`      | Cursor-based binary reader; shared `BinReader` plus byuu VLV decoder          |
| `format`        | One submodule per patch format with `apply(patch, rom) -> Result<Vec<u8>>`    |
| `hash`          | Thin wrappers: `crc32`, `md5`, `sha1`, `adler32`, plus `hex` formatter        |
| `header`        | ROM header detect/strip for SMC (SNES), iNES, FDS, LYNX                       |
| `checksum_fix`  | Recompute Game Boy header + global checksums, Mega Drive header checksum      |
| `info`          | Format-agnostic `describe(patch) -> PatchInfo` powering `rompatch info`       |
| `error`         | `PatchError` enum + `Result<T>` alias                                         |

## Top-level API

The crate root re-exports the most common types:

```rust
use rompatch_core::{FormatKind, HeaderKind, PatchError, Result};
```

### Applying a patch

`format::apply` autodetects the format by magic bytes and dispatches:

```rust
use rompatch_core::format;

let patched = format::apply(&patch_bytes, &rom_bytes)?;
```

To skip detection, call the format-specific function directly:

```rust
use rompatch_core::format::{bps, ips};

let out = ips::apply(&patch, &rom)?;
let out = bps::apply(&patch, &rom)?;
```

`format::detect(patch) -> Option<FormatKind>` returns the detected kind without
applying.

### Hashing

```rust
use rompatch_core::hash;

let crc = hash::crc32(bytes);
let digest = hash::md5(bytes);          // [u8; 16]
let digest = hash::sha1(bytes);         // [u8; 20]
let adler = hash::adler32(bytes);
let display = hash::hex(&digest);       // lowercase hex string
```

### ROM headers

```rust
use rompatch_core::header::{self, HeaderKind};

if let Some(kind) = header::detect(rom) {
    let (head, body) = header::split(rom, kind);
    // patch `body`, then concat `head` + patched body on write
}
```

`HeaderKind::header_size()` returns the fixed prefix length per system.

### Patch metadata

```rust
use rompatch_core::info;

let report = info::describe(&patch_bytes)?;
println!("{} ({} bytes)", report.format.name(), report.patch_size);
for (k, v) in &report.fields {
    println!("  {k}: {v}");
}
```

Header fields parsed only; block bodies are never decompressed by `describe`.

## Errors

All public APIs return `Result<T, PatchError>`. The variants distinguish:

- `Truncated` / `InvalidMagic` / `InvalidEncoding` (parse-time failures)
- `InputSizeMismatch`, `InputHashMismatch`, `OutputHashMismatch`,
  `PatchHashMismatch`, `InputMd5Mismatch`, `OutputMd5Mismatch`
  (integrity failures)
- `OffsetOutOfRange`, `NoMatchingFile`, `UnsupportedFeature(&'static str)`

`PatchError` implements `std::error::Error` and `Display`.

## Dependencies

Each crate is single-purpose and pure-Rust:

| Crate         | Purpose                                  |
| ------------- | ---------------------------------------- |
| `crc32fast`   | CRC32 used by IPS-truncate, BPS, UPS     |
| `md-5`        | RUP integrity hash, CLI verify           |
| `sha1`        | CLI verify                               |
| `adler`       | CLI verify (and reserved for VCDIFF)     |
| `bzip2-rs`    | BDF/bsdiff decompression (decode-only)   |
| `proptest`    | dev-only: property tests                 |

No `serde`, no FFI, no `-sys` crates.

## Testing

```bash
cargo test -p rompatch-core --all-targets
```

Tests live alongside source as `#[cfg(test)] mod tests`, plus integration
tests under `tests/` (golden + roundtrip).

## License

Dual-licensed MIT or Apache-2.0.
