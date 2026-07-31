use crate::core::editor_message::{
    EditorMessageBus, EditorMessageProtocol, EditorViewInvalidationMask,
};

use super::fixture::{topic, typed_messages, view};

#[test]
fn publish_routes_each_typed_family_only_to_its_exact_topic() {
    let mut bus = EditorMessageBus::default();
    let subscribers = typed_messages()
        .into_iter()
        .map(|(topic_name, _)| bus.register_subscriber([topic(topic_name)]).unwrap())
        .collect::<Vec<_>>();

    for (index, (topic_name, message)) in typed_messages().into_iter().enumerate() {
        let report = bus.publish(topic(topic_name), message.clone());

        assert_eq!(report.protocol(), EditorMessageProtocol::Publish);
        assert_eq!(report.delivered(), &[subscribers[index]]);
        assert_eq!(
            bus.deliveries_for(subscribers[index])[0]
                .message()
                .payload(),
            message.payload()
        );
        for (other_index, subscriber) in subscribers.iter().enumerate() {
            let expected_delivery_count = usize::from(other_index <= index);
            assert_eq!(
                bus.deliveries_for(*subscriber).len(),
                expected_delivery_count
            );
        }
    }
}

#[test]
fn publish_marks_the_message_view_dirty() {
    let mut bus = EditorMessageBus::default();
    let subscriber = bus.register_subscriber([topic("editor.focus")]).unwrap();
    let scene_view = view("scene.workspace");
    let (_, focus_message) = typed_messages().pop().unwrap();

    bus.publish(
        topic("editor.focus"),
        focus_message.with_dirty(
            scene_view.clone(),
            EditorViewInvalidationMask::PRESENTATION_DATA,
        ),
    );

    assert_eq!(bus.deliveries_for(subscriber).len(), 1);
    assert_eq!(
        bus.dirty_set().mask_for(&scene_view),
        Some(EditorViewInvalidationMask::PRESENTATION_DATA)
    );
}
