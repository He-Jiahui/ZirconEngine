use std::io;
use std::path::Path;

use crate::foundation::persistence::atomic_file::stage_atomic_write;

use super::commit_fence::ConfigCommitFence;

pub(in crate::foundation::runtime) trait ConfigFileWriter:
    Send + Sync
{
    fn write(&self, path: &Path, bytes: &[u8], commit_fence: &ConfigCommitFence) -> io::Result<()>;
}

#[derive(Debug, Default)]
pub(super) struct AtomicConfigFileWriter;

impl ConfigFileWriter for AtomicConfigFileWriter {
    fn write(&self, path: &Path, bytes: &[u8], commit_fence: &ConfigCommitFence) -> io::Result<()> {
        let pending = stage_atomic_write(path, bytes)?;
        commit_fence.commit(|| pending.commit())
    }
}
