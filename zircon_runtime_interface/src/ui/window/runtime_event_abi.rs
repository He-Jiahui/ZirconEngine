use crate::{
    ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1,
};

use super::{
    runtime_event_to_window_input_pump_event, UiRuntimeEvent, UiRuntimeEventAdapterContext,
    UiRuntimeEventAdapterError, UiRuntimeEventAdapterResult, UiWindowInputPumpBatch,
    UiWindowInputPumpEvent,
};

/// Converts a raw host event at the ABI boundary without retaining its payload.
///
/// # Safety
///
/// Every nonempty payload that passes carrier shape and size checks must point to
/// initialized bytes that remain readable and immutable until this call returns.
pub unsafe fn runtime_abi_event_to_window_input_pump_event(
    context: &UiRuntimeEventAdapterContext,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowInputPumpEvent> {
    if event.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err(UiRuntimeEventAdapterError::UnsupportedAbi {
            actual: event.abi_version,
            expected: ZIRCON_RUNTIME_ABI_VERSION_V1,
        });
    }
    // The raw borrow is bounded to this synchronous conversion; output text is owned.
    let borrowed = unsafe { UiRuntimeEvent::from_abi(event) }.map_err(|_| {
        if event.kind == ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1 {
            UiRuntimeEventAdapterError::InvalidAccessibilityPayload
        } else {
            UiRuntimeEventAdapterError::InvalidTextPayload
        }
    })?;
    runtime_event_to_window_input_pump_event(context, borrowed)
}

/// Converts in input order and returns no prefix batch on any conversion error.
///
/// # Safety
///
/// Each raw event must satisfy [`runtime_abi_event_to_window_input_pump_event`]'s
/// payload requirements through its conversion.
pub unsafe fn runtime_abi_events_to_window_input_pump_batch(
    context: &UiRuntimeEventAdapterContext,
    events: impl IntoIterator<Item = ZrRuntimeEventV1>,
) -> UiRuntimeEventAdapterResult<UiWindowInputPumpBatch> {
    let events = events.into_iter();
    let mut batch = UiWindowInputPumpBatch::with_capacity(events.size_hint().0);
    for event in events {
        batch.push(unsafe { runtime_abi_event_to_window_input_pump_event(context, event)? });
    }
    Ok(batch)
}
