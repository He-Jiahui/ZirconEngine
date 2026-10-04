//! 持久化事务失败的领域边界；操作错误保留阶段、路径和底层 I/O 原因，便于区分提交失败与恢复失败。
//! 返回错误并不证明磁盘未改变；调用端还需检查保留的日志并完成恢复，才能开启同一 owner 的下一批事务。

use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::io::ArtifactIdentityExhausted;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionPhase {
    Recovery,
    Stage,
    Commit,
    Rollback,
}

#[derive(Debug, Error)]
pub enum DurableTransactionError {
    #[error(transparent)]
    ArtifactIdentityExhausted(#[from] ArtifactIdentityExhausted),
    #[error("invalid durable transaction journal {path}: {reason}")]
    InvalidJournal { path: PathBuf, reason: String },
    #[error("failed to deserialize durable transaction journal {path}: {source}")]
    JournalDeserialize {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("durable file transaction failed during {phase:?} for {path}: {source}")]
    Operation {
        phase: TransactionPhase,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl DurableTransactionError {
    pub(crate) fn invalid(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::InvalidJournal {
            path: path.into(),
            reason: reason.into(),
        }
    }

    pub(crate) fn operation(
        phase: TransactionPhase,
        path: impl Into<PathBuf>,
        source: io::Error,
    ) -> Self {
        Self::Operation {
            phase,
            path: path.into(),
            source,
        }
    }
}
