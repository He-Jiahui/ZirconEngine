//! 运行时 ABI 事件先转换为窗口泵事件，再交测试管理器；转换和分发失败都按原批次索引上报。

use zircon_runtime_interface::ui::{
    dispatch::UiInputDispatchResult,
    window::{runtime_abi_event_to_window_input_pump_event, UiRuntimeEventAdapterContext},
};
use zircon_runtime_interface::ZrRuntimeEventV1;

use super::runtime_ui_manager::RuntimeUiManager;
use super::runtime_ui_manager_error::RuntimeUiManagerError;

pub(super) fn dispatch_runtime_event(
    manager: &mut RuntimeUiManager,
    context: &UiRuntimeEventAdapterContext,
    event: ZrRuntimeEventV1,
) -> Result<UiInputDispatchResult, RuntimeUiManagerError> {
    let pump_event = unsafe { runtime_abi_event_to_window_input_pump_event(context, event) }?;
    manager
        .dispatch_window_input_pump_event(pump_event)
        .map_err(RuntimeUiManagerError::from)
}

/// 转换和输入处理均逐项提交；错误索引对应原 ABI 事件，前缀状态保留供后续断言核对。
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
