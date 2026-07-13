use std::collections::BTreeSet;

use crate::core::editor_message::DocumentId;

use super::HistoryContextId;

pub fn resolve_history_context(
    explicit: Option<HistoryContextId>,
    target_document: Option<DocumentId>,
    participants: &BTreeSet<DocumentId>,
) -> HistoryContextId {
    if participants.len() > 1 {
        return HistoryContextId::Global;
    }
    match explicit.or_else(|| target_document.map(HistoryContextId::Document)) {
        Some(context) => context,
        None => HistoryContextId::Global,
    }
}
