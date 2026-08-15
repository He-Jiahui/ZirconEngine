use std::error::Error;
use std::path::{Path, PathBuf};

use super::{DocumentToolkitDescriptor, SaveCtx};

pub type ToolkitSaveFailure = Box<dyn Error + Send + Sync + 'static>;

/// Immutable document bytes captured while the toolkit save exclusion is held.
/// The recovery owner converts `source_path` to the existing project-relative
/// path contract before it persists the snapshot.
pub struct DocumentAutosavePayload {
    source_path: PathBuf,
    bytes: Vec<u8>,
}

impl DocumentAutosavePayload {
    pub fn new(source_path: impl Into<PathBuf>, bytes: Vec<u8>) -> Self {
        Self {
            source_path: source_path.into(),
            bytes,
        }
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// Implements one open document's concrete persistence hook without owning dirty state.
pub trait DocumentToolkit<Host>: Send + Sync {
    fn descriptor(&self) -> &DocumentToolkitDescriptor;

    fn save(&self, host: &Host, context: &mut SaveCtx) -> Result<(), ToolkitSaveFailure>;

    /// Returns the physical canonical source without serializing document bytes.
    fn autosave_source_path(&self, host: &Host) -> Result<PathBuf, ToolkitSaveFailure>;

    fn capture_autosave(&self, host: &Host) -> Result<DocumentAutosavePayload, ToolkitSaveFailure>;
}
