use rompatch_core::{format::rup, hash};

fn write_rup_vlv(out: &mut Vec<u8>, value: u64) {
    let mut bytes = Vec::new();
    let mut v = value;
    while v != 0 {
        bytes.push((v & 0xFF) as u8);
        v >>= 8;
    }
    out.push(bytes.len() as u8);
    out.extend_from_slice(&bytes);
}

fn build_rup_single(rom: &[u8], target: &[u8], xor_records: &[(u64, Vec<u8>)]) -> Vec<u8> {
    let mut p = b"NINJA2".to_vec();
    // pad to 0x800
    p.resize(0x800, 0);

    p.push(0x01); // OPEN_NEW_FILE
    write_rup_vlv(&mut p, 4); // filename length
    p.extend_from_slice(b"rom\0");
    p.push(0); // rom type
    write_rup_vlv(&mut p, rom.len() as u64);
    write_rup_vlv(&mut p, target.len() as u64);
    p.extend_from_slice(&hash::md5(rom));
    p.extend_from_slice(&hash::md5(target));

    if rom.len() != target.len() {
        // Mode 'A' assumed (target larger). overflow is bytes from
        // source.len()..target.len() XOR 0xFF.
        assert!(target.len() > rom.len());
        let overflow: Vec<u8> = target[rom.len()..].iter().map(|b| b ^ 0xFF).collect();
        p.push(b'A');
        write_rup_vlv(&mut p, overflow.len() as u64);
        p.extend_from_slice(&overflow);
    }

    for (offset, xor) in xor_records {
        p.push(0x02); // XOR_RECORD
        write_rup_vlv(&mut p, *offset);
        write_rup_vlv(&mut p, xor.len() as u64);
        p.extend_from_slice(xor);
    }
    p.push(0x00); // END
    p
}

#[test]
fn rup_applies_xor_record_same_size() {
    let rom = vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80];
    let mut target = rom.clone();
    target[2] = 0xAA;
    target[3] = 0xBB;

    let xor: Vec<u8> = vec![0x30 ^ 0xAA, 0x40 ^ 0xBB];
    let patch = build_rup_single(&rom, &target, &[(2, xor)]);

    assert_eq!(rup::apply(&patch, &rom).unwrap(), target);
}

#[test]
fn rup_grows_output_with_overflow() {
    let rom = vec![0u8; 4];
    let target = vec![0u8, 0, 0, 0, 0xCA, 0xFE];
    let patch = build_rup_single(&rom, &target, &[]);
    assert_eq!(rup::apply(&patch, &rom).unwrap(), target);
}

#[test]
fn rup_rejects_no_matching_file() {
    let real_rom = vec![0u8; 4];
    let other_rom = vec![1u8; 4];
    let target = vec![0u8; 4];
    let patch = build_rup_single(&real_rom, &target, &[]);
    assert!(rup::apply(&patch, &other_rom).is_err());
}

#[test]
fn rup_rejects_invalid_magic() {
    let mut p = b"WRONG!".to_vec();
    p.resize(0x800, 0);
    let rom = vec![0u8; 4];
    assert!(rup::apply(&p, &rom).is_err());
}
