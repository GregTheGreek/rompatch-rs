# Fuzz harnesses

`cargo-fuzz` targets, one per patch format plus a `dispatch` target that
exercises the magic-byte detection path. Each target hands arbitrary bytes
to the relevant `apply()` and asserts only that we don't panic.

This crate is intentionally **excluded from the workspace** (`exclude =
["fuzz"]` in the root `Cargo.toml`) because `libfuzzer-sys` requires a
nightly toolchain.

## Prerequisites

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
```

## Running a target

```bash
cargo +nightly fuzz run ips                        # fuzz forever
cargo +nightly fuzz run bps -- -max_total_time=60  # 60-second smoke
```

Targets:

```
ips    ups    bps    pmsr    aps_gba    aps_n64    ppf    rup    bdf    dispatch
```

## CI

`.github/workflows/ci.yml` runs every target for 60 seconds on each push. The
job is non-blocking but failures should be triaged - any panic from arbitrary
input bytes is a parser bug.

## Corpus and crashes

cargo-fuzz writes corpus seeds under `corpus/<target>/` and crashes under
`artifacts/<target>/`. Both directories are gitignored; reproduce a crash
locally with:

```bash
cargo +nightly fuzz run <target> artifacts/<target>/<crash-input>
```

## Adding a target

1. Create `fuzz/fuzz_targets/<name>.rs`:

   ```rust
   #![no_main]
   use libfuzzer_sys::fuzz_target;
   use rompatch_core::format::<name>;

   fuzz_target!(|data: &[u8]| {
       let rom = vec![0u8; 256];
       let _ = <name>::apply(data, &rom);
   });
   ```

2. Add a `[[bin]]` block in [`Cargo.toml`](Cargo.toml) pointing at the file.
3. Add the target name to the CI matrix in `.github/workflows/ci.yml`.
