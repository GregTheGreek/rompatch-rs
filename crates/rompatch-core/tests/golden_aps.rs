use rompatch_core::format::aps;

const GBA_BLOCK: usize = 0x0001_0000;

fn build_aps_gba(source_size: u32, target_size: u32, records: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut p = b"APS1".to_vec();
    p.extend_from_slice(&source_size.to_le_bytes());
    p.extend_from_slice(&target_size.to_le_bytes());
    for (offset, xor) in records {
        assert_eq!(xor.len(), GBA_BLOCK, "GBA records are fixed 64KiB");
        p.extend_from_slice(&offset.to_le_bytes());
        // Source/target CRC-16 - we don't verify them, so put zeros.
        p.extend_from_slice(&0u16.to_le_bytes());
        p.extend_from_slice(&0u16.to_le_bytes());
        p.extend_from_slice(xor);
    }
    p
}

#[test]
fn aps_gba_applies_xor_block() {
    let mut rom = vec![0u8; GBA_BLOCK];
    rom[0..4].copy_from_slice(&[0x10, 0x20, 0x30, 0x40]);
    let mut xor = vec![0u8; GBA_BLOCK];
    xor[0..4].copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);

    let patch = build_aps_gba(GBA_BLOCK as u32, GBA_BLOCK as u32, &[(0, xor)]);
    let out = aps::apply_gba(&patch, &rom).unwrap();
    assert_eq!(
        &out[0..4],
        &[0x10 ^ 0x11, 0x20 ^ 0x22, 0x30 ^ 0x33, 0x40 ^ 0x44]
    );
    assert_eq!(out.len(), GBA_BLOCK);
}

#[test]
fn aps_gba_rejects_wrong_source_size() {
    let xor = vec![0u8; GBA_BLOCK];
    let patch = build_aps_gba(GBA_BLOCK as u32, GBA_BLOCK as u32, &[(0, xor)]);
    let rom = vec![0u8; GBA_BLOCK + 1];
    assert!(aps::apply_gba(&patch, &rom).is_err());
}

#[test]
fn aps_gba_rejects_truncated_record() {
    let mut p = b"APS1".to_vec();
    p.extend_from_slice(&(GBA_BLOCK as u32).to_le_bytes());
    p.extend_from_slice(&(GBA_BLOCK as u32).to_le_bytes());
    // Record header but truncated body.
    p.extend_from_slice(&0u32.to_le_bytes());
    p.extend_from_slice(&0u16.to_le_bytes());
    p.extend_from_slice(&0u16.to_le_bytes());
    p.extend_from_slice(&[0u8; 100]);
    let rom = vec![0u8; GBA_BLOCK];
    assert!(aps::apply_gba(&p, &rom).is_err());
}

fn build_aps_n64_simple(target_size: u32, records: &[u8]) -> Vec<u8> {
    let mut p = b"APS10".to_vec();
    p.push(0x00); // header type: simple
    p.push(0x00); // encoding method
    p.extend_from_slice(&[b' '; 50]); // description
    p.extend_from_slice(&target_size.to_le_bytes());
    p.extend_from_slice(records);
    p
}

#[test]
fn aps_n64_simple_applies_data_record() {
    let mut records = Vec::new();
    records.extend_from_slice(&5u32.to_le_bytes()); // offset
    records.push(3); // length
    records.extend_from_slice(&[0xAA, 0xBB, 0xCC]);
    let patch = build_aps_n64_simple(16, &records);
    let rom = vec![0u8; 16];

    let out = aps::apply_n64(&patch, &rom).unwrap();
    assert_eq!(out.len(), 16);
    assert_eq!(&out[5..8], &[0xAA, 0xBB, 0xCC]);
}

#[test]
fn aps_n64_simple_applies_rle_record() {
    let mut records = Vec::new();
    records.extend_from_slice(&2u32.to_le_bytes()); // offset
    records.push(0); // RLE flag
    records.push(0xFF); // byte
    records.push(5); // run length
    let patch = build_aps_n64_simple(16, &records);
    let rom = vec![0u8; 16];

    let out = aps::apply_n64(&patch, &rom).unwrap();
    assert_eq!(&out[2..7], &[0xFF; 5]);
    assert_eq!(out[1], 0);
    assert_eq!(out[7], 0);
}

#[test]
fn aps_n64_rejects_invalid_magic() {
    let p = b"XXXXX\x00\x00".to_vec();
    let rom = vec![0u8; 4];
    assert!(aps::apply_n64(&p, &rom).is_err());
}
