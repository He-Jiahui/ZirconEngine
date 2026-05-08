use std::any::type_name;
use std::sync::{Arc, Mutex};

use crate::core::state::{NextState, OnEnter, OnExit, OnTransition, StateSpec};
use crate::core::CoreRuntime;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum GameFlow {
    Loading,
    MainMenu,
    InGame,
}

impl Default for GameFlow {
    fn default() -> Self {
        Self::Loading
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PauseMode {
    Running,
    Paused,
}

impl Default for PauseMode {
    fn default() -> Self {
        Self::Running
    }
}

#[test]
fn state_spec_initializes_current_state_and_records_initial_event() {
    let runtime = CoreRuntime::new();

    let event = runtime.init_state::<GameFlow>();

    assert_eq!(
        <GameFlow as StateSpec>::state_name(),
        type_name::<GameFlow>()
    );
    assert_eq!(
        runtime.state::<GameFlow>().unwrap().get(),
        &GameFlow::Loading
    );
    assert_eq!(runtime.next_state::<GameFlow>(), NextState::Unchanged);
    assert_eq!(event.exited, None);
    assert_eq!(event.entered, Some(GameFlow::Loading));
    assert!(event.allow_same_state_transitions);
    assert_eq!(runtime.state_transition_events::<GameFlow>(), vec![event]);
}

#[test]
fn next_state_applies_pending_transition_once_and_resets_queue() {
    let runtime = CoreRuntime::new();
    runtime.init_state::<GameFlow>();

    runtime.set_next_state(GameFlow::InGame);
    assert_eq!(
        runtime.next_state::<GameFlow>(),
        NextState::Pending(GameFlow::InGame)
    );

    let event = runtime.apply_state_transition::<GameFlow>().unwrap();

    assert_eq!(event.exited, Some(GameFlow::Loading));
    assert_eq!(event.entered, Some(GameFlow::InGame));
    assert!(event.allow_same_state_transitions);
    assert_eq!(
        runtime.state::<GameFlow>().unwrap().get(),
        &GameFlow::InGame
    );
    assert_eq!(runtime.next_state::<GameFlow>(), NextState::Unchanged);
    assert!(runtime.apply_state_transition::<GameFlow>().is_none());
}

#[test]
fn set_if_neq_suppresses_identity_transitions_but_set_keeps_them_explicit() {
    let runtime = CoreRuntime::new();
    runtime.init_state::<GameFlow>();

    runtime.set_next_state_if_neq(GameFlow::Loading);
    assert_eq!(runtime.apply_state_transition::<GameFlow>(), None);
    assert_eq!(runtime.state_transition_events::<GameFlow>().len(), 1);

    runtime.set_next_state(GameFlow::Loading);
    let event = runtime.apply_state_transition::<GameFlow>().unwrap();

    assert_eq!(event.exited, Some(GameFlow::Loading));
    assert_eq!(event.entered, Some(GameFlow::Loading));
    assert!(event.allow_same_state_transitions);
    assert_eq!(runtime.state_transition_events::<GameFlow>().len(), 2);
}

#[test]
fn transition_hooks_run_exit_transition_enter_order_for_matching_labels() {
    let runtime = CoreRuntime::new();
    let events = Arc::new(Mutex::new(Vec::new()));

    let on_exit = events.clone();
    runtime.register_on_exit(OnExit::new(GameFlow::Loading), move |_| {
        on_exit.lock().unwrap().push("exit-loading");
    });
    let on_transition = events.clone();
    runtime.register_on_transition(
        OnTransition::new(GameFlow::Loading, GameFlow::InGame),
        move |_| {
            on_transition
                .lock()
                .unwrap()
                .push("transition-loading-ingame");
        },
    );
    let on_enter = events.clone();
    runtime.register_on_enter(OnEnter::new(GameFlow::InGame), move |_| {
        on_enter.lock().unwrap().push("enter-ingame");
    });

    runtime.init_state::<GameFlow>();
    runtime.set_next_state(GameFlow::InGame);

    let event = runtime.apply_state_transition::<GameFlow>().unwrap();

    assert_eq!(event.exited, Some(GameFlow::Loading));
    assert_eq!(event.entered, Some(GameFlow::InGame));
    assert_eq!(
        *events.lock().unwrap(),
        vec!["exit-loading", "transition-loading-ingame", "enter-ingame"]
    );
}

#[test]
fn independent_state_specs_keep_events_and_current_values_separate() {
    let runtime = CoreRuntime::new();

    runtime.init_state::<GameFlow>();
    runtime.init_state::<PauseMode>();
    runtime.set_next_state(GameFlow::MainMenu);
    runtime.set_next_state(PauseMode::Paused);

    let flow_event = runtime.apply_state_transition::<GameFlow>().unwrap();
    let pause_event = runtime.apply_state_transition::<PauseMode>().unwrap();

    assert_eq!(flow_event.exited, Some(GameFlow::Loading));
    assert_eq!(flow_event.entered, Some(GameFlow::MainMenu));
    assert_eq!(pause_event.exited, Some(PauseMode::Running));
    assert_eq!(pause_event.entered, Some(PauseMode::Paused));
    assert_eq!(
        runtime.state::<GameFlow>().unwrap().get(),
        &GameFlow::MainMenu
    );
    assert_eq!(
        runtime.state::<PauseMode>().unwrap().get(),
        &PauseMode::Paused
    );
    assert_eq!(runtime.state_transition_events::<GameFlow>().len(), 2);
    assert_eq!(runtime.state_transition_events::<PauseMode>().len(), 2);
}
