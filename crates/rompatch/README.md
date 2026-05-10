# rompatch

The CLI binary. Thin shell over [`rompatch-core`](../rompatch-core/README.md).

```bash
cargo build --release -p rompatch
./target/release/rompatch <COMMAND> [args]
```

## Subcommands

### `apply` - apply a patch

```
rompatch apply <ROM> <PATCH> [OPTIONS]
```

Detects the patch format from its magic bytes, applies it, and writes the
patched ROM. Default output is `<rom-stem>.patched.<rom-ext>` next to the
input.

| Flag                          | Effect                                                                          |
| ----------------------------- | ------------------------------------------------------------------------------- |
| `-o, --output <PATH>`         | Write to `<PATH>` instead of the default                                        |
| `--format <NAME>`             | Override autodetect; one of `ips`, `ups`, `bps`, `pmsr`, `aps-gba`, `aps-n64`, `ppf`, `rup`, `bdf` (case-insensitive; `bsdiff` is an alias for `bdf`) |
| `--strip-header`              | Detect and strip an SMC/iNES/FDS/LYNX header before patching; reattached on write |
| `--fix-checksum`              | Recompute Game Boy or Mega Drive cartridge checksum after patching              |
| `--verify-input  <ALGO:HEX>`  | Check input ROM hash before patching                                            |
| `--verify-output <ALGO:HEX>`  | Check output ROM hash after patching                                            |

`<ALGO>` is one of `crc32`, `md5`, `sha1`, `adler32`. `<HEX>` is the expected
digest as lowercase hex.

Examples:

```bash
# Simplest case: autodetect + default output path
rompatch apply rom.gba hack.bps

# Write to a specific path, then verify the result hash
rompatch apply rom.smc hack.ups -o patched.smc \
  --verify-output sha1:da39a3ee5e6b4b0d3255bfef95601890afd80709

# Headered SNES ROM + checksum fix
rompatch apply headered.smc hack.ips --strip-header --fix-checksum

# Force a format when autodetect would be ambiguous or wrong
rompatch apply rom.gba weird.bin --format ips
```

### `detect` - print the detected format

```
rompatch detect <PATCH>
```

Reads the magic bytes and prints the format name (e.g. `IPS`, `BPS`, `BDF`).
Exits non-zero if no known format matches.

### `info` - dump patch metadata

```
rompatch info <PATCH>
```

Prints the format and patch size, then format-specific header fields:
declared input/output sizes, embedded CRC32 / MD5 expectations, PPF
description string, RUP author/title/version, BDF block sizes, etc.

Block bodies are never decompressed; this is a header-only inspector.

### `hash` - print a file hash

```
rompatch hash <FILE> [--algo <ALGO>]
```

`<ALGO>` defaults to `crc32`. Other choices: `md5`, `sha1`, `adler32`.

```bash
rompatch hash rom.gba
rompatch hash rom.gba --algo sha1
```

## Exit codes

| Code | Meaning                                                   |
| ---- | --------------------------------------------------------- |
| 0    | Success                                                   |
| 1    | Bad CLI usage (unknown subcommand, missing arg)           |
| 2    | Patch error (corrupt, unknown format, hash mismatch)      |
| 3    | I/O error (file missing, permission denied)               |

## Dependencies

- `rompatch-core` - all real work
- `lexopt` - tiny zero-dep arg parser

## License

Dual-licensed MIT or Apache-2.0.
