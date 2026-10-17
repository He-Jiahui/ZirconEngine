use crate::{
    ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1,
};

use super::{
    runtime_event_to_window_input_pump_event, UiRuntimeEvent, UiRuntimeEventAdapterContext,
    UiRuntimeEventAdapterError, UiRuntimeEventAdapterResult, UiWindowInputPumpBatch,
    UiWindowInputPumpEvent,
};

/// 在原始 ABI 边界同步建立载荷借用并生成拥有数据的泵事件。
/// Converts a raw host event at the ABI boundary without retaining its payload.
///
/// # Safety
///
/// Every nonempty payload that passes carrier shape and size checks must point to
/// initialized bytes that remain readable and immutable until this call returns.
/// 进入非空载荷借用分支时，完整字节范围还须位于同一有效分配，地址加长度不得回绕。
///
/// # 参数与返回
/// `context` 提供输出身份与时间序号，`event` 提供固定 ABI 数值和调用期载荷。
/// 先检查 ABI 版本，再建立借用；载荷形状错误按事件种类映射为文本或无障碍载荷错误。
/// 成功结果拥有输出数据，不保留原指针；转换不执行 UI 分发，也不验证 viewport 与窗口身份的对应关系。
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
    // SAFETY: 调用方为非空载荷保证同一有效分配内的完整字节范围已初始化，且同步转换期间可读、不变；
    // 转换结果拥有数据，不保留原载荷借用。ABI 与数值检查本身不证明指针有效。
    let borrowed = unsafe { UiRuntimeEvent::from_abi(event) }.map_err(|_| {
        if event.kind == ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1 {
            UiRuntimeEventAdapterError::InvalidAccessibilityPayload
        } else {
            UiRuntimeEventAdapterError::InvalidTextPayload
        }
    })?;
    runtime_event_to_window_input_pump_event(context, borrowed)
}

/// 顺序转换原始 ABI 事件，首错返回且不交付前缀批次。
/// Converts in input order and returns no prefix batch on any conversion error.
///
/// # Safety
///
/// Each raw event must satisfy [`runtime_abi_event_to_window_input_pump_event`]'s
/// payload requirements through its conversion.
///
/// # 参数、返回与约束
/// 同一 `context` 用于所有元素，时间和序号不自动递增；迭代器下界只用于预留容量。
/// 首个转换错误使局部批次被丢弃，不交付前缀；已消费的迭代器元素及其自身副作用不回滚。
/// 成功结果按输入顺序拥有数据，不合并重绘，也不调用分发器。
pub unsafe fn runtime_abi_events_to_window_input_pump_batch(
    context: &UiRuntimeEventAdapterContext,
    events: impl IntoIterator<Item = ZrRuntimeEventV1>,
) -> UiRuntimeEventAdapterResult<UiWindowInputPumpBatch> {
    let events = events.into_iter();
    let mut batch = UiWindowInputPumpBatch::with_capacity(events.size_hint().0);
    for event in events {
        // SAFETY: 调用方须为当前元素分别满足单事件入口的载荷前提；
        // 此处同步完成拥有数据的转换，局部结果不保留该元素的载荷借用。
        batch.push(unsafe { runtime_abi_event_to_window_input_pump_event(context, event)? });
    }
    Ok(batch)
}
