//! Lifecycle state transitions and their typed diagnostics.

use std::fmt;

use super::super::phases::EditorPluginLoadingPhase;
use super::super::sdk::lifecycle::EditorPluginLifecycleStage;

/// The manager-owned activation state for a discovered editor plugin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditorPluginState {
    Discovered,
    Validated,
    Loading,
    Active,
    Revoking,
    Disabled,
    Faulted,
}

impl EditorPluginState {
    /// Returns whether a lifecycle executor may publish the requested next state.
    pub fn can_transition_to(self, next: Self) -> bool {
        match self {
            Self::Discovered => matches!(next, Self::Validated | Self::Disabled | Self::Faulted),
            Self::Validated => matches!(next, Self::Loading | Self::Disabled | Self::Faulted),
            Self::Loading => matches!(next, Self::Active | Self::Faulted),
            Self::Active => matches!(next, Self::Revoking | Self::Faulted),
            Self::Revoking => matches!(next, Self::Disabled | Self::Faulted),
            Self::Disabled => matches!(next, Self::Validated | Self::Faulted),
            Self::Faulted => matches!(next, Self::Validated),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditorPluginTransitionError {
    UnknownPlugin {
        package_id: String,
    },
    DuplicateProjectSelection {
        package_id: String,
    },
    InvalidEnablement {
        package_id: String,
        state: EditorPluginState,
        enabled: bool,
    },
    InvalidTransition {
        package_id: String,
        from: EditorPluginState,
        to: EditorPluginState,
    },
    InvalidLoadingPhaseAdvance {
        reached: EditorPluginLoadingPhase,
        requested: EditorPluginLoadingPhase,
    },
    LoadingPhaseUnavailable {
        package_id: String,
        loading_phase: EditorPluginLoadingPhase,
        reached: Option<EditorPluginLoadingPhase>,
    },
    ManagedLifecycleTransitionRequired {
        package_id: String,
        requested: EditorPluginState,
    },
    ManagedLifecycleEventReserved {
        package_id: String,
        stage: EditorPluginLifecycleStage,
    },
    ManagedLifecycleBroadcastReserved {
        stage: EditorPluginLifecycleStage,
    },
    DisabledLifecycleRetryRequired {
        package_id: String,
    },
    MutationInProgress,
}

impl fmt::Display for EditorPluginTransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPlugin { package_id } => {
                write!(formatter, "editor plugin `{package_id}` is not discovered")
            }
            Self::DuplicateProjectSelection { package_id } => write!(
                formatter,
                "project manifest selects editor plugin `{package_id}` more than once"
            ),
            Self::InvalidEnablement {
                package_id,
                state,
                enabled,
            } => write!(
                formatter,
                "editor plugin `{package_id}` cannot be {} from {state:?}",
                if *enabled { "enabled" } else { "disabled" }
            ),
            Self::InvalidTransition {
                package_id,
                from,
                to,
            } => write!(
                formatter,
                "editor plugin `{package_id}` cannot transition from {from:?} to {to:?}"
            ),
            Self::InvalidLoadingPhaseAdvance { reached, requested } => write!(
                formatter,
                "editor plugin loading phase cannot move backward from {reached:?} to {requested:?}"
            ),
            Self::LoadingPhaseUnavailable {
                package_id,
                loading_phase,
                reached,
            } => write!(
                formatter,
                "editor plugin `{package_id}` requires {loading_phase:?}, but the reached phase is {reached:?}"
            ),
            Self::ManagedLifecycleTransitionRequired {
                package_id,
                requested,
            } => write!(
                formatter,
                "editor plugin `{package_id}` must reach {requested:?} through manager lifecycle scheduling"
            ),
            Self::ManagedLifecycleEventReserved { package_id, stage } => write!(
                formatter,
                "editor plugin `{package_id}` must reach lifecycle stage {stage:?} through manager scheduling"
            ),
            Self::ManagedLifecycleBroadcastReserved { stage } => write!(
                formatter,
                "editor plugin lifecycle broadcast cannot dispatch manager-owned stage {stage:?}"
            ),
            Self::DisabledLifecycleRetryRequired { package_id } => write!(
                formatter,
                "editor plugin `{package_id}` must retry its failed disabled lifecycle callback before another transition"
            ),
            Self::MutationInProgress => {
                formatter.write_str("editor plugin manager is dispatching a lifecycle mutation")
            }
        }
    }
}

impl std::error::Error for EditorPluginTransitionError {}
