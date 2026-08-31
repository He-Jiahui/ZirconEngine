use std::sync::{Arc, Mutex};

use crate::core::runtime::state_machine::{OnEnter, OnExit, OnTransition};
use crate::core::CoreRuntime;

use super::GameFlow;

#[test]
fn state_hook_index_dispatches_only_matching_buckets_in_registration_order() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));

    for (state, label) in [
        (GameFlow::MainMenu, "exit-main-menu"),
        (GameFlow::Loading, "exit-loading-1"),
        (GameFlow::Loading, "exit-loading-2"),
    ] {
        let calls = Arc::clone(&calls);
        runtime.register_on_exit(OnExit::new(state), move |_| {
            calls.lock().unwrap().push(label);
        });
    }

    for (exited, entered, label) in [
        (
            GameFlow::MainMenu,
            GameFlow::InGame,
            "transition-main-menu-ingame",
        ),
        (
            GameFlow::Loading,
            GameFlow::InGame,
            "transition-loading-ingame-1",
        ),
        (
            GameFlow::Loading,
            GameFlow::InGame,
            "transition-loading-ingame-2",
        ),
    ] {
        let calls = Arc::clone(&calls);
        runtime.register_on_transition(OnTransition::new(exited, entered), move |_| {
            calls.lock().unwrap().push(label);
        });
    }

    for (state, label) in [
        (GameFlow::MainMenu, "enter-main-menu"),
        (GameFlow::InGame, "enter-ingame-1"),
        (GameFlow::InGame, "enter-ingame-2"),
    ] {
        let calls = Arc::clone(&calls);
        runtime.register_on_enter(OnEnter::new(state), move |_| {
            calls.lock().unwrap().push(label);
        });
    }

    runtime.init_state::<GameFlow>();
    runtime.set_next_state(GameFlow::InGame);
    runtime.apply_state_transition::<GameFlow>().unwrap();

    assert_eq!(
        *calls.lock().unwrap(),
        [
            "exit-loading-1",
            "exit-loading-2",
            "transition-loading-ingame-1",
            "transition-loading-ingame-2",
            "enter-ingame-1",
            "enter-ingame-2",
        ]
    );
}

#[test]
fn state_hook_index_preserves_initial_and_identity_transition_semantics() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));

    let exit_calls = Arc::clone(&calls);
    runtime.register_on_exit(OnExit::new(GameFlow::Loading), move |_| {
        exit_calls.lock().unwrap().push("exit-loading");
    });
    let transition_calls = Arc::clone(&calls);
    runtime.register_on_transition(
        OnTransition::new(GameFlow::Loading, GameFlow::Loading),
        move |_| {
            transition_calls
                .lock()
                .unwrap()
                .push("transition-loading-loading");
        },
    );
    let enter_calls = Arc::clone(&calls);
    runtime.register_on_enter(OnEnter::new(GameFlow::Loading), move |_| {
        enter_calls.lock().unwrap().push("enter-loading");
    });

    runtime.init_state::<GameFlow>();
    assert_eq!(*calls.lock().unwrap(), ["enter-loading"]);

    calls.lock().unwrap().clear();
    runtime.set_next_state_if_neq(GameFlow::Loading);
    assert!(runtime.apply_state_transition::<GameFlow>().is_none());
    assert!(calls.lock().unwrap().is_empty());

    runtime.set_next_state(GameFlow::Loading);
    runtime.apply_state_transition::<GameFlow>().unwrap();
    assert_eq!(
        *calls.lock().unwrap(),
        [
            "exit-loading",
            "transition-loading-loading",
            "enter-loading",
        ]
    );
}

#[test]
fn state_machine_uses_canonical_hash_bucket_hook_index() {
    let machine_source = include_str!("../../core/runtime/state_machine/machine.rs");
    let hook_index_source = include_str!("../../core/runtime/state_machine/hook_index.rs");

    assert!(machine_source.contains("hooks: StateHookIndex<T>"));
    assert!(!machine_source.contains("on_enter: Vec<"));
    assert!(!machine_source.contains("on_exit: Vec<"));
    assert!(!machine_source.contains("on_transition: Vec<"));

    assert!(hook_index_source.contains("HashMap<T, Vec<StateHook<T>>>"));
    assert!(hook_index_source.contains("HashMap<T, HashMap<T, Vec<StateHook<T>>>>"));
    assert!(hook_index_source.contains(".get(entered)"));
    assert!(hook_index_source.contains(".get(exited)"));
    assert!(hook_index_source.contains(".and_then(|targets| targets.get(entered))"));
    assert!(!hook_index_source.contains(".iter()"));
    assert!(!hook_index_source.contains(".values("));
    assert!(!hook_index_source.contains(".filter("));
    assert!(!hook_index_source.contains(".filter_map("));
    assert!(!hook_index_source.contains("for hooks"));
    assert!(!hook_index_source.contains("for (_,"));
}
