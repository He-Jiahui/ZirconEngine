use zircon_runtime::core::framework::sound::SoundDynamicEventDelivery;
use zircon_runtime_interface::{
    ZrByteSlice, ZrPluginEventCallbackRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::package::events::SOUND_DYNAMIC_EVENT_NAMESPACE;

use super::slice::borrowed_slice;

pub(crate) fn sound_dynamic_event_callback_request(
    delivery: &SoundDynamicEventDelivery,
) -> ZrPluginEventCallbackRequestV1 {
    let plugin_id = delivery.handler.plugin_id.as_bytes();
    let handler_id = delivery.handler.handler_id.as_bytes();
    let event_id = delivery.invocation.event_id.as_bytes();
    let source_path = delivery
        .invocation
        .source_path
        .as_deref()
        .unwrap_or_default();
    let payload_schema = delivery.invocation.payload_schema.as_bytes();
    let payload = delivery.invocation.payload.as_slice();
    ZrPluginEventCallbackRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrByteSlice::from_static(SOUND_DYNAMIC_EVENT_NAMESPACE.as_bytes()),
        borrowed_slice(plugin_id),
        borrowed_slice(handler_id),
        borrowed_slice(event_id),
        borrowed_slice(source_path.as_bytes()),
        delivery.invocation.time_seconds,
        borrowed_slice(payload_schema),
        borrowed_slice(payload),
    )
}
