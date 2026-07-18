use std::sync::Mutex;

use zircon_runtime::core::framework::sound::{SoundError, SoundMixerGraph};

use crate::engine::SoundEngineState;
use crate::kira_bridge::{compile_graph_update, validate_graph};
use crate::poison_recovery::lock_recover;

#[cfg(test)]
use std::{cell::Cell, fmt::Debug, sync::Arc, time::Duration};

#[cfg(test)]
use kira::backend::Backend;

#[cfg(test)]
use crate::kira_bridge::KiraEngine;

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
        let plan = if snapshot.kira_active {
            Some(compile_graph_update(&snapshot.graph, &graph)?)
        } else {
            validate_graph(&graph)?;
            None
        };
        if commit_graph_attempt(
            manager,
            snapshot.revision,
            snapshot.kira_active,
            graph,
            plan,
            &commit,
        )? {
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
        let plan = if snapshot.kira_active {
            Some(compile_graph_update(&snapshot.graph, &graph)?)
        } else {
            validate_graph(&graph)?;
            None
        };
        if commit_graph_attempt(
            manager,
            snapshot.revision,
            snapshot.kira_active,
            graph.clone(),
            plan,
            &commit,
        )? {
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
    expected_kira_active: bool,
    graph: SoundMixerGraph,
    plan: Option<crate::kira_bridge::GraphSyncPlan>,
    commit: &Commit,
) -> Result<bool, SoundError>
where
    Commit: Fn(&mut SoundEngineState),
{
    commit_graph_state_attempt(
        &manager.state,
        expected_revision,
        expected_kira_active,
        graph,
        plan,
        |state| state.graph_revision,
        |state| state.kira.is_active(),
        |state, graph, plan| state.kira.apply_graph_update(graph, plan),
        |state, graph| state.replace_graph(graph),
        commit,
    )
}

#[allow(clippy::too_many_arguments)]
fn commit_graph_state_attempt<State, Revision, Active, Apply, Replace, Commit>(
    state: &Mutex<State>,
    expected_revision: u64,
    expected_kira_active: bool,
    graph: SoundMixerGraph,
    plan: Option<crate::kira_bridge::GraphSyncPlan>,
    revision: Revision,
    active: Active,
    apply: Apply,
    replace: Replace,
    commit: &Commit,
) -> Result<bool, SoundError>
where
    Revision: Fn(&State) -> u64,
    Active: Fn(&State) -> bool,
    Apply: Fn(
        &mut State,
        &SoundMixerGraph,
        crate::kira_bridge::GraphSyncPlan,
    ) -> Result<(), SoundError>,
    Replace: Fn(&mut State, SoundMixerGraph),
    Commit: Fn(&mut State),
{
    let mut state = lock_recover(state);
    if revision(&state) != expected_revision || active(&state) != expected_kira_active {
        return Ok(false);
    }
    #[cfg(test)]
    let lock_started = std::time::Instant::now();
    if let Some(plan) = plan {
        apply(&mut state, &graph, plan)?;
    }
    replace(&mut state, graph);
    commit(&mut state);
    #[cfg(test)]
    LAST_GRAPH_COMMIT_LOCK_HOLD.with(|hold| hold.set(Some(lock_started.elapsed())));
    Ok(true)
}

#[cfg(test)]
pub(crate) fn last_graph_commit_lock_hold_for_test() -> Option<Duration> {
    LAST_GRAPH_COMMIT_LOCK_HOLD.with(Cell::get)
}

#[cfg(test)]
pub(crate) struct ActiveGraphCommitHarness<B: Backend> {
    state: Mutex<ActiveGraphCommitState<B>>,
}

#[cfg(test)]
struct ActiveGraphCommitState<B: Backend> {
    revision: u64,
    graph: Arc<SoundMixerGraph>,
    kira: KiraEngine<B>,
}

#[cfg(test)]
impl<B> ActiveGraphCommitHarness<B>
where
    B: Backend,
    B::Error: Debug,
{
    pub(crate) fn new(kira: KiraEngine<B>, graph: SoundMixerGraph) -> Self {
        Self {
            state: Mutex::new(ActiveGraphCommitState {
                revision: 0,
                graph: Arc::new(graph),
                kira,
            }),
        }
    }

    pub(crate) fn replace_graph(&self, graph: SoundMixerGraph) -> Result<Duration, SoundError> {
        for _ in 0..GRAPH_MUTATION_RETRY_LIMIT {
            let (revision, kira_active, current) = {
                let state = lock_recover(&self.state);
                (
                    state.revision,
                    state.kira.is_active(),
                    Arc::clone(&state.graph),
                )
            };
            let plan = if kira_active {
                Some(compile_graph_update(&current, &graph)?)
            } else {
                validate_graph(&graph)?;
                None
            };
            if commit_graph_state_attempt(
                &self.state,
                revision,
                kira_active,
                graph.clone(),
                plan,
                |state| state.revision,
                |state| state.kira.is_active(),
                |state, graph, plan| state.kira.apply_graph_update(graph, plan),
                |state, graph| {
                    state.graph = Arc::new(graph);
                    state.revision = state.revision.wrapping_add(1);
                },
                &|_| {},
            )? {
                return last_graph_commit_lock_hold_for_test().ok_or_else(|| {
                    SoundError::InvalidMixerGraph(
                        "active graph commit did not record its lock hold".to_string(),
                    )
                });
            }
        }
        Err(SoundError::InvalidMixerGraph(
            "active graph commit harness exceeded the retry limit".to_string(),
        ))
    }

    pub(crate) fn with_kira_mut<T>(&self, operation: impl FnOnce(&mut KiraEngine<B>) -> T) -> T {
        operation(&mut lock_recover(&self.state).kira)
    }
}
