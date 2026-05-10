use rompatch_core::{
    format::{ips, ups},
    hash, info,
};

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

#[test]
fn describes_ups_header_and_footer() {
    let src = vec![0u8; 4];
    let dst = vec![0xAA, 0, 0, 0];
    let mut body = Vec::new();
    write_vlv(&mut body, 0);
    body.push(0xAA);
    body.push(0x00);

    let mut p = b"UPS1".to_vec();
    write_vlv(&mut p, 4);
    write_vlv(&mut p, 4);
    p.extend_from_slice(&body);
    p.extend_from_slice(&hash::crc32(&src).to_le_bytes());
    p.extend_from_slice(&hash::crc32(&dst).to_le_bytes());
    let patch_crc = hash::crc32(&p);
    p.extend_from_slice(&patch_crc.to_le_bytes());

    assert_eq!(ups::apply(&p, &src).unwrap(), dst);

    let report = info::describe(&p).unwrap();
    let map: std::collections::HashMap<_, _> = report.fields.into_iter().collect();
    assert_eq!(map["input size"], "4");
    assert_eq!(map["output size"], "4");
    assert_eq!(map["input CRC32"], format!("{:08x}", hash::crc32(&src)));
    assert_eq!(map["output CRC32"], format!("{:08x}", hash::crc32(&dst)));
}

#[test]
fn describes_ips_without_extra_fields() {
    let patch = b"PATCHEOF".to_vec();
    assert!(ips::apply(&patch, &[0u8; 4]).is_ok());
    let report = info::describe(&patch).unwrap();
    assert!(report.fields.is_empty());
}

#[test]
fn rejects_unknown_magic() {
    let p = b"XXXX".to_vec();
    assert!(info::describe(&p).is_err());
}
