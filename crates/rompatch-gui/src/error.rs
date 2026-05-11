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
pub struct IpcError {
    pub kind: &'static str,
    pub message: String,
}

impl From<GuiError> for IpcError {
    fn from(e: GuiError) -> Self {
        let kind = match &e {
            GuiError::Io(_) => "io",
            GuiError::Apply(_) => "apply",
            GuiError::Patch(_) => "patch",
        };
        Self {
            kind,
            message: e.to_string(),
        }
    }
}

impl serde::Serialize for GuiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        IpcError::from(GuiError::clone_for_serialize(self)).serialize(serializer)
    }
}

impl GuiError {
    fn clone_for_serialize(e: &GuiError) -> GuiError {
        match e {
            GuiError::Io(err) => GuiError::Io(std::io::Error::new(err.kind(), err.to_string())),
            GuiError::Apply(err) => GuiError::Apply(err.clone()),
            GuiError::Patch(err) => GuiError::Patch(err.clone()),
        }
    }
}

pub type GuiResult<T> = Result<T, GuiError>;
