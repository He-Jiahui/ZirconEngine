use std::collections::BTreeSet;

use crate::core::editing::engine::{resolve_history_context, EditCommandError, HistoryContextId};
use crate::core::editor_message::DocumentId;
use crate::core::play::{PlayInstanceId, WorldDomain};

#[test]
fn routing_uses_document_context_and_only_defaults_multiple_documents_to_global() {
    let first = DocumentId::new(1);
    let second = DocumentId::new(2);
    assert_eq!(
        resolve_history_context(WorldDomain::Edit, None, Some(first), &BTreeSet::new()).unwrap(),
        HistoryContextId::Document(first)
    );

    let participants = BTreeSet::from([first, second]);
    assert_eq!(
        resolve_history_context(
            WorldDomain::Edit,
            Some(HistoryContextId::Document(first)),
            Some(first),
            &participants,
        )
        .unwrap(),
        HistoryContextId::Document(first)
    );
    assert_eq!(
        resolve_history_context(WorldDomain::Edit, None, Some(first), &participants).unwrap(),
        HistoryContextId::Global
    );
}

#[test]
fn routing_partitions_play_history_by_instance() {
    let first = PlayInstanceId::for_test(1);
    let second = PlayInstanceId::for_test(2);

    assert_eq!(
        resolve_history_context(WorldDomain::Play(first), None, None, &BTreeSet::new()).unwrap(),
        HistoryContextId::PlaySession(first)
    );
    assert_eq!(
        resolve_history_context(WorldDomain::Play(second), None, None, &BTreeSet::new()).unwrap(),
        HistoryContextId::PlaySession(second)
    );
    assert_ne!(
        HistoryContextId::PlaySession(first),
        HistoryContextId::PlaySession(second)
    );
}

#[test]
fn routing_rejects_history_from_another_world_domain() {
    let instance = PlayInstanceId::for_test(7);
    let play_history = HistoryContextId::PlaySession(instance);
    let document = DocumentId::new(9);

    assert!(matches!(
        resolve_history_context(
            WorldDomain::Edit,
            Some(play_history),
            None,
            &BTreeSet::new(),
        ),
        Err(EditCommandError::CrossWorldHistory {
            world_domain: WorldDomain::Edit,
            requested,
        }) if requested == play_history
    ));
    assert!(matches!(
        resolve_history_context(
            WorldDomain::Play(instance),
            Some(HistoryContextId::Global),
            None,
            &BTreeSet::new(),
        ),
        Err(EditCommandError::CrossWorldHistory {
            world_domain: WorldDomain::Play(found),
            requested: HistoryContextId::Global,
        }) if found == instance
    ));
    assert!(matches!(
        resolve_history_context(
            WorldDomain::Play(instance),
            Some(play_history),
            Some(document),
            &BTreeSet::from([document]),
        ),
        Err(EditCommandError::CrossWorldHistory {
            world_domain: WorldDomain::Play(found),
            requested: HistoryContextId::Document(found_document),
        }) if found == instance && found_document == document
    ));
}
