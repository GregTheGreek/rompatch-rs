# rompatch-rs

Pure-Rust CLI for applying ROM patches. A from-scratch rewrite of the apply
path of [marcrobledo/RomPatcher.js](https://github.com/marcrobledo/RomPatcher.js),
focused on safety, minimal dependencies, and high test coverage.

This crate **applies** patches only. It does not create them.

## Supported formats

| Format  | Magic       | Typical targets                            | Notes                                                        |
| ------- | ----------- | ------------------------------------------ | ------------------------------------------------------------ |
| IPS     | `PATCH`     | NES, SNES, Game Boy, GBA, Genesis          | Classic offset+data, optional RLE, `EOF` terminator          |
| UPS     | `UPS1`      | SNES, GBA, DS                              | byuu XOR delta, VLV-encoded, CRC32 of source/target/patch    |
| BPS     | `BPS1`      | SNES, GBA, DS (modern byuu/bsnes hacks)    | byuu copy/insert interpreter, VLV-encoded, CRC32 verified    |
| PMSR    | `PMSR`      | N64 (Paper Mario Star Rod only)            | Paper Mario Star Rod; CRC32-verified record list             |
| APS-GBA | `APS1`      | Game Boy Advance                           | Linear 64 KiB block records                                  |
| APS-N64 | `APS10`     | Nintendo 64                                | N64 cart-id + length record list                             |
| PPF     | `PPF`       | PlayStation (PSX), Saturn, Dreamcast, PS2  | v1 / v2 / v3 (BIN/GI image, optional block-check + undo)     |
| RUP     | `NINJA2`    | Any (NINJA-2 multi-system container)       | NINJA-2 sequential XOR, MD5-verified                         |
| BDF     | `BSDIFF40`  | Any (generic binary diff)                  | bsdiff with bzip2-compressed control/diff/extra blocks       |

VCDIFF/xdelta and ZIP-input are intentionally deferred from v1; see
[`crates/rompatch-core/src/format/README.md`](crates/rompatch-core/src/format/README.md)
for context.

## Quickstart

```bash
cargo build --release
./target/release/rompatch apply path/to/rom.gba path/to/hack.bps -o patched.gba
```

Auto-detection picks the format from the patch's magic bytes; pass
`--format <name>` to override. See the CLI README below for all flags.

## Layout

```
.
├── crates/
│   ├── rompatch-core/   library: parsers, hashes, headers, info
│   └── rompatch/        CLI binary (apply/detect/info/hash)
└── fuzz/                cargo-fuzz harnesses (one per format + dispatch)
```

Per-directory documentation:

- [`crates/rompatch-core/README.md`](crates/rompatch-core/README.md) - library API reference
- [`crates/rompatch/README.md`](crates/rompatch/README.md) - CLI subcommands and flags
- [`crates/rompatch-core/src/format/README.md`](crates/rompatch-core/src/format/README.md) - wire format notes per patch type
- [`fuzz/README.md`](fuzz/README.md) - how to run the fuzz harnesses

## Development

```bash
cargo test --workspace --all-targets    # unit + integration
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cargo deny check                        # license + advisory + dup-dep audit
cargo llvm-cov --workspace              # coverage (requires cargo-llvm-cov)
```

CI runs all of the above on Linux and macOS, plus a 60-second fuzz smoke per
target on nightly. See `.github/workflows/ci.yml`.

## License

Dual-licensed under MIT or Apache-2.0 at your option.
