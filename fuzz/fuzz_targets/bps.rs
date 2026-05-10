#![no_main]
use libfuzzer_sys::fuzz_target;
use rompatch_core::format::bps;

fuzz_target!(|data: &[u8]| {
    let rom = vec![0u8; 256];
    let _ = bps::apply(data, &rom);
});
