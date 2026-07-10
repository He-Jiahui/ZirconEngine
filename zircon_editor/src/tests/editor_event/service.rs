use crate::core::editor_event::{EditorEvent, EditorEventSource, EditorEventTransient};

use super::support::{env_lock, EventRuntimeHarness};

#[test]
fn separated_event_service_preserves_sequence_revision_and_journal_order() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("editor_event_service_equivalence");

    let first = harness
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Transient(EditorEventTransient::OpenCommandPalette),
        )
        .unwrap();
    let second = harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Transient(EditorEventTransient::HoverNode {
                node_path: "workbench/root".to_string(),
                hovered: false,
            }),
        )
        .unwrap();

    assert_eq!(first.sequence.0, 1);
    assert_eq!(first.before_revision, 0);
    assert_eq!(first.after_revision, 1);
    assert_eq!(second.sequence.0, 2);
    assert_eq!(second.before_revision, 1);
    assert_eq!(second.after_revision, 2);
    assert_eq!(
        harness.runtime.journal().records(),
        &[first.clone(), second.clone()]
    );
}
