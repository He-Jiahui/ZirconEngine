use std::collections::HashSet;

use zircon_runtime::core::framework::sound::SoundMixerGraph;

use crate::engine::SoundEngineState;

pub(crate) fn retain_dynamic_event_runtime_state(
    state: &mut SoundEngineState,
    graph: &SoundMixerGraph,
) {
    let dynamic_event_ids = graph
        .dynamic_events
        .events
        .iter()
        .map(|event| event.id.clone())
        .collect::<HashSet<_>>();
    state.dynamic_events = graph.dynamic_events.clone();
    state
        .dynamic_event_handlers
        .retain(|handler| dynamic_event_ids.contains(&handler.event_id));
    let dynamic_event_handler_keys = state
        .dynamic_event_handlers
        .iter()
        .map(|handler| (handler.plugin_id.clone(), handler.handler_id.clone()))
        .collect::<HashSet<_>>();
    state.dynamic_event_executors.retain(|key, _| {
        dynamic_event_handler_keys.contains(&(key.plugin_id.clone(), key.handler_id.clone()))
    });
    state
        .pending_dynamic_events
        .retain(|event| dynamic_event_ids.contains(&event.event_id));
}
