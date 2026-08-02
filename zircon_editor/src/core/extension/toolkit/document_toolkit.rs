use std::error::Error;

use super::{DocumentToolkitDescriptor, SaveCtx};

pub type ToolkitSaveFailure = Box<dyn Error + Send + Sync + 'static>;

/// Implements one open document's concrete persistence hook without owning dirty state.
pub trait DocumentToolkit<Host>: Send + Sync {
    fn descriptor(&self) -> &DocumentToolkitDescriptor;

    fn save(&self, host: &Host, context: &mut SaveCtx) -> Result<(), ToolkitSaveFailure>;
}
