use crate::core::editor_message::DocumentId;

use super::*;

#[test]
fn playing_policy_applies_locks_and_queues_by_target() {
    let running_document = DocumentId::new(7);
    let other_document = DocumentId::new(8);
    let mut policy = PlayEditPolicy::default();
    policy.begin_play(Some(running_document));

    assert_eq!(
        policy.evaluate(PlayEditTarget::PlayDomain),
        PlayEditDecision::ApplyNow
    );
    assert_eq!(
        policy.evaluate(PlayEditTarget::EditDocument(running_document)),
        PlayEditDecision::RunningDocumentLocked {
            document: running_document
        }
    );
    assert_eq!(
        policy.evaluate(PlayEditTarget::EditDocument(other_document)),
        PlayEditDecision::QueueUntilPlayStops
    );
    assert_eq!(
        policy.evaluate(PlayEditTarget::EditWorkspace),
        PlayEditDecision::QueueUntilPlayStops
    );
}

#[test]
fn edit_mode_applies_edit_targets_and_rejects_missing_play_domain() {
    let policy = PlayEditPolicy::default();

    assert_eq!(
        policy.evaluate(PlayEditTarget::EditDocument(DocumentId::new(1))),
        PlayEditDecision::ApplyNow
    );
    assert_eq!(
        policy.evaluate(PlayEditTarget::EditWorkspace),
        PlayEditDecision::ApplyNow
    );
    assert_eq!(
        policy.evaluate(PlayEditTarget::PlayDomain),
        PlayEditDecision::PlayDomainUnavailable
    );
}
