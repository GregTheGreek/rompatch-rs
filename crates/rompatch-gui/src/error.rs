//! IPC-facing error type. Tauri serializes return values as JSON, so we
//! flatten Rust error variants down to a small struct the frontend can
//! switch on.

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum GuiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Apply(#[from] rompatch_core::ApplyError),
    #[error("{0}")]
    Patch(#[from] rompatch_core::PatchError),
}

#[derive(Debug, Serialize)]
struct IpcError {
    kind: &'static str,
    message: String,
}

impl serde::Serialize for GuiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let kind = match self {
            GuiError::Io(_) => "io",
            GuiError::Apply(_) => "apply",
            GuiError::Patch(_) => "patch",
        };
        IpcError {
            kind,
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

pub type GuiResult<T> = Result<T, GuiError>;
