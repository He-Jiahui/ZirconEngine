#[path = "context.rs"]
mod context;
#[path = "error.rs"]
mod error;
#[path = "reason.rs"]
mod reason;
#[path = "report.rs"]
mod report;
#[path = "source_write_authority.rs"]
mod source_write_authority;

pub use context::{SaveContextError, SaveCtx};
pub use error::{SaveError, SaveHookPanic};
pub use reason::SaveReason;
pub use report::DocumentSaveReport;
pub(crate) use source_write_authority::{
    DocumentSourceWriteAuthority, DocumentSourceWriteLease, DocumentSourceWriteOutcome,
    DocumentSourceWritePublication, DocumentSourceWriteReceipt,
};
