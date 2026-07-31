use crate::core::editor_message::{
    EditorMessageBus, EditorMessageBusError, EditorMessageRequest, EditorMessageResponse,
    EditorRequestHandler, EditorSubscriberId, SharedEditorMessageBus,
};

use super::fixture::{response_message, topic, typed_messages};

struct EchoHandler;

impl EditorRequestHandler for EchoHandler {
    fn handle_editor_request(&mut self, request: &EditorMessageRequest) -> EditorMessageResponse {
        assert_eq!(request.topic(), &topic("editor.document"));
        assert_eq!(request.target(), EditorSubscriberId::new(1));
        EditorMessageResponse::handled(response_message())
    }
}

#[test]
fn request_validates_target_and_records_delivery() {
    let mut bus = EditorMessageBus::default();
    let target = bus.register_subscriber([topic("editor.document")]).unwrap();
    let mut handler = EchoHandler;
    let (_, request_message) = typed_messages().remove(0);

    let response = bus
        .request(
            target,
            topic("editor.document"),
            request_message.clone(),
            &mut handler,
        )
        .unwrap();

    assert_eq!(response.message(), &response_message());
    assert_eq!(bus.deliveries_for(target).len(), 1);
    assert_eq!(bus.deliveries_for(target)[0].message(), &request_message);

    let error = bus
        .request(
            EditorSubscriberId::new(99),
            topic("editor.document"),
            request_message,
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

struct ReentrantHandler {
    bus: SharedEditorMessageBus,
}

impl EditorRequestHandler for ReentrantHandler {
    fn handle_editor_request(&mut self, _request: &EditorMessageRequest) -> EditorMessageResponse {
        let (_, nested_message) = typed_messages().remove(1);
        self.bus
            .publish(topic("editor.transaction"), nested_message);
        EditorMessageResponse::handled(response_message())
    }
}

#[test]
fn shared_request_releases_the_bus_lock_before_invoking_the_handler() {
    let bus = SharedEditorMessageBus::default();
    let request_target = bus.register_subscriber([topic("editor.document")]).unwrap();
    let nested_target = bus
        .register_subscriber([topic("editor.transaction")])
        .unwrap();
    let mut handler = ReentrantHandler { bus: bus.clone() };
    let (_, request_message) = typed_messages().remove(0);

    let response = bus
        .request(
            request_target,
            topic("editor.document"),
            request_message,
            &mut handler,
        )
        .unwrap();

    assert_eq!(response.message(), &response_message());
    assert_eq!(bus.deliveries_for(request_target).len(), 1);
    assert_eq!(bus.deliveries_for(nested_target).len(), 1);
}

struct RemovingHandler {
    bus: SharedEditorMessageBus,
    target: EditorSubscriberId,
}

impl EditorRequestHandler for RemovingHandler {
    fn handle_editor_request(&mut self, _request: &EditorMessageRequest) -> EditorMessageResponse {
        assert!(self.bus.unregister_subscriber(self.target));
        EditorMessageResponse::handled(response_message())
    }
}

#[test]
fn shared_request_revalidates_the_target_after_the_handler_returns() {
    let bus = SharedEditorMessageBus::default();
    let target = bus.register_subscriber([topic("editor.document")]).unwrap();
    let mut handler = RemovingHandler {
        bus: bus.clone(),
        target,
    };
    let (_, request_message) = typed_messages().remove(0);

    let error = bus
        .request(
            target,
            topic("editor.document"),
            request_message,
            &mut handler,
        )
        .unwrap_err();

    assert_eq!(
        error,
        EditorMessageBusError::UnknownSubscriber { subscriber: target }
    );
}
