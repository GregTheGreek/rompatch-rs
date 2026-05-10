#![no_main]
use libfuzzer_sys::fuzz_target;
use rompatch_core::format::bdf;

fuzz_target!(|data: &[u8]| {
    let rom = vec![0u8; 256];
    let _ = bdf::apply(data, &rom);
});
