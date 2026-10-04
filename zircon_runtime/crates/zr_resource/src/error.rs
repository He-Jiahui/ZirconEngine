//! 资源目录与运行时发布的领域错误；调用端据此区分身份冲突、过期加载结果和状态迁移失败。
//! 文件持久化错误由 I/O 子模块独立承载，资源批次预检失败不会写入部分结果。

use thiserror::Error;

pub type ResourceResult<T> = std::result::Result<T, ResourceRegistryError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ResourceRegistryError {
    #[error("missing resource record for locator {locator}")]
    MissingRecordForLocator { locator: String },
    #[error("missing resource record for id {id}")]
    MissingRecordForId { id: String },
    #[error("resource locator {locator} is already owned by {existing_id}, not {requested_id}")]
    LocatorOccupied {
        locator: String,
        existing_id: String,
        requested_id: String,
    },
    #[error(
        "resource {id} locator change from {current_locator} to {requested_locator} requires rename"
    )]
    ExplicitRenameRequired {
        id: String,
        current_locator: String,
        requested_locator: String,
    },
    #[error("resource {id} kind is {current_kind:?}, not {requested_kind:?}")]
    KindConflict {
        id: String,
        current_kind: crate::ResourceKind,
        requested_kind: crate::ResourceKind,
    },
    #[error("resource {id} cannot transition from {current_state:?} to {requested_state:?}")]
    InvalidStateTransition {
        id: String,
        current_state: crate::ResourceState,
        requested_state: crate::ResourceState,
    },
    #[error(
        "resource {id} revision conflict: expected {expected_revision}, actual {actual_revision:?}"
    )]
    RevisionConflict {
        id: String,
        expected_revision: u64,
        actual_revision: Option<u64>,
    },
    #[error("resource {id} revision {current_revision} is exhausted")]
    RevisionExhausted { id: String, current_revision: u64 },
    #[error("resource event sequence is exhausted; rejected {requested_event_count} events")]
    EventSequenceExhausted { requested_event_count: usize },
}
