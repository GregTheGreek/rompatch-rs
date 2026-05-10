use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchError {
    Truncated,
    InvalidMagic,
    InvalidEncoding,
    InputSizeMismatch {
        expected: u64,
        actual: u64,
    },
    InputHashMismatch {
        expected: u32,
        actual: u32,
    },
    OutputHashMismatch {
        expected: u32,
        actual: u32,
    },
    PatchHashMismatch {
        expected: u32,
        actual: u32,
    },
    InputMd5Mismatch {
        expected: [u8; 16],
        actual: [u8; 16],
    },
    OutputMd5Mismatch {
        expected: [u8; 16],
        actual: [u8; 16],
    },
    OffsetOutOfRange {
        offset: u64,
        max: u64,
    },
    NoMatchingFile,
    UnsupportedFeature(&'static str),
}

fn fmt_md5(bytes: &[u8; 16]) -> String {
    use core::fmt::Write;
    let mut s = String::with_capacity(32);
    for b in bytes {
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => f.write_str("patch is truncated"),
            Self::InvalidMagic => f.write_str("invalid patch magic bytes"),
            Self::InvalidEncoding => f.write_str("invalid patch encoding"),
            Self::InputSizeMismatch { expected, actual } => {
                write!(
                    f,
                    "input ROM size mismatch: expected {expected}, got {actual}"
                )
            }
            Self::InputHashMismatch { expected, actual } => write!(
                f,
                "input ROM hash mismatch: expected {expected:#010x}, got {actual:#010x}"
            ),
            Self::OutputHashMismatch { expected, actual } => write!(
                f,
                "output ROM hash mismatch: expected {expected:#010x}, got {actual:#010x}"
            ),
            Self::PatchHashMismatch { expected, actual } => write!(
                f,
                "patch file hash mismatch: expected {expected:#010x}, got {actual:#010x}"
            ),
            Self::InputMd5Mismatch { expected, actual } => write!(
                f,
                "input ROM MD5 mismatch: expected {}, got {}",
                fmt_md5(expected),
                fmt_md5(actual)
            ),
            Self::OutputMd5Mismatch { expected, actual } => write!(
                f,
                "output ROM MD5 mismatch: expected {}, got {}",
                fmt_md5(expected),
                fmt_md5(actual)
            ),
            Self::OffsetOutOfRange { offset, max } => {
                write!(f, "patch offset {offset} exceeds limit {max}")
            }
            Self::NoMatchingFile => f.write_str("no file in patch matches the input ROM"),
            Self::UnsupportedFeature(name) => write!(f, "unsupported patch feature: {name}"),
        }
    }
}

impl std::error::Error for PatchError {}

pub type Result<T> = core::result::Result<T, PatchError>;
