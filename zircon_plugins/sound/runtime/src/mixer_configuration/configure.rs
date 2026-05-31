use zircon_runtime::core::framework::sound::{SoundError, SoundMixerGraph, SoundTrackMeter};

use crate::dynamic_events::catalog::validate_dynamic_event_catalog;
use crate::engine::validation::validate_graph;
use crate::engine::SoundEngineState;

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

    retain_dynamic_event_runtime_state(state, &graph);
    state.graph = graph;
    state.sources = sources;
    state.automation_bindings = automation_bindings;
    retain_timeline_sequences_for_automation_bindings(state);
    reset_mixer_runtime_state(state, meters);
    Ok(())
}
