use std::collections::HashMap;

use kira::backend::Backend;
use zircon_runtime::core::framework::sound::{SoundError, SoundMixerGraph, SoundSourceId};

use crate::engine::{SoundEngineState, SourceVoice};
use crate::kira_bridge::{compile_graph_update, validate_graph, KiraEngine};
use crate::mixer_configuration::configure::{
    commit_mixer_configuration, prepare_mixer_configuration, PreparedMixerConfiguration,
};
use crate::poison_recovery::lock_recover;
use crate::service_types::sources::{stop_bound_source, sync_preconfigured_sources};

use super::super::DefaultSoundManager;

const CONFIGURE_RETRY_LIMIT: usize = 8;

impl DefaultSoundManager {
    pub(in crate::service_types) fn configure_mixer_impl(
        &self,
        graph: SoundMixerGraph,
    ) -> Result<(), SoundError> {
        for _ in 0..CONFIGURE_RETRY_LIMIT {
            let snapshot = lock_recover(&self.state).graph_snapshot();
            let graph_plan = if snapshot.kira_active {
                Some(compile_graph_update(&snapshot.graph, &graph)?)
            } else {
                validate_graph(&graph)?;
                None
            };

            let mut state = lock_recover(&self.state);
            if state.graph_revision != snapshot.revision
                || state.kira.is_active() != snapshot.kira_active
            {
                continue;
            }
            let prepared = prepare_mixer_configuration(&state, &graph)?;
            let previous_graph = state.graph.clone();
            let was_active = snapshot.kira_active;
            let PreparedMixerConfiguration {
                graph,
                sources,
                next_source_id,
                automation_bindings,
                meters,
            } = prepared;
            if let Some(graph_plan) = graph_plan {
                state.kira.apply_graph_update(&graph, graph_plan)?;
            }

            let mut previous_sources = std::mem::take(&mut state.sources);
            if let Err(error) = stop_source_bindings(&mut state.kira, &mut previous_sources) {
                return rollback_configuration(
                    &mut state,
                    &previous_graph,
                    previous_sources,
                    error,
                    was_active,
                );
            }
            state.sources = sources;
            if was_active {
                if let Err(error) = sync_preconfigured_sources(&mut state) {
                    return rollback_configuration(
                        &mut state,
                        &previous_graph,
                        previous_sources,
                        error,
                        was_active,
                    );
                }
            }
            commit_mixer_configuration(
                &mut state,
                graph,
                next_source_id,
                automation_bindings,
                meters,
            );
            return Ok(());
        }

        Err(SoundError::InvalidMixerGraph(
            "concurrent mixer configuration exceeded the retry limit".to_string(),
        ))
    }
}

fn rollback_configuration(
    state: &mut SoundEngineState,
    previous_graph: &SoundMixerGraph,
    mut previous_sources: HashMap<SoundSourceId, SourceVoice>,
    cause: SoundError,
    was_active: bool,
) -> Result<(), SoundError> {
    let mut rollback_error = None;
    let mut current_sources = std::mem::take(&mut state.sources);
    record_first_error(
        &mut rollback_error,
        stop_source_bindings(&mut state.kira, &mut current_sources),
    );
    record_first_error(
        &mut rollback_error,
        stop_source_bindings(&mut state.kira, &mut previous_sources),
    );
    if was_active {
        record_first_error(&mut rollback_error, state.kira.sync_graph(previous_graph));
    }
    state.sources = previous_sources;
    if was_active && rollback_error.is_none() {
        record_first_error(&mut rollback_error, sync_preconfigured_sources(state));
    }

    if let Some(rollback_error) = rollback_error {
        state.deactivate_kira();
        return Err(SoundError::BackendUnavailable {
            detail: format!(
                "mixer configuration failed ({cause}); rollback failed ({rollback_error})"
            ),
        });
    }
    Err(cause)
}

fn record_first_error(first: &mut Option<SoundError>, result: Result<(), SoundError>) {
    if first.is_none() {
        *first = result.err();
    }
}

fn stop_source_bindings<B: Backend>(
    kira: &mut KiraEngine<B>,
    sources: &mut HashMap<SoundSourceId, SourceVoice>,
) -> Result<(), SoundError> {
    for voice in sources.values_mut() {
        stop_bound_source(kira, voice)?;
    }
    Ok(())
}
