//! Runtime-wide finite-state machine contracts and transition hooks.

mod hook;
mod machine;
mod next_state;
mod on_enter;
mod on_exit;
mod on_transition;
mod registry;
mod state;
mod state_spec;
mod state_transition_event;

pub use next_state::NextState;
pub use on_enter::OnEnter;
pub use on_exit::OnExit;
pub use on_transition::OnTransition;
pub use state::State;
pub use state_spec::StateSpec;
pub use state_transition_event::StateTransitionEvent;

pub(crate) use registry::StateRegistry;
