use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundError, SoundMixerGraph, SoundSourceId,
    SoundTrackMeter,
};

use crate::automation::validate_automation_binding;
use crate::descriptor_validation::validate_source_descriptor_for_graph;
use crate::dynamic_events::validate_dynamic_event_catalog;
use crate::engine::validation::validate_graph;
use crate::engine::{SoundEngineState, SourceVoice};

/// Imports a serialized mixer graph into the live engine state.
///
/// The neutral graph stores authoring DTOs, while the engine renders from
/// runtime registries with cursors and state. This function is the single
/// cutover point that converts graph-owned sources and automation bindings into
/// those registries.
pub(crate) fn configure_mixer_graph(
    state: &mut SoundEngineState,
    graph: SoundMixerGraph,
) -> Result<(), SoundError> {
    validate_graph(&graph)?;
    validate_dynamic_event_catalog(&graph.dynamic_events)?;

    let sources = configured_sources(state, &graph)?;
    let automation_bindings = configured_automation_bindings(&graph)?;
    let meters = graph
        .tracks
        .iter()
        .map(|track| SoundTrackMeter::silent(track.id))
        .collect();

    state.dynamic_events = graph.dynamic_events.clone();
    state.graph = graph;
    state.sources = sources;
    state.automation_bindings = automation_bindings;
    state.effect_states.clear();
    state.track_states.clear();
    state.meters = meters;
    state.latency_frames = 0;
    Ok(())
}

fn configured_sources(
    state: &mut SoundEngineState,
    graph: &SoundMixerGraph,
) -> Result<HashMap<SoundSourceId, SourceVoice>, SoundError> {
    let mut sources = HashMap::new();
    let mut next_source_id = state.next_source_id;

    for mut descriptor in graph.sources.iter().cloned() {
        validate_source_descriptor_for_graph(state, graph, &descriptor)?;
        let source_id = descriptor.id.unwrap_or_else(|| {
            next_source_id += 1;
            SoundSourceId::new(next_source_id)
        });
        next_source_id = next_source_id.max(source_id.raw());
        descriptor.id = Some(source_id);
        if sources
            .insert(
                source_id,
                SourceVoice {
                    descriptor,
                    cursor_frame: 0,
                    cursor_position: 0.0,
                },
            )
            .is_some()
        {
            return Err(SoundError::InvalidParameter(
                "configured mixer graph contains duplicate source ids".to_string(),
            ));
        }
    }

    state.next_source_id = next_source_id;
    Ok(sources)
}

fn configured_automation_bindings(
    graph: &SoundMixerGraph,
) -> Result<HashMap<SoundAutomationBindingId, SoundAutomationBinding>, SoundError> {
    let mut automation_bindings = HashMap::new();
    for binding in graph.automation_bindings.iter().cloned() {
        validate_automation_binding(&binding)?;
        if automation_bindings.insert(binding.id, binding).is_some() {
            return Err(SoundError::InvalidParameter(
                "configured mixer graph contains duplicate automation binding ids".to_string(),
            ));
        }
    }
    Ok(automation_bindings)
}
