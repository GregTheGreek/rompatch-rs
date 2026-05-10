# rompatch-rs

Pure-Rust CLI for applying ROM patches. A from-scratch rewrite of the apply
path of [marcrobledo/RomPatcher.js](https://github.com/marcrobledo/RomPatcher.js),
focused on safety, minimal dependencies, and high test coverage.

This crate **applies** patches only. It does not create them.

## Supported formats

| Format  | Magic       | Notes                                                        |
| ------- | ----------- | ------------------------------------------------------------ |
| IPS     | `PATCH`     | Classic offset+data, optional RLE, `EOF` terminator          |
| UPS     | `UPS1`      | byuu XOR delta, VLV-encoded, CRC32 of source/target/patch    |
| BPS     | `BPS1`      | byuu copy/insert interpreter, VLV-encoded, CRC32 verified    |
| PMSR    | `PMSR`      | Paper Mario Star Rod; CRC32-verified record list             |
| APS-GBA | `APS1`      | Linear 64 KiB block records                                  |
| APS-N64 | `APS10`     | N64 cart-id + length record list                             |
| PPF     | `PPF`       | v1 / v2 / v3 (BIN/GI image, optional block-check + undo)     |
| RUP     | `NINJA2`    | NINJA-2 sequential XOR, MD5-verified                         |
| BDF     | `BSDIFF40`  | bsdiff with bzip2-compressed control/diff/extra blocks       |

VCDIFF/xdelta and ZIP-input are intentionally deferred from v1; see
[`crates/rompatch-core/src/format/README.md`](crates/rompatch-core/src/format/README.md)
for context.

## Not in v1 / roadmap

Items deliberately left out of the v1 cut, with the reason. Open to PRs.

| Item                   | Status     | Why                                                                                                   |
| ---------------------- | ---------- | ----------------------------------------------------------------------------------------------------- |
| VCDIFF / xdelta        | Deferred   | RomPatcher.js does not ship vendorable test fixtures; without `xdelta3`-cross-checked goldens we cannot verify our parser. Users with `.xdelta` patches can fall back to the upstream `xdelta3` tool. |
| ZIP-input              | Deferred   | Quality-of-life only - lets `rompatch apply rom.zip patch.bps` work without an `unzip` step first. No format unlock.                                                                                  |
| Patch creation         | Out of scope | This crate is apply-only by design. Match-finding (BPS), delta search (VCDIFF), and encoder logic are an order of magnitude more work than the apply path and are well-served by existing tools.    |
| Web / WASM / Node      | Out of scope | RomPatcher.js already covers the browser/Node side. This rewrite targets the CLI.                                                                                                                   |
| GUI                    | Out of scope | CLI only.                                                                                                                                                                                              |
| EBP                    | Not applicable | EBP exists in CoilSnake's Python `EBPatcher` but is not part of marcrobledo/RomPatcher.js (the reference we mirror).                                                                                |

Things that would make a good follow-up:

- Tight per-format upper bounds on declared output size (today we use a single 256 MiB cap).
- ZIP-input support if there's user demand.
- Coverage gate in CI with a real measured threshold.
- A real-ROM end-to-end smoke test (run manually / locally for licensing reasons).

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
