use zircon_runtime::core::framework::sound::SoundDynamicEventDelivery;
use zircon_runtime_interface::{
    ZrByteSlice, ZrPluginEventCallbackFnV1, ZrPluginEventCallbackRequestV1,
    ZrPluginEventCallbackResultV1, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::package::SOUND_DYNAMIC_EVENT_NAMESPACE;
use crate::service_types::DefaultSoundManager;

impl DefaultSoundManager {
    pub fn register_dynamic_event_abi_callback(
        &self,
        plugin_id: impl Into<String>,
        handler_id: impl Into<String>,
        callback: ZrPluginEventCallbackFnV1,
    ) -> Result<(), zircon_runtime::core::framework::sound::SoundError> {
        self.register_dynamic_event_executor(
            plugin_id,
            handler_id,
            sound_dynamic_event_callback_executor(callback),
        )
    }
}

pub(crate) fn sound_dynamic_event_callback_executor(
    callback: ZrPluginEventCallbackFnV1,
) -> impl Fn(&SoundDynamicEventDelivery) -> Result<(), String> + Send + Sync + 'static {
    move |delivery| invoke_sound_dynamic_event_callback(callback, delivery)
}

pub(crate) fn invoke_sound_dynamic_event_callback(
    callback: ZrPluginEventCallbackFnV1,
    delivery: &SoundDynamicEventDelivery,
) -> Result<(), String> {
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
    let request = ZrPluginEventCallbackRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrByteSlice::from_static(SOUND_DYNAMIC_EVENT_NAMESPACE.as_bytes()),
        borrowed_slice(plugin_id),
        borrowed_slice(handler_id),
        borrowed_slice(event_id),
        borrowed_slice(source_path.as_bytes()),
        delivery.invocation.time_seconds,
        borrowed_slice(payload_schema),
        borrowed_slice(payload),
    );
    let mut result = ZrPluginEventCallbackResultV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    let status = unsafe { callback(request, &mut result) };
    if !status.is_ok() {
        return Err(status_detail(status.status_code(), status.diagnostics));
    }
    if result.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err("sound dynamic event callback returned an unsupported ABI version".to_string());
    }
    if result.status.is_ok() {
        Ok(())
    } else {
        Err(status_detail(
            result.status.status_code(),
            result.status.diagnostics,
        ))
    }
}

fn borrowed_slice(bytes: &[u8]) -> ZrByteSlice {
    if bytes.is_empty() {
        ZrByteSlice::empty()
    } else {
        ZrByteSlice {
            data: bytes.as_ptr(),
            len: bytes.len(),
        }
    }
}

fn status_detail(code: ZrStatusCode, diagnostics: ZrByteSlice) -> String {
    let diagnostics = unsafe { diagnostics.as_slice() };
    if diagnostics.is_empty() {
        format!("sound dynamic event callback returned {code:?}")
    } else {
        String::from_utf8_lossy(diagnostics).into_owned()
    }
}
