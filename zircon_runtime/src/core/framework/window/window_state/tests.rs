use std::num::{NonZeroU16, NonZeroU32};
use std::time::{Duration, Instant};

use super::{
    WindowCreateSpec, WindowDisplayTarget, WindowEffectiveMode, WindowEffectivePlacement,
    WindowEffectiveState, WindowEffectiveStateError, WindowExclusiveFullscreenFallback,
    WindowExclusiveFullscreenFallbackReason, WindowExclusiveFullscreenRequest,
    WindowExternalStatePolicy, WindowFocusState, WindowFullscreenFallback, WindowLogicalExtent,
    WindowLogicalPosition, WindowObservedMode, WindowObservedState, WindowOcclusionState,
    WindowPhysicalExtent, WindowPlacementRequest, WindowReconciliationAction,
    WindowReconciliationPolicy, WindowRequestedMode, WindowRequestedState, WindowStateField,
    WindowStateReconciliation, WindowStateResizeConstraints, WindowStateValidationError,
    WindowVideoModeRequest, WindowVisibilityState,
};
use crate::core::framework::window::{
    DisplayId, DisplayKind, DisplayTopologyGeneration, WindowCommand, WindowCommandHeader,
    WindowCommandId, WindowId, WindowRegistryId,
};

fn display() -> DisplayId {
    DisplayId::new(DisplayKind::PhysicalOutput, "edid:panel-a")
        .expect("fixture display identity is valid")
}

fn physical_extent() -> WindowPhysicalExtent {
    WindowPhysicalExtent::new(1920, 1080).expect("fixture physical extent is valid")
}

fn logical_extent() -> WindowLogicalExtent {
    WindowLogicalExtent::new(960.0, 540.0).expect("fixture logical extent is valid")
}

fn topology_generation() -> DisplayTopologyGeneration {
    DisplayTopologyGeneration::new(7).expect("fixture topology generation is nonzero")
}

fn requested_state() -> WindowRequestedState {
    WindowRequestedState::new(
        "Zircon Runtime",
        WindowPlacementRequest::CenteredOn(WindowDisplayTarget::Display(display())),
        WindowRequestedMode::Windowed,
        physical_extent(),
        WindowStateResizeConstraints::new(
            WindowLogicalExtent::new(320.0, 180.0).expect("fixture minimum is valid"),
            Some(logical_extent()),
        )
        .expect("fixture resize constraints are ordered"),
        true,
        true,
        true,
    )
}

#[test]
fn physical_and_logical_geometry_reject_invalid_values_with_exact_field_diagnostics() {
    assert_eq!(
        WindowPhysicalExtent::new(0, 1080),
        Err(WindowStateValidationError::NonPositive {
            field: WindowStateField::PhysicalWidth,
            value: 0.0,
        })
    );
    assert!(matches!(
        WindowLogicalExtent::new(f64::NAN, 1080.0),
        Err(WindowStateValidationError::NonFinite {
            field: WindowStateField::LogicalWidth,
            value,
        }) if value.is_nan()
    ));
    assert_eq!(
        WindowLogicalPosition::new(1.0, f64::INFINITY),
        Err(WindowStateValidationError::NonFinite {
            field: WindowStateField::PositionY,
            value: f64::INFINITY,
        })
    );
}

#[test]
fn requested_resize_constraints_reject_inversion_instead_of_clamping_to_a_default() {
    let minimum = WindowLogicalExtent::new(960.0, 540.0).expect("fixture minimum is valid");
    let maximum = WindowLogicalExtent::new(640.0, 720.0).expect("fixture maximum is valid");

    assert_eq!(
        WindowStateResizeConstraints::new(minimum, Some(maximum)),
        Err(WindowStateValidationError::MaximumBelowMinimum {
            axis: WindowStateField::MaximumLogicalWidth,
            minimum: 960.0,
            maximum: 640.0,
        })
    );
}

#[test]
fn requested_state_uses_only_stable_display_selection_and_never_persists_focus() {
    let state = requested_state();

    assert_eq!(state.title(), "Zircon Runtime");
    assert_eq!(state.physical_extent(), physical_extent());
    assert_eq!(
        state.placement(),
        &WindowPlacementRequest::CenteredOn(WindowDisplayTarget::Display(display()))
    );
    assert!(state.resizable());
    assert!(state.decorated());
    assert!(state.visible());

    let requested_source = include_str!("requested.rs");
    assert!(
        !requested_source.contains("focused"),
        "focus must remain an observed event fact, not requested window state"
    );
    let placement_source = include_str!("placement.rs");
    assert!(
        !placement_source.contains("Index(") && !placement_source.contains("Current,"),
        "create placement cannot route through a topology-local monitor index or current monitor"
    );
}

#[test]
fn exclusive_fullscreen_requires_explicit_fallback_policy_and_preserves_exact_mode_request() {
    let video_mode = WindowVideoModeRequest::new(
        physical_extent(),
        Some(NonZeroU16::new(30).expect("fixture bit depth is nonzero")),
        Some(NonZeroU32::new(60_000).expect("fixture refresh rate is nonzero")),
    );
    let exact = WindowRequestedMode::ExclusiveFullscreen(WindowExclusiveFullscreenRequest::new(
        WindowDisplayTarget::Display(display()),
        video_mode,
        WindowFullscreenFallback::Exact,
    ));
    let fallback = WindowRequestedMode::ExclusiveFullscreen(WindowExclusiveFullscreenRequest::new(
        WindowDisplayTarget::Primary,
        video_mode,
        WindowFullscreenFallback::AllowFallback,
    ));

    assert!(exact.requires_exact_video_mode());
    assert!(!fallback.requires_exact_video_mode());
    assert_eq!(exact.video_mode(), Some(video_mode));
    assert_eq!(fallback.video_mode(), Some(video_mode));
}

#[test]
fn requested_state_is_a_direct_generation_qualified_window_command_payload() {
    let target = WindowId::new(
        WindowRegistryId::new(19).expect("fixture registry identity is nonzero"),
        2,
        NonZeroU32::new(3).expect("fixture window generation is nonzero"),
    );
    let header = WindowCommandHeader::new(
        target,
        WindowCommandId::new(29).expect("fixture request identity is nonzero"),
        Instant::now() + Duration::from_secs(1),
    );
    let state = requested_state();
    let command = WindowCommand::new(header, state.clone());

    assert_eq!(command.target(), target);
    assert_eq!(command.request_id().raw(), 29);
    assert_eq!(command.desired(), &state);
    assert_eq!(command.deadline(), header.deadline());
}

#[test]
fn create_spec_keeps_initial_intent_separate_from_later_window_commands() {
    let requested = requested_state();
    let create = WindowCreateSpec::new(requested.clone(), topology_generation());

    assert_eq!(create.requested(), &requested);
    assert_eq!(create.display_topology_generation(), topology_generation());
    let create_source = include_str!("create_spec.rs");
    assert!(
        !create_source.contains("struct WindowCommand")
            && !create_source.contains("enum WindowCommand"),
        "creation intent must not grow a second runtime command path"
    );
}

#[test]
fn observed_state_keeps_focus_visibility_and_occlusion_as_distinct_os_facts() {
    let observed = WindowObservedState::new(
        display(),
        topology_generation(),
        physical_extent(),
        logical_extent(),
        WindowLogicalPosition::new(120.0, 64.0).expect("fixture position is valid"),
        2.0,
        WindowObservedMode::ExclusiveFullscreen { video_mode: None },
        WindowFocusState::Unfocused,
        WindowVisibilityState::Minimized,
        WindowOcclusionState::Occluded,
    )
    .expect("fixture observed state is valid");

    assert_eq!(observed.display(), &display());
    assert_eq!(
        observed.display_topology_generation(),
        topology_generation()
    );
    assert_eq!(observed.focus(), WindowFocusState::Unfocused);
    assert_eq!(observed.visibility(), WindowVisibilityState::Minimized);
    assert_eq!(observed.occlusion(), WindowOcclusionState::Occluded);
    assert!(observed.mode().is_fullscreen());
    assert_eq!(
        WindowObservedState::new(
            display(),
            topology_generation(),
            physical_extent(),
            logical_extent(),
            WindowLogicalPosition::new(0.0, 0.0).expect("fixture position is valid"),
            0.0,
            WindowObservedMode::Windowed,
            WindowFocusState::Focused,
            WindowVisibilityState::Visible,
            WindowOcclusionState::Unknown,
        ),
        Err(WindowStateValidationError::NonPositive {
            field: WindowStateField::ScaleFactor,
            value: 0.0,
        })
    );
}

#[test]
fn effective_state_retains_allowed_fullscreen_fallback_and_effective_constraints() {
    let video_mode = WindowVideoModeRequest::new(
        physical_extent(),
        Some(NonZeroU16::new(30).expect("fixture bit depth is nonzero")),
        Some(NonZeroU32::new(60_000).expect("fixture refresh rate is nonzero")),
    );
    let fallback = WindowExclusiveFullscreenFallback::new(
        video_mode,
        WindowExclusiveFullscreenFallbackReason::VideoModeUnavailable,
    );
    let constraints = WindowStateResizeConstraints::new(
        WindowLogicalExtent::new(320.0, 180.0).expect("fixture minimum is valid"),
        Some(logical_extent()),
    )
    .expect("fixture constraints are valid");
    let effective = WindowEffectiveState::new(
        "Zircon Runtime",
        WindowEffectivePlacement::new(
            display(),
            WindowLogicalPosition::new(80.0, 40.0).expect("fixture position is valid"),
        ),
        WindowEffectiveMode::BorderlessFullscreen {
            output: display(),
            exclusive_fallback: Some(fallback),
        },
        physical_extent(),
        constraints,
        true,
        true,
        true,
        topology_generation(),
    )
    .expect("same display is valid for an effective fullscreen state");

    assert_eq!(effective.resize_constraints(), constraints);
    assert_eq!(effective.mode().exclusive_fallback(), Some(fallback));
    assert_eq!(effective.mode().output(), Some(&display()));
    assert_eq!(
        effective.display_topology_generation(),
        topology_generation()
    );
    assert!(
        !include_str!("effective/state.rs").contains("WindowFocusState"),
        "focus remains an observed fact rather than an effective host setting"
    );
}

#[test]
fn reconciliation_actions_are_explicit_for_external_move_resize_mode_and_visibility() {
    let constraints = WindowStateResizeConstraints::new(
        WindowLogicalExtent::new(320.0, 180.0).expect("fixture minimum is valid"),
        Some(logical_extent()),
    )
    .expect("fixture constraints are valid");
    let effective = WindowEffectiveState::new(
        "Zircon Runtime",
        WindowEffectivePlacement::new(
            display(),
            WindowLogicalPosition::new(80.0, 40.0).expect("fixture position is valid"),
        ),
        WindowEffectiveMode::Windowed,
        physical_extent(),
        constraints,
        true,
        true,
        true,
        topology_generation(),
    )
    .expect("windowed state has no fullscreen output constraint");
    let observed = WindowObservedState::new(
        display(),
        topology_generation(),
        WindowPhysicalExtent::new(1280, 720).expect("fixture physical extent is valid"),
        WindowLogicalExtent::new(640.0, 360.0).expect("fixture logical extent is valid"),
        WindowLogicalPosition::new(160.0, 80.0).expect("fixture position is valid"),
        2.0,
        WindowObservedMode::BorderlessFullscreen,
        WindowFocusState::Unfocused,
        WindowVisibilityState::Minimized,
        WindowOcclusionState::Occluded,
    )
    .expect("fixture observed state is valid");
    let policy = WindowReconciliationPolicy::new(
        WindowExternalStatePolicy::AcceptExternal,
        WindowExternalStatePolicy::ReapplyEffective,
        WindowExternalStatePolicy::ReportConflict,
        WindowExternalStatePolicy::AcceptExternal,
    );

    let reconciliation = WindowStateReconciliation::compare(&effective, &observed, policy);

    assert_eq!(
        reconciliation.placement(),
        WindowReconciliationAction::AcceptExternal
    );
    assert_eq!(
        reconciliation.physical_extent(),
        WindowReconciliationAction::ReapplyEffective
    );
    assert_eq!(
        reconciliation.mode(),
        WindowReconciliationAction::ReportConflict
    );
    assert_eq!(
        reconciliation.visibility(),
        WindowReconciliationAction::AcceptExternal
    );
    assert!(reconciliation.has_correction());
    assert!(reconciliation.has_conflict());
}

#[test]
fn effective_fullscreen_state_rejects_an_output_that_differs_from_its_placement() {
    let alternate_display = DisplayId::new(DisplayKind::PhysicalOutput, "edid:panel-b")
        .expect("fixture alternate display identity is valid");
    let constraints = WindowStateResizeConstraints::new(
        WindowLogicalExtent::new(320.0, 180.0).expect("fixture minimum is valid"),
        Some(logical_extent()),
    )
    .expect("fixture constraints are valid");
    let placement = WindowEffectivePlacement::new(
        display(),
        WindowLogicalPosition::new(80.0, 40.0).expect("fixture position is valid"),
    );

    assert_eq!(
        WindowEffectiveState::new(
            "Zircon Runtime",
            placement,
            WindowEffectiveMode::BorderlessFullscreen {
                output: alternate_display.clone(),
                exclusive_fallback: None,
            },
            physical_extent(),
            constraints,
            true,
            true,
            true,
            topology_generation(),
        ),
        Err(WindowEffectiveStateError::FullscreenOutputMismatch {
            placement_display: display(),
            mode_output: alternate_display,
        })
    );
}
