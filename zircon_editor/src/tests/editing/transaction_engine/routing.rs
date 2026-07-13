use std::collections::BTreeSet;

use crate::core::editing::engine::{resolve_history_context, HistoryContextId};
use crate::core::editor_message::DocumentId;

#[test]
fn routing_uses_document_context_until_multiple_documents_participate() {
    let first = DocumentId::new(1);
    let second = DocumentId::new(2);
    assert_eq!(
        resolve_history_context(None, Some(first), &BTreeSet::new()),
        HistoryContextId::Document(first)
    );

    let participants = BTreeSet::from([first, second]);
    assert_eq!(
        resolve_history_context(
            Some(HistoryContextId::Document(first)),
            Some(first),
            &participants,
        ),
        HistoryContextId::Global
    );
}
