use super::{
    WindowEffectiveMode, WindowEffectiveState, WindowObservedMode, WindowObservedState,
    WindowVisibilityState,
};

/// Per-field policy for an externally initiated native-window change. Focus
/// and occlusion do not participate because they are observations only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowExternalStatePolicy {
    AcceptExternal,
    ReapplyEffective,
    ReportConflict,
}

/// Explicit reconciliation policy for the OS-controlled window dimensions.
/// The policy is pure data so the host broker can apply it consistently for
/// backend events and command-completion readbacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowReconciliationPolicy {
    placement: WindowExternalStatePolicy,
    physical_extent: WindowExternalStatePolicy,
    mode: WindowExternalStatePolicy,
    visibility: WindowExternalStatePolicy,
}

impl WindowReconciliationPolicy {
    pub const fn new(
        placement: WindowExternalStatePolicy,
        physical_extent: WindowExternalStatePolicy,
        mode: WindowExternalStatePolicy,
        visibility: WindowExternalStatePolicy,
    ) -> Self {
        Self {
            placement,
            physical_extent,
            mode,
            visibility,
        }
    }

    pub const fn placement(self) -> WindowExternalStatePolicy {
        self.placement
    }

    pub const fn physical_extent(self) -> WindowExternalStatePolicy {
        self.physical_extent
    }

    pub const fn mode(self) -> WindowExternalStatePolicy {
        self.mode
    }

    pub const fn visibility(self) -> WindowExternalStatePolicy {
        self.visibility
    }
}

/// The broker action for one effective-state field after an observed platform
/// event. `InSync` means no policy action is necessary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowReconciliationAction {
    InSync,
    AcceptExternal,
    ReapplyEffective,
    ReportConflict,
}

/// Fixed-size reconciliation result for an observed native-window update.
/// It intentionally excludes focus and occlusion because treating those facts
/// as corrective configuration would recreate the current lifecycle bug.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowStateReconciliation {
    placement: WindowReconciliationAction,
    physical_extent: WindowReconciliationAction,
    mode: WindowReconciliationAction,
    visibility: WindowReconciliationAction,
}

impl WindowStateReconciliation {
    pub fn compare(
        effective: &WindowEffectiveState,
        observed: &WindowObservedState,
        policy: WindowReconciliationPolicy,
    ) -> Self {
        Self {
            placement: decision(
                effective.placement().display() == observed.display()
                    && effective.placement().logical_position() == observed.logical_position(),
                policy.placement(),
            ),
            physical_extent: decision(
                effective.physical_extent() == observed.physical_extent(),
                policy.physical_extent(),
            ),
            mode: decision(
                mode_matches(effective.mode(), observed.mode()),
                policy.mode(),
            ),
            visibility: decision(
                effective.visible() == observed_is_visible(observed.visibility()),
                policy.visibility(),
            ),
        }
    }

    pub const fn placement(self) -> WindowReconciliationAction {
        self.placement
    }

    pub const fn physical_extent(self) -> WindowReconciliationAction {
        self.physical_extent
    }

    pub const fn mode(self) -> WindowReconciliationAction {
        self.mode
    }

    pub const fn visibility(self) -> WindowReconciliationAction {
        self.visibility
    }

    pub const fn has_correction(self) -> bool {
        matches!(self.placement, WindowReconciliationAction::ReapplyEffective)
            || matches!(
                self.physical_extent,
                WindowReconciliationAction::ReapplyEffective
            )
            || matches!(self.mode, WindowReconciliationAction::ReapplyEffective)
            || matches!(
                self.visibility,
                WindowReconciliationAction::ReapplyEffective
            )
    }

    pub const fn has_conflict(self) -> bool {
        matches!(self.placement, WindowReconciliationAction::ReportConflict)
            || matches!(
                self.physical_extent,
                WindowReconciliationAction::ReportConflict
            )
            || matches!(self.mode, WindowReconciliationAction::ReportConflict)
            || matches!(self.visibility, WindowReconciliationAction::ReportConflict)
    }
}

fn decision(
    matches_effective: bool,
    policy: WindowExternalStatePolicy,
) -> WindowReconciliationAction {
    if matches_effective {
        return WindowReconciliationAction::InSync;
    }
    match policy {
        WindowExternalStatePolicy::AcceptExternal => WindowReconciliationAction::AcceptExternal,
        WindowExternalStatePolicy::ReapplyEffective => WindowReconciliationAction::ReapplyEffective,
        WindowExternalStatePolicy::ReportConflict => WindowReconciliationAction::ReportConflict,
    }
}

fn mode_matches(effective: &WindowEffectiveMode, observed: WindowObservedMode) -> bool {
    match (effective, observed) {
        (WindowEffectiveMode::Windowed, WindowObservedMode::Windowed)
        | (
            WindowEffectiveMode::BorderlessFullscreen { .. },
            WindowObservedMode::BorderlessFullscreen,
        ) => true,
        (
            WindowEffectiveMode::ExclusiveFullscreen { video_mode, .. },
            WindowObservedMode::ExclusiveFullscreen {
                video_mode: Some(observed_video_mode),
            },
        ) => *video_mode == observed_video_mode,
        _ => false,
    }
}

const fn observed_is_visible(visibility: WindowVisibilityState) -> bool {
    matches!(visibility, WindowVisibilityState::Visible)
}
