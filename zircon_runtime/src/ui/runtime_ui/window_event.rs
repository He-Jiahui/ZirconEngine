use zircon_runtime_interface::ui::{
    dispatch::UiInputDispatchResult,
    window::{runtime_event_to_window_input_pump_event, UiRuntimeEventAdapterContext},
};
use zircon_runtime_interface::ZrRuntimeEventV1;

use super::runtime_ui_manager::RuntimeUiManager;
use super::runtime_ui_manager_error::RuntimeUiManagerError;

pub(super) fn dispatch_runtime_event(
    manager: &mut RuntimeUiManager,
    context: &UiRuntimeEventAdapterContext,
    event: ZrRuntimeEventV1,
) -> Result<UiInputDispatchResult, RuntimeUiManagerError> {
    let pump_event = runtime_event_to_window_input_pump_event(context, event)?;
    manager
        .dispatch_window_input_pump_event(pump_event)
        .map_err(RuntimeUiManagerError::from)
}

pub(super) fn dispatch_runtime_event_batch(
    manager: &mut RuntimeUiManager,
    context: &UiRuntimeEventAdapterContext,
    events: impl IntoIterator<Item = ZrRuntimeEventV1>,
) -> Result<Vec<UiInputDispatchResult>, RuntimeUiManagerError> {
    let mut results = Vec::new();
    for (index, event) in events.into_iter().enumerate() {
        let result = dispatch_runtime_event(manager, context, event).map_err(|source| {
            RuntimeUiManagerError::RuntimeEventBatch {
                index,
                source: Box::new(source),
            }
        })?;
        results.push(result);
    }
    Ok(results)
}
