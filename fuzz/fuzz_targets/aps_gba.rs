#![no_main]
use libfuzzer_sys::fuzz_target;
use rompatch_core::format::aps;

fuzz_target!(|data: &[u8]| {
    let rom = vec![0u8; 256];
    let _ = aps::apply_gba(data, &rom);
});
