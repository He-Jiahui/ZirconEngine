use crate::core::editor_message::{DocumentId, SelectionDomain};
use crate::core::jobs::{JobEventKind, JobId};

use super::{
    DocumentMessage, EditorMessage, EditorMessagePayload, EditorMessageProtocol, FocusMessage,
    ModeMessage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EditorMessageRetention {
    Lossless,
    Latest(EditorMessageCoalescingKey),
    Bounded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum EditorMessageCoalescingKey {
    DocumentDirty(DocumentId),
    DocumentFocus,
    SceneMode,
    Selection(SelectionDomain),
    FocusObject,
    JobProgress(JobId),
    SceneInspection,
}

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
        FocusMessage::FocusObject { .. } => {
            EditorMessageRetention::Latest(EditorMessageCoalescingKey::FocusObject)
        }
    }
}
