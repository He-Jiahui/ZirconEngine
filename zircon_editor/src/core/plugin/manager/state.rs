//! Lifecycle state transitions and their typed diagnostics.

use std::fmt;

use super::super::catalog::EditorPluginCatalog;
use super::super::catalog_snapshot::EditorPluginCatalogSnapshot;
use super::super::phases::EditorPluginLoadingPhase;
use super::super::sdk::lifecycle::{EditorPluginLifecycleEvent, EditorPluginLifecycleStage};
use super::snapshot::EditorPluginManagerEntry;

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

pub(super) fn has_failed_disabled_lifecycle(
    catalog: &EditorPluginCatalogSnapshot,
    entry: &EditorPluginManagerEntry,
) -> bool {
    entry.state == EditorPluginState::Faulted
        && catalog.lifecycle_stage_failed(entry.package_id(), &EditorPluginLifecycleStage::Disabled)
}

pub(super) fn is_manager_owned_activation_stage(stage: &EditorPluginLifecycleStage) -> bool {
    matches!(
        stage,
        EditorPluginLifecycleStage::Loaded
            | EditorPluginLifecycleStage::Enabled
            | EditorPluginLifecycleStage::Disabled
    )
}

pub(super) fn state_after_enablement_request(
    package_id: &str,
    mut state: EditorPluginState,
    enabled: bool,
    loading_phase: EditorPluginLoadingPhase,
    reached_loading_phase: Option<EditorPluginLoadingPhase>,
) -> Result<EditorPluginState, EditorPluginTransitionError> {
    if enabled {
        while !matches!(
            state,
            EditorPluginState::Validated | EditorPluginState::Loading | EditorPluginState::Active
        ) {
            let next = match state {
                EditorPluginState::Discovered
                | EditorPluginState::Disabled
                | EditorPluginState::Faulted => EditorPluginState::Validated,
                _ => return invalid_enablement(package_id, state, enabled),
            };
            if !state.can_transition_to(next) {
                return invalid_enablement(package_id, state, enabled);
            }
            state = next;
        }
        if phase_is_reached(loading_phase, reached_loading_phase) {
            while state != EditorPluginState::Active {
                let next = match state {
                    EditorPluginState::Validated => EditorPluginState::Loading,
                    EditorPluginState::Loading => EditorPluginState::Active,
                    _ => return invalid_enablement(package_id, state, enabled),
                };
                if !state.can_transition_to(next) {
                    return invalid_enablement(package_id, state, enabled);
                }
                state = next;
            }
        }
        return Ok(state);
    }

    while state != EditorPluginState::Disabled {
        let next = match state {
            EditorPluginState::Discovered | EditorPluginState::Validated => {
                EditorPluginState::Disabled
            }
            EditorPluginState::Active => EditorPluginState::Revoking,
            EditorPluginState::Revoking => EditorPluginState::Disabled,
            EditorPluginState::Loading | EditorPluginState::Faulted => {
                return invalid_enablement(package_id, state, enabled);
            }
            EditorPluginState::Disabled => break,
        };
        if !state.can_transition_to(next) {
            return invalid_enablement(package_id, state, enabled);
        }
        state = next;
    }
    Ok(state)
}

pub(super) fn apply_enablement_request(
    catalog: &mut EditorPluginCatalog,
    entry: &mut EditorPluginManagerEntry,
    enabled: bool,
    reached_loading_phase: Option<EditorPluginLoadingPhase>,
    failed_disabled_lifecycle: bool,
) -> Result<bool, EditorPluginTransitionError> {
    validate_enablement_request(
        entry,
        enabled,
        reached_loading_phase,
        failed_disabled_lifecycle,
    )?;
    if failed_disabled_lifecycle {
        entry.state = EditorPluginState::Revoking;
        let report = catalog.record_lifecycle_event(
            entry.package_id.as_str(),
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::Disabled),
        );
        entry.state = if report.is_success() {
            EditorPluginState::Disabled
        } else {
            EditorPluginState::Faulted
        };
        return Ok(true);
    }

    let state = state_after_enablement_request(
        entry.package_id.as_str(),
        entry.state,
        enabled,
        entry.loading_phase,
        reached_loading_phase,
    )?;
    if entry.state == state {
        return Ok(false);
    }
    if enabled && state == EditorPluginState::Active {
        entry.state = EditorPluginState::Validated;
        return Ok(activate_entry(catalog, entry, reached_loading_phase));
    }
    if !enabled && entry.state == EditorPluginState::Active {
        entry.state = EditorPluginState::Revoking;
        let report = catalog.record_lifecycle_event(
            entry.package_id.as_str(),
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::Disabled),
        );
        entry.state = if report.is_success() {
            EditorPluginState::Disabled
        } else {
            EditorPluginState::Faulted
        };
        return Ok(true);
    }
    entry.state = state;
    Ok(false)
}

pub(super) fn validate_enablement_request(
    entry: &EditorPluginManagerEntry,
    enabled: bool,
    reached_loading_phase: Option<EditorPluginLoadingPhase>,
    failed_disabled_lifecycle: bool,
) -> Result<(), EditorPluginTransitionError> {
    if failed_disabled_lifecycle && enabled {
        return Err(
            EditorPluginTransitionError::DisabledLifecycleRetryRequired {
                package_id: entry.package_id.clone(),
            },
        );
    }
    if !failed_disabled_lifecycle {
        let _ = state_after_enablement_request(
            entry.package_id.as_str(),
            entry.state,
            enabled,
            entry.loading_phase,
            reached_loading_phase,
        )?;
    }
    Ok(())
}

pub(super) fn normalize_entries_for_loading_phase(
    entries: &mut [EditorPluginManagerEntry],
    reached_loading_phase: Option<EditorPluginLoadingPhase>,
) {
    for entry in entries {
        if is_phase_gated_state(entry.state)
            && !phase_is_reached(entry.loading_phase, reached_loading_phase)
        {
            entry.state = EditorPluginState::Validated;
        }
    }
}

pub(super) fn activate_eligible_entries(
    catalog: &mut EditorPluginCatalog,
    entries: &mut [EditorPluginManagerEntry],
    reached_loading_phase: Option<EditorPluginLoadingPhase>,
) -> bool {
    entries.iter_mut().fold(false, |changed, entry| {
        activate_entry(catalog, entry, reached_loading_phase) || changed
    })
}

fn activate_entry(
    catalog: &mut EditorPluginCatalog,
    entry: &mut EditorPluginManagerEntry,
    reached_loading_phase: Option<EditorPluginLoadingPhase>,
) -> bool {
    if entry.state != EditorPluginState::Validated
        || !phase_is_reached(entry.loading_phase, reached_loading_phase)
    {
        return false;
    }

    entry.state = EditorPluginState::Loading;
    if !catalog.lifecycle_stage_succeeded(
        entry.package_id.as_str(),
        &EditorPluginLifecycleStage::Loaded,
    ) {
        let report = catalog.record_lifecycle_event(
            entry.package_id.as_str(),
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::Loaded),
        );
        if !report.is_success() {
            entry.state = EditorPluginState::Faulted;
            return true;
        }
    }
    let report = catalog.record_lifecycle_event(
        entry.package_id.as_str(),
        EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::Enabled),
    );
    entry.state = if report.is_success() {
        EditorPluginState::Active
    } else {
        EditorPluginState::Faulted
    };
    true
}

pub(super) fn is_phase_gated_state(state: EditorPluginState) -> bool {
    matches!(
        state,
        EditorPluginState::Loading | EditorPluginState::Active
    )
}

pub(super) fn phase_is_reached(
    loading_phase: EditorPluginLoadingPhase,
    reached_loading_phase: Option<EditorPluginLoadingPhase>,
) -> bool {
    reached_loading_phase.is_some_and(|reached| loading_phase <= reached)
}

fn invalid_enablement(
    package_id: &str,
    state: EditorPluginState,
    enabled: bool,
) -> Result<EditorPluginState, EditorPluginTransitionError> {
    Err(EditorPluginTransitionError::InvalidEnablement {
        package_id: package_id.to_string(),
        state,
        enabled,
    })
}
