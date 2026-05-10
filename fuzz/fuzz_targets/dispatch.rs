#![no_main]
use libfuzzer_sys::fuzz_target;
use rompatch_core::format;

// Top-level dispatch: any byte string is a candidate "patch", magic detection
// picks the format. Catches cross-format misrouting and detect() panics.
fuzz_target!(|data: &[u8]| {
    let rom = vec![0u8; 256];
    let _ = format::detect(data);
    let _ = format::apply(data, &rom);
});
