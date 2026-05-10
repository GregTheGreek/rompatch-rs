use rompatch_core::format::pmsr;

fn build_pmsr(records: &[(u32, &[u8])]) -> Vec<u8> {
    let mut p = b"PMSR".to_vec();
    p.extend_from_slice(&(records.len() as u32).to_be_bytes());
    for (offset, data) in records {
        p.extend_from_slice(&offset.to_be_bytes());
        p.extend_from_slice(&(data.len() as u32).to_be_bytes());
        p.extend_from_slice(data);
    }
    p
}

#[test]
fn applies_records_via_apply_records_helper() {
    let rom = vec![0u8; 32];
    let mut expected = rom.clone();
    expected[4..8].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
    expected[20..22].copy_from_slice(&[0xCA, 0xFE]);

    let patch = build_pmsr(&[(4, &[0xDE, 0xAD, 0xBE, 0xEF]), (20, &[0xCA, 0xFE])]);
    assert_eq!(pmsr::apply_records(&patch, &rom).unwrap(), expected);
}

#[test]
fn extends_output_when_record_overruns() {
    let rom = vec![0u8; 4];
    let patch = build_pmsr(&[(8, &[0x01, 0x02])]);
    let out = pmsr::apply_records(&patch, &rom).unwrap();
    assert_eq!(out.len(), 10);
    assert_eq!(&out[8..], &[0x01, 0x02]);
}

#[test]
fn rejects_invalid_magic() {
    let p = b"XXXX\x00\x00\x00\x00".to_vec();
    let rom = vec![0u8; 4];
    assert!(pmsr::apply_records(&p, &rom).is_err());
}

#[test]
fn public_apply_rejects_wrong_size() {
    // Without the actual 40 MiB Paper Mario ROM, the public apply
    // must reject any input on size or CRC.
    let rom = vec![0u8; 16];
    let patch = build_pmsr(&[]);
    assert!(pmsr::apply(&patch, &rom).is_err());
}
