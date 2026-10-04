use crate::core::editor_message::{DocumentId, SelectionDomain};
use crate::core::jobs::{JobEventKind, JobId};
use crate::core::play::WorldDomain;

use super::{
    DocumentMessage, EditorMessage, EditorMessagePayload, EditorMessageProtocol, FocusMessage,
    ModeMessage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// 收件箱对事实的保留责任：逐条必须交付、同键仅需最新态、或允许有界淘汰。
pub(super) enum EditorMessageRetention {
    Lossless,
    Latest(EditorMessageCoalescingKey),
    Bounded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
// 同键表示可被后续事实替代；文档、任务和世界域身份必须在键中保留，避免互相覆盖。
pub(super) enum EditorMessageCoalescingKey {
    DocumentDirty(DocumentId),
    DocumentFocus,
    SceneMode,
    Selection(SelectionDomain),
    FocusObject(WorldDomain),
    JobProgress(JobId),
    SceneInspection,
}

// 保留策略按协议及事实语义选择；事务和生命周期不能跳过中间项，状态通知则由消费者回查权威状态。
// 场景检查仅保留最新层级代次，选择变化由消息层合并；代次不连续时消费方必须重同步。
pub(super) fn editor_message_retention(
    protocol: EditorMessageProtocol,
    message: &EditorMessage,
) -> EditorMessageRetention {
    if protocol == EditorMessageProtocol::Request {
        return EditorMessageRetention::Lossless;
    }

    match message.payload() {
        EditorMessagePayload::Transaction(_) => EditorMessageRetention::Lossless,
        EditorMessagePayload::Document(document) => document_retention(document),
        EditorMessagePayload::Mode(mode) => mode_retention(mode),
        EditorMessagePayload::Focus(focus) => focus_retention(focus),
        EditorMessagePayload::SceneInspection(_) => {
            EditorMessageRetention::Latest(EditorMessageCoalescingKey::SceneInspection)
        }
        EditorMessagePayload::Tool(_) => EditorMessageRetention::Lossless,
        EditorMessagePayload::Job(job) => match job.kind() {
            JobEventKind::Progress { .. } => {
                EditorMessageRetention::Latest(EditorMessageCoalescingKey::JobProgress(job.id()))
            }
            JobEventKind::Started
            | JobEventKind::Completed
            | JobEventKind::Failed { .. }
            | JobEventKind::Cancelled => EditorMessageRetention::Lossless,
        },
        EditorMessagePayload::JobJournalGap(_) => EditorMessageRetention::Lossless,
        EditorMessagePayload::Custom { .. } => EditorMessageRetention::Bounded,
    }
}

fn document_retention(message: &DocumentMessage) -> EditorMessageRetention {
    match message {
        DocumentMessage::DirtyChanged { doc, .. } => {
            EditorMessageRetention::Latest(EditorMessageCoalescingKey::DocumentDirty(*doc))
        }
        DocumentMessage::FocusRequested { .. } => {
            EditorMessageRetention::Latest(EditorMessageCoalescingKey::DocumentFocus)
        }
        DocumentMessage::Opened { .. }
        | DocumentMessage::Closed { .. }
        | DocumentMessage::Saved { .. } => EditorMessageRetention::Lossless,
    }
}

fn mode_retention(message: &ModeMessage) -> EditorMessageRetention {
    match message {
        ModeMessage::SceneModeChanged { .. } => {
            EditorMessageRetention::Latest(EditorMessageCoalescingKey::SceneMode)
        }
        ModeMessage::PlayStateChanged { .. } => EditorMessageRetention::Lossless,
    }
}

fn focus_retention(message: &FocusMessage) -> EditorMessageRetention {
    match message {
        FocusMessage::SelectionChanged { domain, .. } => {
            EditorMessageRetention::Latest(EditorMessageCoalescingKey::Selection(*domain))
        }
        FocusMessage::FocusObject { domain, .. } => {
            EditorMessageRetention::Latest(EditorMessageCoalescingKey::FocusObject(*domain))
        }
    }
}
