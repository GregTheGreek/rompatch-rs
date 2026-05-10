#![no_main]
use libfuzzer_sys::fuzz_target;
use rompatch_core::format::ppf;

fuzz_target!(|data: &[u8]| {
    let rom = vec![0u8; 256];
    let _ = ppf::apply(data, &rom);
});
