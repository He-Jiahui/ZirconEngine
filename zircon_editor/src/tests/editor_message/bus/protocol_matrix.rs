use crate::core::editor_message::{
    EditorMessage, EditorMessageBus, EditorMessageProtocol, EditorMessageRequest,
    EditorMessageResponse, EditorRequestHandler,
};

use super::fixture::{topic, typed_messages};

struct EchoPayloadHandler;

impl EditorRequestHandler for EchoPayloadHandler {
    fn handle_editor_request(&mut self, request: &EditorMessageRequest) -> EditorMessageResponse {
        EditorMessageResponse::handled(request.message().clone())
    }
}

#[test]
fn publish_request_and_broadcast_preserve_all_typed_payload_families() {
    for (topic_name, message) in typed_messages() {
        assert_protocol_preserves_payload(EditorMessageProtocol::Publish, topic_name, &message);
        assert_protocol_preserves_payload(EditorMessageProtocol::Request, topic_name, &message);
        assert_protocol_preserves_payload(EditorMessageProtocol::Broadcast, topic_name, &message);
    }
}

fn assert_protocol_preserves_payload(
    protocol: EditorMessageProtocol,
    topic_name: &str,
    message: &EditorMessage,
) {
    let mut bus = EditorMessageBus::default();
    let target = bus.register_subscriber([topic(topic_name)]).unwrap();

    match protocol {
        EditorMessageProtocol::Publish => {
            bus.publish(topic(topic_name), message.clone());
        }
        EditorMessageProtocol::Request => {
            bus.request(
                target,
                topic(topic_name),
                message.clone(),
                &mut EchoPayloadHandler,
            )
            .unwrap();
        }
        EditorMessageProtocol::Broadcast => {
            bus.broadcast(topic(topic_name), message.clone());
        }
    }

    let deliveries = bus.deliveries_for(target);
    let delivery = &deliveries[0];
    assert_eq!(delivery.protocol(), protocol);
    assert_eq!(delivery.message().payload(), message.payload());
}
