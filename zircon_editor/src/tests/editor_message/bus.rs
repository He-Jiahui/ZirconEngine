use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::{
    EditorMessage, EditorMessageBus, EditorMessageBusError, EditorMessagePayload,
    EditorMessageProtocol, EditorMessageRequest, EditorMessageResponse, EditorRequestHandler,
    EditorSubscriberId, EditorTopic, EditorViewInvalidationMask, ViewDirtySet,
};

fn topic(value: &str) -> EditorTopic {
    EditorTopic::parse(value).unwrap()
}

fn view(value: &str) -> ViewInstanceId {
    ViewInstanceId::new(value)
}

#[test]
fn publish_only_delivers_to_matching_subscribers_and_marks_dirty_view() {
    let mut bus = EditorMessageBus::default();
    let selection_subscriber = bus.register_subscriber([topic("selection.changed")]);
    let asset_subscriber = bus.register_subscriber([topic("asset.tree.changed")]);
    let scene_view = view("scene.workspace");

    let report = bus.publish(
        topic("selection.changed"),
        EditorMessage::text("node-42").with_dirty(
            scene_view.clone(),
            EditorViewInvalidationMask::PRESENTATION_DATA,
        ),
    );

    assert_eq!(report.protocol(), EditorMessageProtocol::Publish);
    assert_eq!(report.delivered(), &[selection_subscriber]);
    assert_eq!(bus.deliveries_for(selection_subscriber).len(), 1);
    assert!(bus.deliveries_for(asset_subscriber).is_empty());
    assert_eq!(
        bus.dirty_set().mask_for(&scene_view),
        Some(EditorViewInvalidationMask::PRESENTATION_DATA)
    );

    let dirty = bus.drain_dirty();
    assert_eq!(dirty.len(), 1);
    assert!(bus.dirty_set().is_empty());
}

#[test]
fn view_dirty_set_merges_masks_per_view_and_keeps_views_separate() {
    let scene_view = view("scene.workspace");
    let inspector_view = view("inspector.properties");
    let mut dirty = ViewDirtySet::default();

    dirty.mark(scene_view.clone(), EditorViewInvalidationMask::PAINT_ONLY);
    dirty.mark(scene_view.clone(), EditorViewInvalidationMask::HIT_TEST);
    dirty.mark(inspector_view.clone(), EditorViewInvalidationMask::LAYOUT);

    assert_eq!(dirty.len(), 2);
    assert_eq!(
        dirty.mask_for(&scene_view),
        Some(EditorViewInvalidationMask::PAINT_ONLY.union(EditorViewInvalidationMask::HIT_TEST))
    );
    assert_eq!(
        dirty.mask_for(&inspector_view),
        Some(EditorViewInvalidationMask::LAYOUT)
    );
}

#[test]
fn request_response_validates_target_and_records_request_delivery() {
    struct EchoHandler;

    impl EditorRequestHandler for EchoHandler {
        fn handle_editor_request(
            &mut self,
            request: &EditorMessageRequest,
        ) -> EditorMessageResponse {
            assert_eq!(request.topic(), &topic("layout.query"));
            assert_eq!(request.target(), EditorSubscriberId::new(1));
            EditorMessageResponse::handled(EditorMessage::text("layout-state"))
        }
    }

    let mut bus = EditorMessageBus::default();
    let target = bus.register_subscriber([topic("layout.query")]);
    let mut handler = EchoHandler;

    let response = bus
        .request(
            target,
            topic("layout.query"),
            EditorMessage::text("current"),
            &mut handler,
        )
        .unwrap();

    assert_eq!(
        response.message().payload(),
        &EditorMessagePayload::Text("layout-state".to_string())
    );
    assert_eq!(bus.deliveries_for(target).len(), 1);
    assert_eq!(
        bus.deliveries_for(target)[0].protocol(),
        EditorMessageProtocol::Request
    );

    let error = bus
        .request(
            EditorSubscriberId::new(99),
            topic("layout.query"),
            EditorMessage::empty(),
            &mut handler,
        )
        .unwrap_err();
    assert_eq!(
        error,
        EditorMessageBusError::UnknownSubscriber {
            subscriber: EditorSubscriberId::new(99)
        }
    );
}

#[test]
fn broadcast_reaches_all_subscribers_without_topic_filtering() {
    let mut bus = EditorMessageBus::default();
    let scene = bus.register_subscriber([topic("selection.changed")]);
    let console = bus.register_subscriber([topic("diagnostics.changed")]);

    let report = bus.broadcast(topic("theme.changed"), EditorMessage::empty());

    assert_eq!(report.protocol(), EditorMessageProtocol::Broadcast);
    assert_eq!(report.delivered(), &[scene, console]);
    assert_eq!(bus.deliveries_for(scene).len(), 1);
    assert_eq!(bus.deliveries_for(console).len(), 1);
    assert_eq!(
        bus.deliveries_for(scene)[0].topic(),
        &topic("theme.changed")
    );
}
