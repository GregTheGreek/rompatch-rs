use rompatch_core::{format::bps, hash};

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

fn write_signed_vlv(out: &mut Vec<u8>, value: i64) {
    let raw = if value < 0 {
        ((-value) as u64) << 1 | 1
    } else {
        (value as u64) << 1
    };
    write_vlv(out, raw);
}

enum Action {
    SourceRead { length: u64 },
    TargetRead { bytes: Vec<u8> },
    SourceCopy { length: u64, delta: i64 },
    TargetCopy { length: u64, delta: i64 },
}

fn build_bps(source: &[u8], target: &[u8], metadata: &[u8], actions: &[Action]) -> Vec<u8> {
    let mut p = b"BPS1".to_vec();
    write_vlv(&mut p, source.len() as u64);
    write_vlv(&mut p, target.len() as u64);
    write_vlv(&mut p, metadata.len() as u64);
    p.extend_from_slice(metadata);

    for action in actions {
        match action {
            Action::SourceRead { length } => {
                write_vlv(&mut p, (length - 1) << 2);
            }
            Action::TargetRead { bytes } => {
                write_vlv(&mut p, ((bytes.len() as u64 - 1) << 2) | 1);
                p.extend_from_slice(bytes);
            }
            Action::SourceCopy { length, delta } => {
                write_vlv(&mut p, ((length - 1) << 2) | 2);
                write_signed_vlv(&mut p, *delta);
            }
            Action::TargetCopy { length, delta } => {
                write_vlv(&mut p, ((length - 1) << 2) | 3);
                write_signed_vlv(&mut p, *delta);
            }
        }
    }

    p.extend_from_slice(&hash::crc32(source).to_le_bytes());
    p.extend_from_slice(&hash::crc32(target).to_le_bytes());
    let patch_crc = hash::crc32(&p);
    p.extend_from_slice(&patch_crc.to_le_bytes());
    p
}

#[test]
fn applies_pure_source_read_identity() {
    let src: Vec<u8> = (0u8..16).collect();
    let dst = src.clone();
    let patch = build_bps(&src, &dst, b"", &[Action::SourceRead { length: 16 }]);
    assert_eq!(bps::apply(&patch, &src).unwrap(), dst);
}

#[test]
fn applies_pure_target_read_replaces_all() {
    let src = vec![0u8; 8];
    let dst: Vec<u8> = (1u8..=8).collect();
    let patch = build_bps(
        &src,
        &dst,
        b"",
        &[Action::TargetRead { bytes: dst.clone() }],
    );
    assert_eq!(bps::apply(&patch, &src).unwrap(), dst);
}

#[test]
fn applies_source_copy_with_negative_delta() {
    // Take src[4..8] and place it at output[0..4], then take src[0..4] for output[4..8].
    let src: Vec<u8> = (0u8..8).collect();
    let mut dst = vec![0u8; 8];
    dst[..4].copy_from_slice(&src[4..8]);
    dst[4..].copy_from_slice(&src[0..4]);

    let patch = build_bps(
        &src,
        &dst,
        b"",
        &[
            // source_relative_offset starts at 0; +4 -> 4; copy 4 bytes -> source_relative=8
            Action::SourceCopy {
                length: 4,
                delta: 4,
            },
            // 8 + delta = 0; delta = -8
            Action::SourceCopy {
                length: 4,
                delta: -8,
            },
        ],
    );
    assert_eq!(bps::apply(&patch, &src).unwrap(), dst);
}

#[test]
fn applies_target_copy_rle() {
    // Output: [0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA] from src zeros.
    // Strategy: TargetRead 1 byte (0xAA), then TargetCopy with delta=-1 length=7.
    let src = vec![0u8; 8];
    let dst = vec![0xAAu8; 8];

    let patch = build_bps(
        &src,
        &dst,
        b"",
        &[
            Action::TargetRead { bytes: vec![0xAA] },
            // target_relative_offset starts at 0; +(-1) wraps... wait we want it to become 0.
            // After the first action, output_offset = 1. We want target_relative_offset = 0.
            // 0 + delta = 0 -> delta = 0.
            Action::TargetCopy {
                length: 7,
                delta: 0,
            },
        ],
    );
    assert_eq!(bps::apply(&patch, &src).unwrap(), dst);
}

#[test]
fn applies_combined_actions() {
    // src = [0,1,2,3,4,5,6,7]
    // dst = [0,1,2,3, 0xCA, 0xFE, 1, 2]
    //   source_read 4 -> [0,1,2,3, _, _, _, _], output_offset=4
    //   target_read [0xCA, 0xFE] -> output_offset=6
    //   source_copy len=2, source_rel: 0 + delta = 1, copy src[1..3]=[1,2] -> output_offset=8
    let src: Vec<u8> = (0u8..8).collect();
    let dst = vec![0u8, 1, 2, 3, 0xCA, 0xFE, 1, 2];

    let patch = build_bps(
        &src,
        &dst,
        b"",
        &[
            Action::SourceRead { length: 4 },
            Action::TargetRead {
                bytes: vec![0xCA, 0xFE],
            },
            Action::SourceCopy {
                length: 2,
                delta: 1,
            },
        ],
    );
    assert_eq!(bps::apply(&patch, &src).unwrap(), dst);
}

#[test]
fn applies_with_metadata_skipped() {
    let src = vec![0u8; 4];
    let dst = vec![0u8; 4];
    let patch = build_bps(
        &src,
        &dst,
        b"hello world",
        &[Action::SourceRead { length: 4 }],
    );
    assert_eq!(bps::apply(&patch, &src).unwrap(), dst);
}

#[test]
fn rejects_wrong_source_size() {
    let src = vec![0u8; 4];
    let dst = vec![0u8; 4];
    let patch = build_bps(&src, &dst, b"", &[Action::SourceRead { length: 4 }]);
    let wrong = vec![0u8; 5];
    assert!(bps::apply(&patch, &wrong).is_err());
}

#[test]
fn rejects_source_crc_mismatch() {
    let src = vec![0u8; 4];
    let dst = vec![0u8; 4];
    let patch = build_bps(&src, &dst, b"", &[Action::SourceRead { length: 4 }]);
    let mutated = vec![0xFFu8; 4];
    assert!(bps::apply(&patch, &mutated).is_err());
}

#[test]
fn rejects_tampered_patch_crc() {
    let src = vec![0u8; 4];
    let dst = vec![0u8; 4];
    let mut patch = build_bps(&src, &dst, b"", &[Action::SourceRead { length: 4 }]);
    let last = patch.len() - 1;
    patch[last] ^= 0xFF;
    assert!(bps::apply(&patch, &src).is_err());
}

#[test]
fn rejects_invalid_magic() {
    let src = vec![0u8; 4];
    let mut p = b"NOTB".to_vec();
    p.extend_from_slice(&[0u8; 12]);
    assert!(bps::apply(&p, &src).is_err());
}

#[test]
fn rejects_truncated_patch() {
    let src = vec![0u8; 4];
    let p = b"BPS1".to_vec();
    assert!(bps::apply(&p, &src).is_err());
}

#[test]
fn rejects_source_copy_out_of_range() {
    // delta pushes source_relative_offset past source.len()
    let src = vec![0u8; 4];
    let dst = vec![0u8; 4];
    let patch = build_bps(
        &src,
        &dst,
        b"",
        &[Action::SourceCopy {
            length: 4,
            delta: 100,
        }],
    );
    assert!(bps::apply(&patch, &src).is_err());
}

#[test]
fn rejects_target_copy_reading_unwritten() {
    // target_relative_offset == output_offset is invalid (reading bytes not yet written).
    let src = vec![0u8; 4];
    let dst = vec![0u8; 4];
    // Write nothing first, then TargetCopy with delta=0 -> read output[0] which is the
    // very byte we're about to write.
    let patch = build_bps(
        &src,
        &dst,
        b"",
        &[Action::TargetCopy {
            length: 1,
            delta: 0,
        }],
    );
    assert!(bps::apply(&patch, &src).is_err());
}
