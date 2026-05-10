# Patch formats

One module per supported format. Each exposes a single entry point:

```rust
pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>>;
```

Detection lives in [`mod.rs`](mod.rs) and probes magic bytes in an order that
disambiguates overlapping prefixes (`APS10` must be checked before `APS1`).

## Implemented

| Module    | Magic       | Notes                                                                        |
| --------- | ----------- | ---------------------------------------------------------------------------- |
| `ips.rs`  | `PATCH`     | 24-bit BE offsets, 16-bit BE size, optional RLE (size=0), `EOF` terminator   |
| `ups.rs`  | `UPS1`      | byuu VLV header (input/output size), XOR delta body, three CRC32s in footer  |
| `bps.rs`  | `BPS1`      | byuu VLV header, action stream (SourceRead/TargetRead/SourceCopy/TargetCopy), three CRC32s in footer |
| `pmsr.rs` | `PMSR`      | Paper Mario Star Rod; record list + CRC32 ROM check                          |
| `aps.rs`  | `APS1`/`APS10` | Two variants: GBA (12-byte header + 64 KiB XOR records) and N64 (variable header + offset+length records). See module header doc for details. |
| `ppf.rs`  | `PPF`       | v1 / v2 / v3. v3 carries 64-bit offsets, BIN/GI image-type byte, optional block check + undo data |
| `rup.rs`  | `NINJA2`    | NINJA-2 sequential XOR delta with custom VLV; MD5 integrity                  |
| `bdf.rs`  | `BSDIFF40`  | bsdiff: 32-byte header with three signed 8-byte size fields, three concatenated bzip2 streams (control / diff / extra) |

## Deferred

- **VCDIFF/xdelta** - deferred entirely from v1. RomPatcher.js does not ship
  vendorable test fixtures (their harness needs a copyrighted DS ROM plus a
  romhacking.net patch). Without `xdelta3`-cross-checked fixtures we cannot
  verify our parser against anything but our own reading of RFC 3284, which
  is not enough confidence to ship. Users with `.xdelta` patches fall back
  to the upstream `xdelta3` tool.

- **EBP** - listed in the original plan in error. EBP exists in CoilSnake's
  Python `EBPatcher` but does not exist in marcrobledo/RomPatcher.js (the
  reference we follow). Not in scope.

- **ZIP input** - deferred. Pure quality-of-life so that
  `rompatch apply rom.zip patch.bps` would work directly. No format unlock,
  just saves the user one `unzip` call.

## Shared building blocks

All formats share:

- `BinReader` from [`bin_file`](../bin_file.rs) for cursor-based reads,
  including `read_vlv` for byuu's variable-length encoding (UPS/BPS/RUP).
- The `hash` module for CRC32 (UPS/BPS/PMSR), MD5 (RUP), Adler32 (reserved),
  and SHA-1 (CLI verify).
- The `PatchError` enum from [`error`](../error.rs) for all parse and
  integrity failures.

## Adding a new format

1. Add a `format/<name>.rs` module exposing
   `pub fn apply(patch: &[u8], rom: &[u8]) -> Result<Vec<u8>>`.
2. Declare it in [`mod.rs`](mod.rs) and add a `FormatKind` variant +
   `name()` mapping.
3. Extend `detect()` to recognize the magic; place the probe so that no
   later probe can shadow it.
4. Wire the kind into `apply()` dispatch.
5. Wire it into the CLI: `commands/apply.rs::parse_format` plus the
   `--format` doc string in the help text.
6. If the format has a useful header, extend `info::describe` to print it.
7. Add a fuzz target under [`fuzz/fuzz_targets/`](../../../../fuzz/fuzz_targets).
