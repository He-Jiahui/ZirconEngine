use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundError, SoundMixerGraph, SoundSourceId,
    SoundTrackMeter,
};

use crate::dynamic_events::catalog::validate_dynamic_event_catalog;
use crate::engine::{SoundEngineState, SourceVoice};

use super::automation::configured_automation_bindings;
use super::dynamic_events::retain_dynamic_event_runtime_state;
use super::runtime_state::reset_mixer_runtime_state;
use super::sources::configured_sources;
use super::timeline::retain_timeline_sequences_for_automation_bindings;

/// Imports a serialized mixer graph into the live engine state.
///
/// The neutral graph stores authoring DTOs, while the engine renders from
/// runtime registries with cursors and state. This function is the cutover
/// point that validates the graph and delegates each runtime registry rebuild
/// to its owning configuration stage.
pub(crate) struct PreparedMixerConfiguration {
    pub(crate) graph: SoundMixerGraph,
    pub(crate) sources: HashMap<SoundSourceId, SourceVoice>,
    pub(crate) next_source_id: u64,
    pub(crate) automation_bindings: HashMap<SoundAutomationBindingId, SoundAutomationBinding>,
    pub(crate) meters: Vec<SoundTrackMeter>,
}

pub(crate) fn prepare_mixer_configuration(
    state: &SoundEngineState,
    graph: &SoundMixerGraph,
) -> Result<PreparedMixerConfiguration, SoundError> {
    validate_dynamic_event_catalog(&graph.dynamic_events)?;

    let (sources, next_source_id, source_descriptors) = configured_sources(state, graph)?;
    let automation_bindings = configured_automation_bindings(graph)?;
    let meters = graph
        .tracks
        .iter()
        .map(|track| SoundTrackMeter::silent(track.id))
        .collect();

    let mut graph = graph.clone();
    graph.sources = source_descriptors;

    Ok(PreparedMixerConfiguration {
        graph,
        sources,
        next_source_id,
        automation_bindings,
        meters,
    })
}

pub(crate) fn commit_mixer_configuration(
    state: &mut SoundEngineState,
    graph: SoundMixerGraph,
    next_source_id: u64,
    automation_bindings: HashMap<SoundAutomationBindingId, SoundAutomationBinding>,
    meters: Vec<SoundTrackMeter>,
) {
    retain_dynamic_event_runtime_state(state, &graph);
    state.replace_graph(graph);
    state.next_source_id = next_source_id;
    state.automation_bindings = automation_bindings;
    retain_timeline_sequences_for_automation_bindings(state);
    reset_mixer_runtime_state(state, meters);
}
