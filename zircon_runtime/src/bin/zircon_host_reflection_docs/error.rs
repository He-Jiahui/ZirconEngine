use std::io;
use std::path::PathBuf;

use thiserror::Error;
use zircon_runtime::script::VmError;

pub type HostReflectionDocsResult<T> = std::result::Result<T, HostReflectionDocsError>;

#[derive(Debug, Error)]
pub enum HostReflectionDocsError {
    #[error("{0}")]
    Usage(String),
    #[error("failed to collect built-in host modules: {source}")]
    CollectBuiltInHostModules {
        #[source]
        source: VmError,
    },
    #[error("failed to write host interface docs to {}: {source}", path.display())]
    WriteHostInterfaceDocs {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
