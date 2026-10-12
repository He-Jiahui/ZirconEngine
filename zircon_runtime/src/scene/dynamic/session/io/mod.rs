//! 会话档案的路径和后台 I/O 边界；封存结果、路径身份与任务票据在此汇合。

#[path = "atomic.rs"]
mod atomic;
#[path = "load_save/mod.rs"]
mod load_save;
#[path = "mutation.rs"]
mod mutation;
#[path = "reader/mod.rs"]
mod reader;
#[path = "support.rs"]
mod support;
#[path = "writer.rs"]
mod writer;

pub(super) use atomic::{save_artifact_to_path_atomically, save_to_path_atomically};
pub(super) use load_save::{
    load_from_path, load_or_empty_from_path, preview_save_to_path, save_to_path,
};
pub(super) use mutation::{
    mutate_archive_at_path_atomically, mutate_archive_at_path_with_report_atomically,
};
pub use reader::{
    RuntimeSessionArchiveReadArtifact, RuntimeSessionArchiveReadOutcome,
    RuntimeSessionArchiveReadSubmission, RuntimeSessionArchiveReader,
    RuntimeSessionArchiveReaderDiagnostics, RuntimeSessionArchiveReaderLimits,
    RuntimeSessionArchiveReaderSubmitError,
};
pub use writer::{
    RuntimeSessionArchiveWriteSubmission, RuntimeSessionArchiveWriter,
    RuntimeSessionArchiveWriterLimits, RuntimeSessionArchiveWriterSubmitError,
};
