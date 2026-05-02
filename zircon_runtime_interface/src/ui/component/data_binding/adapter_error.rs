use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ui::component::{UiComponentEventKind, UiValueKind};

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiComponentAdapterError {
    #[error("unsupported Runtime UI component target domain {domain}")]
    UnsupportedTargetDomain { domain: String },
    #[error("unsupported Runtime UI component target path {path} in domain {domain}")]
    UnsupportedTargetPath { domain: String, path: String },
    #[error("Runtime UI component target {domain}:{path} is missing source {source_name}")]
    MissingSource {
        domain: String,
        path: String,
        source_name: String,
    },
    #[error("component target {domain}:{path} does not support event {event_kind:?}")]
    UnsupportedEvent {
        domain: String,
        path: String,
        event_kind: UiComponentEventKind,
    },
    #[error(
        "invalid value kind {actual:?} for component target {domain}:{path}; expected {expected:?}"
    )]
    InvalidValueKind {
        domain: String,
        path: String,
        expected: UiValueKind,
        actual: UiValueKind,
    },
    #[error("component target {domain}:{path} rejected input: {reason}")]
    RejectedInput {
        domain: String,
        path: String,
        reason: String,
    },
    #[error("component target {domain}:{path} mutation failed: {reason}")]
    HostMutation {
        domain: String,
        path: String,
        reason: String,
    },
}
