use proptest::prelude::*;
use rompatch_core::format::{aps, bdf, bps, ips, pmsr, ppf, rup, ups};

proptest! {
    #[test]
    fn ips_no_panic(patch in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let rom = vec![0u8; 256];
        let _ = ips::apply(&patch, &rom);
    }

    #[test]
    fn ups_no_panic(patch in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let rom = vec![0u8; 256];
        let _ = ups::apply(&patch, &rom);
    }

    #[test]
    fn bps_no_panic(patch in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let rom = vec![0u8; 256];
        let _ = bps::apply(&patch, &rom);
    }

    #[test]
    fn pmsr_no_panic(patch in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let rom = vec![0u8; 256];
        let _ = pmsr::apply_records(&patch, &rom);
    }

    #[test]
    fn aps_gba_no_panic(patch in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let rom = vec![0u8; 256];
        let _ = aps::apply_gba(&patch, &rom);
    }

    #[test]
    fn aps_n64_no_panic(patch in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let rom = vec![0u8; 256];
        let _ = aps::apply_n64(&patch, &rom);
    }

    #[test]
    fn ppf_no_panic(patch in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let rom = vec![0u8; 256];
        let _ = ppf::apply(&patch, &rom);
    }

    #[test]
    fn rup_no_panic(patch in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let rom = vec![0u8; 256];
        let _ = rup::apply(&patch, &rom);
    }

    #[test]
    fn bdf_no_panic(patch in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let rom = vec![0u8; 256];
        let _ = bdf::apply(&patch, &rom);
    }

    #[test]
    fn ips_no_panic_truncated(
        patch in proptest::collection::vec(any::<u8>(), 0..512),
        n in 0usize..512
    ) {
        let rom = vec![0u8; 64];
        let cut = n.min(patch.len());
        let _ = ips::apply(&patch[..cut], &rom);
    }

    #[test]
    fn bps_no_panic_truncated(
        patch in proptest::collection::vec(any::<u8>(), 0..512),
        n in 0usize..512
    ) {
        let rom = vec![0u8; 64];
        let cut = n.min(patch.len());
        let _ = bps::apply(&patch[..cut], &rom);
    }
}
