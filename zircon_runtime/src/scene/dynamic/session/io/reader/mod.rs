#[path = "contract.rs"]
mod contract;
#[path = "service.rs"]
mod service;

pub use contract::{
    RuntimeSessionArchiveReadArtifact, RuntimeSessionArchiveReadOutcome,
    RuntimeSessionArchiveReadSubmission, RuntimeSessionArchiveReaderDiagnostics,
    RuntimeSessionArchiveReaderLimits, RuntimeSessionArchiveReaderSubmitError,
};
pub use service::RuntimeSessionArchiveReader;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
