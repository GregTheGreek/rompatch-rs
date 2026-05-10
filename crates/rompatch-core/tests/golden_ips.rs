use rompatch_core::format::ips;

fn record_data(offset: u32, data: &[u8]) -> Vec<u8> {
    let mut r = Vec::new();
    r.extend_from_slice(&offset.to_be_bytes()[1..4]);
    let size = u16::try_from(data.len()).expect("data too large for IPS record");
    r.extend_from_slice(&size.to_be_bytes());
    r.extend_from_slice(data);
    r
}

fn record_rle(offset: u32, count: u16, fill: u8) -> Vec<u8> {
    let mut r = Vec::new();
    r.extend_from_slice(&offset.to_be_bytes()[1..4]);
    r.extend_from_slice(&[0u8, 0u8]);
    r.extend_from_slice(&count.to_be_bytes());
    r.push(fill);
    r
}

fn build_patch(records: &[Vec<u8>], truncate: Option<u32>) -> Vec<u8> {
    let mut out = b"PATCH".to_vec();
    for rec in records {
        out.extend_from_slice(rec);
    }
    out.extend_from_slice(b"EOF");
    if let Some(t) = truncate {
        out.extend_from_slice(&t.to_be_bytes()[1..4]);
    }
    out
}

#[test]
fn applies_simple_data_record() {
    let rom = vec![0u8; 16];
    let patch = build_patch(&[record_data(4, b"\xDE\xAD\xBE\xEF")], None);
    let got = ips::apply(&patch, &rom).unwrap();
    let mut want = vec![0u8; 16];
    want[4..8].copy_from_slice(b"\xDE\xAD\xBE\xEF");
    assert_eq!(got, want);
}

#[test]
fn applies_rle_record() {
    let rom = vec![0u8; 16];
    let patch = build_patch(&[record_rle(0, 8, 0xAA)], None);
    let got = ips::apply(&patch, &rom).unwrap();
    let mut want = vec![0u8; 16];
    want[..8].fill(0xAA);
    assert_eq!(got, want);
}

#[test]
fn extends_rom_when_offset_exceeds_length() {
    let rom = vec![0u8; 4];
    let patch = build_patch(&[record_data(8, b"\xFF\xEE")], None);
    let got = ips::apply(&patch, &rom).unwrap();
    let mut want = vec![0u8; 10];
    want[8] = 0xFF;
    want[9] = 0xEE;
    assert_eq!(got, want);
}

#[test]
fn applies_truncate_offset() {
    let rom = vec![0u8; 16];
    let patch = build_patch(&[], Some(8));
    let got = ips::apply(&patch, &rom).unwrap();
    assert_eq!(got, vec![0u8; 8]);
}

#[test]
fn rejects_invalid_magic() {
    let rom = vec![0u8; 16];
    let mut patch = b"NOTIPS".to_vec();
    patch.extend_from_slice(b"EOF");
    assert!(ips::apply(&patch, &rom).is_err());
}

#[test]
fn rejects_truncated_record_header() {
    let rom = vec![0u8; 16];
    let mut patch = b"PATCH".to_vec();
    patch.extend_from_slice(&[0, 0, 0]);
    assert!(ips::apply(&patch, &rom).is_err());
}

#[test]
fn empty_patch_returns_rom_unchanged() {
    let rom = vec![1, 2, 3, 4, 5];
    let patch = build_patch(&[], None);
    let got = ips::apply(&patch, &rom).unwrap();
    assert_eq!(got, rom);
}

#[test]
fn multiple_records_apply_in_order() {
    let rom = vec![0u8; 16];
    let patch = build_patch(
        &[
            record_data(0, b"\x01\x02"),
            record_data(4, b"\x03\x04"),
            record_rle(8, 4, 0xFF),
        ],
        None,
    );
    let got = ips::apply(&patch, &rom).unwrap();
    let mut want = vec![0u8; 16];
    want[0] = 1;
    want[1] = 2;
    want[4] = 3;
    want[5] = 4;
    want[8..12].fill(0xFF);
    assert_eq!(got, want);
}

#[test]
fn truncate_offset_can_grow_output() {
    let rom = vec![1u8; 4];
    let patch = build_patch(&[], Some(8));
    let got = ips::apply(&patch, &rom).unwrap();
    assert_eq!(got.len(), 8);
    assert_eq!(&got[..4], &[1, 1, 1, 1]);
    assert_eq!(&got[4..], &[0, 0, 0, 0]);
}
