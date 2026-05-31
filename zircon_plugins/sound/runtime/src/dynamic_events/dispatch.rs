use zircon_runtime::core::framework::sound::{
    SoundDynamicEventDelivery, SoundDynamicEventHandlerDescriptor, SoundDynamicEventInvocation,
};

pub(crate) fn dispatch_dynamic_events(
    handlers: &[SoundDynamicEventHandlerDescriptor],
    pending: &mut Vec<SoundDynamicEventInvocation>,
) -> Vec<SoundDynamicEventDelivery> {
    let pending_events = pending.drain(..).collect::<Vec<_>>();
    let mut deliveries = Vec::new();
    for invocation in pending_events {
        let mut matching_handlers = handlers
            .iter()
            .filter(|handler| handler.event_id == invocation.event_id)
            .cloned()
            .collect::<Vec<_>>();
        matching_handlers.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.plugin_id.cmp(&right.plugin_id))
                .then_with(|| left.handler_id.cmp(&right.handler_id))
        });
        deliveries.extend(
            matching_handlers
                .into_iter()
                .map(|handler| SoundDynamicEventDelivery {
                    handler,
                    invocation: invocation.clone(),
                }),
        );
    }
    deliveries
}
