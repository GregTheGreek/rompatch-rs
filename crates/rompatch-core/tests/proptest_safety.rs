use proptest::prelude::*;
use rompatch_core::format::{ips, ups};

proptest! {
    #[test]
    fn ips_no_panic_on_arbitrary_bytes(patch in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let rom = vec![0u8; 256];
        let _ = ips::apply(&patch, &rom);
    }

    #[test]
    fn ups_no_panic_on_arbitrary_bytes(patch in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let rom = vec![0u8; 256];
        let _ = ups::apply(&patch, &rom);
    }

    #[test]
    fn ips_no_panic_on_truncation(
        patch in proptest::collection::vec(any::<u8>(), 0..512),
        n in 0usize..512
    ) {
        let rom = vec![0u8; 64];
        let cut = n.min(patch.len());
        let _ = ips::apply(&patch[..cut], &rom);
    }
}
