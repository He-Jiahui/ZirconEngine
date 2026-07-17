use zircon_runtime::core::framework::sound::{SoundError, SoundMixerGraph};

use crate::engine::SoundEngineState;
use crate::kira_bridge::compile_graph_update;
use crate::poison_recovery::lock_recover;

#[cfg(test)]
use std::{cell::Cell, time::Duration};

use super::super::DefaultSoundManager;

const GRAPH_MUTATION_RETRY_LIMIT: usize = 8;

#[cfg(test)]
thread_local! {
    static LAST_GRAPH_COMMIT_LOCK_HOLD: Cell<Option<Duration>> = const { Cell::new(None) };
}

pub(crate) fn mutate_graph<Mutate, Commit>(
    manager: &DefaultSoundManager,
    mutate: Mutate,
    commit: Commit,
) -> Result<(), SoundError>
where
    Mutate: Fn(&mut SoundMixerGraph) -> Result<(), SoundError>,
    Commit: Fn(&mut SoundEngineState),
{
    for _ in 0..GRAPH_MUTATION_RETRY_LIMIT {
        let snapshot = lock_recover(&manager.state).graph_snapshot();
        let mut graph = (*snapshot.graph).clone();
        mutate(&mut graph)?;
        let plan = compile_graph_update(&snapshot.graph, &graph)?;
        if commit_graph_attempt(manager, snapshot.revision, graph, plan, &commit)? {
            return Ok(());
        }
    }

    Err(SoundError::InvalidMixerGraph(
        "concurrent mixer graph mutation exceeded the retry limit".to_string(),
    ))
}

pub(crate) fn replace_graph<Commit>(
    manager: &DefaultSoundManager,
    graph: SoundMixerGraph,
    commit: Commit,
) -> Result<(), SoundError>
where
    Commit: Fn(&mut SoundEngineState),
{
    for _ in 0..GRAPH_MUTATION_RETRY_LIMIT {
        let snapshot = lock_recover(&manager.state).graph_snapshot();
        let plan = compile_graph_update(&snapshot.graph, &graph)?;
        if commit_graph_attempt(manager, snapshot.revision, graph.clone(), plan, &commit)? {
            return Ok(());
        }
    }

    Err(SoundError::InvalidMixerGraph(
        "concurrent mixer graph replacement exceeded the retry limit".to_string(),
    ))
}

fn commit_graph_attempt<Commit>(
    manager: &DefaultSoundManager,
    expected_revision: u64,
    graph: SoundMixerGraph,
    plan: crate::kira_bridge::GraphSyncPlan,
    commit: &Commit,
) -> Result<bool, SoundError>
where
    Commit: Fn(&mut SoundEngineState),
{
    let mut state = lock_recover(&manager.state);
    if state.graph_revision != expected_revision {
        return Ok(false);
    }
    #[cfg(test)]
    let lock_started = std::time::Instant::now();
    if state.kira.is_active() {
        state.kira.apply_graph_update(&graph, plan)?;
    }
    state.replace_graph(graph);
    commit(&mut state);
    #[cfg(test)]
    LAST_GRAPH_COMMIT_LOCK_HOLD.with(|hold| hold.set(Some(lock_started.elapsed())));
    Ok(true)
}

#[cfg(test)]
pub(crate) fn last_graph_commit_lock_hold_for_test() -> Option<Duration> {
    LAST_GRAPH_COMMIT_LOCK_HOLD.with(Cell::get)
}
