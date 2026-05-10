use rompatch_core::format::bdf;

fn case(name: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let patch = std::fs::read(dir.join(format!("bdf_{name}.patch"))).unwrap();
    let src = std::fs::read(dir.join(format!("bdf_{name}.src"))).unwrap();
    let tgt = std::fs::read(dir.join(format!("bdf_{name}.tgt"))).unwrap();
    (patch, src, tgt)
}

#[test]
fn applies_identity_patch() {
    let (patch, src, tgt) = case("identity");
    assert_eq!(bdf::apply(&patch, &src).unwrap(), tgt);
}

#[test]
fn applies_byte_flip() {
    let (patch, src, tgt) = case("byteflip");
    assert_eq!(bdf::apply(&patch, &src).unwrap(), tgt);
}

#[test]
fn applies_growth_with_extra_block() {
    let (patch, src, tgt) = case("growth");
    assert_eq!(bdf::apply(&patch, &src).unwrap(), tgt);
}

#[test]
fn applies_negative_source_seek() {
    let (patch, src, tgt) = case("seek");
    assert_eq!(bdf::apply(&patch, &src).unwrap(), tgt);
}

#[test]
fn rejects_invalid_magic() {
    let mut p = b"NOTABDF1".to_vec();
    p.extend_from_slice(&[0u8; 24]);
    let rom = vec![0u8; 4];
    assert!(bdf::apply(&p, &rom).is_err());
}

#[test]
fn rejects_truncated_header() {
    let p = b"BSDIFF40".to_vec();
    let rom = vec![0u8; 4];
    assert!(bdf::apply(&p, &rom).is_err());
}
