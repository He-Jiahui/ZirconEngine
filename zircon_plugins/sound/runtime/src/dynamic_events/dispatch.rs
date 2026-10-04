//! 分发先计算容量，再按事件和已排序处理器生成交付快照；同一事件的最后一个接收者可直接取得原调用。
use zircon_runtime::core::framework::sound::{
    SoundDynamicEventDelivery, SoundDynamicEventHandlerDescriptor, SoundDynamicEventInvocation,
};

use super::handlers::DynamicEventHandlerRegistry;

pub(crate) fn dispatch_dynamic_events(
    handlers: &DynamicEventHandlerRegistry,
    pending: &mut Vec<SoundDynamicEventInvocation>,
) -> Vec<SoundDynamicEventDelivery> {
    let delivery_capacity = pending.iter().fold(0_usize, |count, invocation| {
        count.saturating_add(
            handlers
                .indices_for_event(invocation.event_id.as_str())
                .map_or(0, <[usize]>::len),
        )
    });
    let mut deliveries = Vec::with_capacity(delivery_capacity);
    // 交付结果按队列顺序固定；处理器执行可能失败，但已出队事件不会自动重放。
    for invocation in pending.drain(..) {
        let Some((last_handler_index, leading_handler_indices)) = handlers
            .indices_for_event(invocation.event_id.as_str())
            .and_then(<[usize]>::split_last)
        else {
            continue;
        };
        deliveries.extend(leading_handler_indices.iter().map(|handler_index| {
            SoundDynamicEventDelivery {
                handler: handlers.handler(*handler_index).clone(),
                invocation: invocation.clone(),
            }
        }));
        deliveries.push(SoundDynamicEventDelivery {
            handler: handlers.handler(*last_handler_index).clone(),
            invocation,
        });
    }
    deliveries
}

#[cfg(test)]
#[path = "tests/dispatch.rs"]
mod tests;
