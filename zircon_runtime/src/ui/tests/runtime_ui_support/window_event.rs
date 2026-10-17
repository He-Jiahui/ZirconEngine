//! 运行时 ABI 事件先转换为窗口泵事件，再交测试管理器；转换和分发失败都按原批次索引上报。

use zircon_runtime_interface::ui::{
    dispatch::UiInputDispatchResult,
    window::{runtime_abi_event_to_window_input_pump_event, UiRuntimeEventAdapterContext},
};
use zircon_runtime_interface::ZrRuntimeEventV1;

use super::runtime_ui_manager::RuntimeUiManager;
use super::runtime_ui_manager_error::RuntimeUiManagerError;

/// 同步转换原始 ABI 事件，再交当前测试宿主处理。
///
/// # 参数与约束
/// `context` 提供窗口和时间身份；按值的 `event` 不拥有其载荷字节。
/// 非空载荷必须处于同一有效分配，完整范围已初始化，并在转换返回前可读且不变；
/// 事件形状和 ABI 数值校验不证明原指针有效。具体前提见
/// [`runtime_abi_event_to_window_input_pump_event`]。
///
/// # 返回与错误
/// 适配失败时不进入分发；之后的处理或重建报错，不恢复此前已发生的状态修改。
pub(super) fn dispatch_runtime_event(
    manager: &mut RuntimeUiManager,
    context: &UiRuntimeEventAdapterContext,
    event: ZrRuntimeEventV1,
) -> Result<UiInputDispatchResult, RuntimeUiManagerError> {
    // TODO: [CR-ZCA-R072-0002] 安全 helper 尚未落实非空原始载荷的有效性，已核调用只传空载荷。
    // 扩展前须核实存储所有者、完整字节范围、初始化及转换期间的只读有效期，
    // 或改用已验证的有效切片入口；当前版本和形状检查不能替代这些前提。
    let pump_event = unsafe { runtime_abi_event_to_window_input_pump_event(context, event) }?;
    manager
        .dispatch_window_input_pump_event(pump_event)
        .map_err(RuntimeUiManagerError::from)
}

/// 转换和输入处理均逐项提交；错误索引对应原 ABI 事件，前缀状态保留供后续断言核对。
///
/// 同一 `context` 复用于所有事件，时间和序号不会由本 helper 自动递增。
/// 原始载荷的有效性约束逐项适用；全部成功才返回有序结果向量。
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
