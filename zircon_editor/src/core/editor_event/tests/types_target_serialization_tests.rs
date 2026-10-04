use super::EditorViewportEvent;
use crate::core::editor_event::ViewInstanceId;
use crate::scene::viewport::DisplayMode;

#[test]
fn legacy_viewport_variants_keep_their_existing_serde_shape() {
    let legacy = EditorViewportEvent::SetDisplayMode {
        mode: DisplayMode::Shaded,
    };
    let encoded = serde_json::to_value(&legacy).expect("legacy viewport event serializes");
    assert_eq!(
        encoded,
        serde_json::json!({"SetDisplayMode": {"mode": "Shaded"}})
    );
    assert_eq!(
        serde_json::from_value::<EditorViewportEvent>(encoded).unwrap(),
        legacy
    );
}

#[test]
fn targeted_viewport_events_roundtrip_without_erasing_the_leaf_id() {
    let event = EditorViewportEvent::ForView {
        view_id: ViewInstanceId::new("editor.scene#right"),
        event: Box::new(EditorViewportEvent::SetDisplayMode {
            mode: DisplayMode::WireOnly,
        }),
    };
    let decoded = serde_json::from_value::<EditorViewportEvent>(
        serde_json::to_value(&event).expect("targeted viewport event serializes"),
    )
    .expect("targeted viewport event deserializes");
    assert_eq!(decoded, event);
}
