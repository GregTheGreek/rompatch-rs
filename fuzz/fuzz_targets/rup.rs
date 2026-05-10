#![no_main]
use libfuzzer_sys::fuzz_target;
use rompatch_core::format::rup;

fuzz_target!(|data: &[u8]| {
    let rom = vec![0u8; 256];
    let _ = rup::apply(data, &rom);
});
