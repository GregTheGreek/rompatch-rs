use rompatch_core::{format::ups, hash};

fn write_vlv(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte | 0x80);
            return;
        }
        out.push(byte);
        value -= 1;
    }
}

fn build_ups(input_size: u64, output_size: u64, body: &[u8], src: &[u8], dst: &[u8]) -> Vec<u8> {
    let mut p = b"UPS1".to_vec();
    write_vlv(&mut p, input_size);
    write_vlv(&mut p, output_size);
    p.extend_from_slice(body);
    p.extend_from_slice(&hash::crc32(src).to_le_bytes());
    p.extend_from_slice(&hash::crc32(dst).to_le_bytes());
    let patch_crc = hash::crc32(&p);
    p.extend_from_slice(&patch_crc.to_le_bytes());
    p
}

#[test]
fn applies_single_byte_change() {
    let src = vec![0u8; 8];
    let mut dst = vec![0u8; 8];
    dst[4] = 0xAA;

    let mut body = Vec::new();
    write_vlv(&mut body, 4);
    body.push(0xAA);
    body.push(0x00);

    let patch = build_ups(8, 8, &body, &src, &dst);
    assert_eq!(ups::apply(&patch, &src).unwrap(), dst);
}

#[test]
fn applies_growth() {
    let src = vec![0u8; 4];
    let mut dst = vec![0u8; 8];
    dst[6] = 0x42;

    let mut body = Vec::new();
    write_vlv(&mut body, 6);
    body.push(0x42);
    body.push(0x00);

    let patch = build_ups(4, 8, &body, &src, &dst);
    assert_eq!(ups::apply(&patch, &src).unwrap(), dst);
}

#[test]
fn applies_multibyte_xor_run() {
    let src = vec![0xAAu8; 8];
    let mut dst = src.clone();
    dst[2] = 0x11;
    dst[3] = 0x22;
    dst[4] = 0x33;

    let mut body = Vec::new();
    write_vlv(&mut body, 2);
    body.push(0xAA ^ 0x11);
    body.push(0xAA ^ 0x22);
    body.push(0xAA ^ 0x33);
    body.push(0x00);

    let patch = build_ups(8, 8, &body, &src, &dst);
    assert_eq!(ups::apply(&patch, &src).unwrap(), dst);
}

#[test]
fn rejects_wrong_input_size() {
    let src = vec![0u8; 4];
    let dst = vec![0u8; 4];
    let body: Vec<u8> = Vec::new();
    let patch = build_ups(4, 4, &body, &src, &dst);
    let wrong_src = vec![0u8; 5];
    assert!(ups::apply(&patch, &wrong_src).is_err());
}

#[test]
fn rejects_input_crc_mismatch() {
    let src = vec![0u8; 4];
    let dst = vec![1u8; 4];
    let mut body = Vec::new();
    write_vlv(&mut body, 0);
    body.extend_from_slice(&[0x01, 0x01, 0x01, 0x01]);
    body.push(0x00);
    let patch = build_ups(4, 4, &body, &src, &dst);
    let mutated_src = vec![0xFFu8; 4];
    assert!(ups::apply(&patch, &mutated_src).is_err());
}

#[test]
fn rejects_invalid_magic() {
    let src = vec![0u8; 4];
    let mut p = b"NOTM".to_vec();
    p.extend_from_slice(&[0u8; 12]);
    assert!(ups::apply(&p, &src).is_err());
}

#[test]
fn rejects_truncated_patch() {
    let src = vec![0u8; 4];
    let p = b"UPS1".to_vec();
    assert!(ups::apply(&p, &src).is_err());
}
