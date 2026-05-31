use zircon_runtime::core::framework::sound::SoundDynamicEventDelivery;
use zircon_runtime_interface::{
    ZrPluginEventCallbackFnV1, ZrPluginEventCallbackResultV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::request::sound_dynamic_event_callback_request;
use super::status::status_detail;

pub(crate) fn sound_dynamic_event_callback_executor(
    callback: ZrPluginEventCallbackFnV1,
) -> impl Fn(&SoundDynamicEventDelivery) -> Result<(), String> + Send + Sync + 'static {
    move |delivery| invoke_sound_dynamic_event_callback(callback, delivery)
}

pub(crate) fn invoke_sound_dynamic_event_callback(
    callback: ZrPluginEventCallbackFnV1,
    delivery: &SoundDynamicEventDelivery,
) -> Result<(), String> {
    let request = sound_dynamic_event_callback_request(delivery);
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
