use std::num::{NonZeroU16, NonZeroU32};

use crate::core::framework::window::{
    DisplayId, DisplayKind, DisplayTopologyGeneration, WindowCreateSpec, WindowDisplayTarget,
    WindowEffectiveMode, WindowEffectivePlacement, WindowEffectiveState,
    WindowExclusiveFullscreenFallback, WindowExclusiveFullscreenFallbackReason, WindowFocusState,
    WindowLogicalExtent, WindowLogicalPosition, WindowObservedMode, WindowObservedState,
    WindowOcclusionState, WindowPhysicalExtent, WindowPlacementRequest, WindowRequestedGeneration,
    WindowRequestedMode, WindowRequestedState, WindowStateResizeConstraints,
    WindowVideoModeRequest, WindowVisibilityState,
};
use crate::core::framework::window::{WindowId, WindowRegistryId};

use super::{WindowStateRegistry, WindowStateRegistryError};

fn window() -> WindowId {
    WindowId::new(
        WindowRegistryId::new(61).expect("fixture registry identity is nonzero"),
        3,
        NonZeroU32::new(2).expect("fixture window generation is nonzero"),
    )
}

fn display() -> DisplayId {
    DisplayId::new(DisplayKind::PhysicalOutput, "edid:panel-a")
        .expect("fixture display identity is valid")
}

fn topology_generation() -> DisplayTopologyGeneration {
    DisplayTopologyGeneration::new(5).expect("fixture topology generation is nonzero")
}

fn physical_extent() -> WindowPhysicalExtent {
    WindowPhysicalExtent::new(1920, 1080).expect("fixture physical extent is valid")
}

fn logical_extent() -> WindowLogicalExtent {
    WindowLogicalExtent::new(960.0, 540.0).expect("fixture logical extent is valid")
}

fn constraints() -> WindowStateResizeConstraints {
    WindowStateResizeConstraints::new(
        WindowLogicalExtent::new(320.0, 180.0).expect("fixture minimum is valid"),
        Some(logical_extent()),
    )
    .expect("fixture constraints are valid")
}

fn requested(title: &str) -> WindowRequestedState {
    WindowRequestedState::new(
        title,
        WindowPlacementRequest::CenteredOn(WindowDisplayTarget::Display(display())),
        WindowRequestedMode::Windowed,
        physical_extent(),
        constraints(),
        true,
        true,
        true,
    )
}

fn observed(physical_extent: WindowPhysicalExtent) -> WindowObservedState {
    WindowObservedState::new(
        display(),
        topology_generation(),
        physical_extent,
        logical_extent(),
        WindowLogicalPosition::new(80.0, 40.0).expect("fixture position is valid"),
        2.0,
        WindowObservedMode::Windowed,
        WindowFocusState::Focused,
        WindowVisibilityState::Visible,
        WindowOcclusionState::Unoccluded,
    )
    .expect("fixture observed state is valid")
}

fn effective(physical_extent: WindowPhysicalExtent) -> WindowEffectiveState {
    WindowEffectiveState::new(
        "Zircon Runtime",
        WindowEffectivePlacement::new(
            display(),
            WindowLogicalPosition::new(80.0, 40.0).expect("fixture position is valid"),
        ),
        WindowEffectiveMode::Windowed,
        physical_extent,
        constraints(),
        true,
        true,
        true,
        topology_generation(),
    )
    .expect("effective state uses one display for placement and fullscreen output")
}

fn create() -> WindowCreateSpec {
    WindowCreateSpec::new(requested("Zircon Runtime"), topology_generation())
}

#[test]
fn register_creates_independent_initial_state_generations() {
    let mut registry = WindowStateRegistry::default();
    let snapshot = registry
        .register(
            window(),
            create(),
            observed(physical_extent()),
            effective(physical_extent()),
        )
        .expect("initial state registers");

    assert_eq!(snapshot.window(), window());
    assert_eq!(snapshot.create().generation().get(), 1);
    assert_eq!(snapshot.requested().generation().get(), 1);
    assert_eq!(snapshot.observed().generation().get(), 1);
    assert_eq!(snapshot.effective().generation().get(), 1);
    assert_eq!(snapshot.effective().requested_generation().get(), 1);
    assert_eq!(snapshot.requested().state().title(), "Zircon Runtime");
    assert_eq!(registry.len(), 1);
}

#[test]
fn requested_observed_and_effective_publications_advance_only_their_own_generation() {
    let mut registry = WindowStateRegistry::default();
    registry
        .register(
            window(),
            create(),
            observed(physical_extent()),
            effective(physical_extent()),
        )
        .expect("initial state registers");

    let requested = registry
        .replace_requested(
            window(),
            WindowRequestedGeneration::initial(),
            requested("Updated Title"),
        )
        .expect("current requested generation is accepted");
    assert_eq!(requested.requested().generation().get(), 2);
    assert_eq!(requested.observed().generation().get(), 1);
    assert_eq!(requested.effective().generation().get(), 1);

    let observed = registry
        .publish_observed(
            window(),
            observed(WindowPhysicalExtent::new(1280, 720).expect("fixture extent is valid")),
        )
        .expect("observed event publishes");
    assert_eq!(observed.requested().generation().get(), 2);
    assert_eq!(observed.observed().generation().get(), 2);
    assert_eq!(observed.effective().generation().get(), 1);

    let effective = registry
        .publish_effective(
            window(),
            requested.requested().generation(),
            effective(WindowPhysicalExtent::new(1280, 720).expect("fixture extent is valid")),
        )
        .expect("effective publication matches the requested generation");
    assert_eq!(effective.requested().generation().get(), 2);
    assert_eq!(effective.observed().generation().get(), 2);
    assert_eq!(effective.effective().generation().get(), 2);
    assert_eq!(effective.effective().requested_generation().get(), 2);
}

#[test]
fn stale_requested_or_future_effective_publication_rejects_without_mutating_state() {
    let mut registry = WindowStateRegistry::default();
    registry
        .register(
            window(),
            create(),
            observed(physical_extent()),
            effective(physical_extent()),
        )
        .expect("initial state registers");
    let updated = registry
        .replace_requested(
            window(),
            WindowRequestedGeneration::initial(),
            requested("Updated Title"),
        )
        .expect("current requested generation is accepted");

    assert_eq!(
        registry.replace_requested(
            window(),
            WindowRequestedGeneration::initial(),
            requested("Stale Title"),
        ),
        Err(WindowStateRegistryError::RequestedGenerationMismatch {
            window: window(),
            expected: WindowRequestedGeneration::initial(),
            actual: updated.requested().generation(),
        })
    );
    let future_requested_generation = updated
        .requested()
        .generation()
        .next()
        .expect("fixture requested generation can advance");
    assert_eq!(
        registry.preflight_command_completion(window(), Some(future_requested_generation)),
        Err(WindowStateRegistryError::EffectiveRequestGenerationAhead {
            window: window(),
            source_requested: future_requested_generation,
            current_requested: updated.requested().generation(),
        })
    );
    assert_eq!(
        registry.publish_effective(
            window(),
            future_requested_generation,
            effective(physical_extent()),
        ),
        Err(WindowStateRegistryError::EffectiveRequestGenerationAhead {
            window: window(),
            source_requested: future_requested_generation,
            current_requested: updated.requested().generation(),
        })
    );
    assert_eq!(
        registry
            .snapshot(window())
            .expect("state remains registered")
            .requested()
            .state()
            .title(),
        "Updated Title"
    );
}

#[test]
fn effective_source_generation_never_regresses_after_a_newer_native_completion() {
    let mut registry = WindowStateRegistry::default();
    registry
        .register(
            window(),
            create(),
            observed(physical_extent()),
            effective(physical_extent()),
        )
        .expect("initial state registers");
    let current_request = registry
        .replace_requested(
            window(),
            WindowRequestedGeneration::initial(),
            requested("First Title"),
        )
        .expect("first request publishes");
    let newer_request = registry
        .replace_requested(
            window(),
            current_request.requested().generation(),
            requested("Second Title"),
        )
        .expect("second request publishes");
    registry
        .publish_effective(
            window(),
            newer_request.requested().generation(),
            effective(physical_extent()),
        )
        .expect("newer native completion publishes effective state");

    assert_eq!(
        registry.publish_effective(
            window(),
            current_request.requested().generation(),
            effective(physical_extent()),
        ),
        Err(
            WindowStateRegistryError::EffectiveRequestGenerationRegressed {
                window: window(),
                source_requested: current_request.requested().generation(),
                current_effective: newer_request.requested().generation(),
            }
        )
    );
}

#[test]
fn command_completion_preflight_is_side_effect_free_for_any_admitted_request() {
    let mut registry = WindowStateRegistry::default();
    registry
        .register(
            window(),
            create(),
            observed(physical_extent()),
            effective(physical_extent()),
        )
        .expect("initial state registers");

    registry
        .preflight_command_completion(window(), Some(WindowRequestedGeneration::initial()))
        .expect("current request can publish an effective completion");
    let before_update = registry
        .snapshot(window())
        .expect("preflight does not change any snapshot");
    assert_eq!(before_update.observed().generation().get(), 1);
    assert_eq!(before_update.effective().generation().get(), 1);

    registry
        .replace_requested(
            window(),
            WindowRequestedGeneration::initial(),
            requested("Newer Title"),
        )
        .expect("new request publishes");
    registry
        .preflight_command_completion(window(), Some(WindowRequestedGeneration::initial()))
        .expect("an accepted older request can report its actual effective state");
    let after_update = registry
        .snapshot(window())
        .expect("stale preflight does not mutate any snapshot");
    assert_eq!(after_update.requested().generation().get(), 2);
    assert_eq!(after_update.observed().generation().get(), 1);
    assert_eq!(after_update.effective().generation().get(), 1);
}

#[test]
fn remove_returns_last_snapshot_and_rejects_later_generation_queries() {
    let mut registry = WindowStateRegistry::default();
    registry
        .register(
            window(),
            create(),
            observed(physical_extent()),
            effective(physical_extent()),
        )
        .expect("initial state registers");

    let removed = registry.remove(window()).expect("registered state removes");

    assert_eq!(removed.window(), window());
    assert!(!registry.contains(window()));
    assert_eq!(
        registry.snapshot(window()),
        Err(WindowStateRegistryError::UnknownWindowState { window: window() })
    );
}

#[test]
fn allowed_exclusive_fallback_is_stored_as_effective_state_not_silently_discarded() {
    let mut registry = WindowStateRegistry::default();
    registry
        .register(
            window(),
            create(),
            observed(physical_extent()),
            effective(physical_extent()),
        )
        .expect("initial state registers");
    let video_mode = WindowVideoModeRequest::new(
        physical_extent(),
        Some(NonZeroU16::new(30).expect("fixture bit depth is nonzero")),
        Some(NonZeroU32::new(60_000).expect("fixture refresh rate is nonzero")),
    );
    let mut effective_state = effective(physical_extent());
    effective_state = WindowEffectiveState::new(
        effective_state.title(),
        effective_state.placement().clone(),
        WindowEffectiveMode::BorderlessFullscreen {
            output: display(),
            exclusive_fallback: Some(WindowExclusiveFullscreenFallback::new(
                video_mode,
                WindowExclusiveFullscreenFallbackReason::VideoModeUnavailable,
            )),
        },
        effective_state.physical_extent(),
        effective_state.resize_constraints(),
        effective_state.resizable(),
        effective_state.decorated(),
        effective_state.visible(),
        effective_state.display_topology_generation(),
    )
    .expect("allowed fullscreen fallback resolves onto the placement display");

    let snapshot = registry
        .publish_effective(
            window(),
            WindowRequestedGeneration::initial(),
            effective_state,
        )
        .expect("current request may publish an allowed fallback");

    assert!(snapshot
        .effective()
        .state()
        .mode()
        .exclusive_fallback()
        .is_some());
}
