//! Domain-owned validation, evidence hashing, and restart disposition.

use std::io;
use std::path::Path;

use super::super::schema::JournalDocument;
use super::super::stage::digest_file;

/// Determines whether restart recovery restores live files or only retires reserved artifacts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryMode {
    /// Restore the original live generation for an interrupted active transaction.
    RestoreOriginal,
    /// Preserve live files and clean only the journal's validated reserved artifacts.
    CleanupArtifacts,
}

/// 实现者负责验证项目根、允许修改的目标与退休文件范围；通用日志校验不能替代业务授权。
/// Domain-owned validation and digest policy for a recovered journal document.
pub trait RecoveryPolicy {
    /// Sampled once under the transaction owner lock for each detection or recovery call.
    fn recovery_mode(&self) -> RecoveryMode {
        RecoveryMode::RestoreOriginal
    }

    fn validate_document(
        &self,
        journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String>;

    fn digest_file(&mut self, path: &Path) -> io::Result<String> {
        digest_file(path)
    }
}
