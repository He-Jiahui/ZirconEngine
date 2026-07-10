use crate::core::editor_message::{EditorMessageBus, EditorMessageProtocol};

use super::fixture::{topic, typed_messages};

#[test]
fn broadcast_reaches_all_subscribers_without_topic_filtering() {
    let mut bus = EditorMessageBus::default();
    let scene = bus.register_subscriber([topic("editor.focus")]);
    let console = bus.register_subscriber([topic("editor.document")]);
    let (_, message) = typed_messages().pop().unwrap();

    let report = bus.broadcast(topic("editor.mode"), message);

    assert_eq!(report.protocol(), EditorMessageProtocol::Broadcast);
    assert_eq!(report.delivered(), &[scene, console]);
    assert_eq!(bus.deliveries_for(scene).len(), 1);
    assert_eq!(bus.deliveries_for(console).len(), 1);
    assert_eq!(bus.deliveries_for(scene)[0].topic(), &topic("editor.mode"));
}
