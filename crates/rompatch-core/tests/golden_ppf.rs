use rompatch_core::format::ppf;

fn build_ppf_v1(records: &[(u32, &[u8])]) -> Vec<u8> {
    let mut p = b"PPF".to_vec();
    p.extend_from_slice(b"10");
    p.push(0);
    p.extend_from_slice(&[b' '; 50]);
    for (offset, data) in records {
        p.extend_from_slice(&offset.to_le_bytes());
        p.push(data.len() as u8);
        p.extend_from_slice(data);
    }
    p
}

fn build_ppf_v3(undo_data: bool, records: &[(u64, &[u8])]) -> Vec<u8> {
    let mut p = b"PPF".to_vec();
    p.extend_from_slice(b"30");
    p.push(2);
    p.extend_from_slice(&[b' '; 50]);
    p.push(0); // image type
    p.push(0); // block check
    p.push(u8::from(undo_data));
    p.push(0); // dummy
    for (offset, data) in records {
        p.extend_from_slice(&offset.to_le_bytes());
        p.push(data.len() as u8);
        p.extend_from_slice(data);
        if undo_data {
            p.extend_from_slice(&vec![0u8; data.len()]);
        }
    }
    p
}

#[test]
fn ppf_v1_applies_records() {
    let rom = vec![0u8; 16];
    let patch = build_ppf_v1(&[(4, &[0x11, 0x22, 0x33])]);
    let out = ppf::apply(&patch, &rom).unwrap();
    assert_eq!(&out[4..7], &[0x11, 0x22, 0x33]);
}

#[test]
fn ppf_v3_applies_records_with_undo_data() {
    let rom = vec![0u8; 16];
    let patch = build_ppf_v3(true, &[(2, &[0xAA, 0xBB])]);
    let out = ppf::apply(&patch, &rom).unwrap();
    assert_eq!(&out[2..4], &[0xAA, 0xBB]);
}

#[test]
fn ppf_v3_handles_64bit_offsets() {
    let rom = vec![0u8; 8];
    let patch = build_ppf_v3(false, &[(16u64, &[0xFF])]);
    let out = ppf::apply(&patch, &rom).unwrap();
    assert_eq!(out.len(), 17);
    assert_eq!(out[16], 0xFF);
}

#[test]
fn ppf_stops_at_file_id_marker() {
    let mut p = build_ppf_v1(&[(0, &[0x42])]);
    p.extend_from_slice(b"@BEGIN_FILE_ID.DIZ");
    p.extend_from_slice(b"trailing junk");
    let rom = vec![0u8; 4];
    let out = ppf::apply(&p, &rom).unwrap();
    assert_eq!(out[0], 0x42);
}

#[test]
fn ppf_rejects_invalid_magic() {
    let p = b"XYZ10\x00".to_vec();
    let rom = vec![0u8; 4];
    assert!(ppf::apply(&p, &rom).is_err());
}

#[test]
fn ppf_rejects_unknown_version() {
    let mut p = b"PPF".to_vec();
    p.extend_from_slice(b"99");
    p.push(8);
    p.extend_from_slice(&[b' '; 50]);
    let rom = vec![0u8; 4];
    assert!(ppf::apply(&p, &rom).is_err());
}
