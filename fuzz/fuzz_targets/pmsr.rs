#![no_main]
use libfuzzer_sys::fuzz_target;
use rompatch_core::format::pmsr;

// pmsr::apply rejects any ROM that isn't the 40 MiB Paper Mario USA1.0 image
// before the body is ever parsed, which would skip the parser entirely. Fuzz
// the apply_records helper instead so the record loop is actually exercised.
fuzz_target!(|data: &[u8]| {
    let rom = vec![0u8; 256];
    let _ = pmsr::apply_records(data, &rom);
});
