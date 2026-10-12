//! Runtime-wide finite-state machine contracts and transition hooks.

#[path = "hook.rs"]
mod hook;
#[path = "hook_index.rs"]
mod hook_index;
#[path = "machine.rs"]
mod machine;
#[path = "next_state.rs"]
mod next_state;
#[path = "on_enter.rs"]
mod on_enter;
#[path = "on_exit.rs"]
mod on_exit;
#[path = "on_transition.rs"]
mod on_transition;
#[path = "registry.rs"]
mod registry;
#[path = "state.rs"]
mod state;
#[path = "state_spec.rs"]
mod state_spec;
#[path = "state_transition_event.rs"]
mod state_transition_event;

pub use next_state::NextState;
pub use on_enter::OnEnter;
pub use on_exit::OnExit;
pub use on_transition::OnTransition;
pub use state::State;
pub use state_spec::StateSpec;
pub use state_transition_event::StateTransitionEvent;

pub(crate) use hook_index::StateHookIndex;
pub(crate) use registry::StateRegistry;
