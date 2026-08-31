use std::num::NonZeroU32;

use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::{SurfaceLeaseError, SurfaceLeaseRegistry, SurfaceLeaseRequest};
use crate::core::framework::window::{
    DisplayId, DisplayKind, DisplayLogicalRect, DisplayObservation, DisplayOrientation,
    DisplayOutputCapabilities, DisplayPhysicalRect, DisplaySnapshot, DisplayTopologyGeneration,
    DisplayTopologySnapshot, WindowId, WindowRegistryId,
};

fn window(slot: u32) -> WindowId {
    WindowId::new(
        WindowRegistryId::new(17).expect("fixture registry identity is nonzero"),
        slot,
        NonZeroU32::MIN,
    )
}

fn viewport(raw: u64) -> ZrRuntimeViewportHandle {
    ZrRuntimeViewportHandle::new(raw)
}

fn output(key: &str) -> DisplayId {
    DisplayId::new(DisplayKind::PhysicalOutput, key).expect("fixture output identity is valid")
}

fn topology(generation: u64, output: DisplayId) -> DisplayTopologySnapshot {
    let snapshot = DisplaySnapshot::new(
        output.clone(),
        DisplayObservation {
            physical_bounds: DisplayPhysicalRect::new(
                0,
                0,
                NonZeroU32::new(1920).expect("fixture output width is nonzero"),
                NonZeroU32::new(1080).expect("fixture output height is nonzero"),
            ),
            usable_logical_bounds: DisplayLogicalRect::new(0.0, 0.0, 1920.0, 1080.0)
                .expect("fixture output logical bounds are valid"),
            scale_factor: 1.0,
            refresh_rate_millihertz: None,
            orientation: DisplayOrientation::Landscape,
            safe_area: None,
            output_capabilities: DisplayOutputCapabilities::default(),
        },
    )
    .expect("fixture display snapshot is valid");
    DisplayTopologySnapshot::new(
        DisplayTopologyGeneration::new(generation).expect("fixture topology generation is nonzero"),
        vec![snapshot],
        Some(output),
    )
    .expect("fixture display topology is valid")
}

fn request(
    window: WindowId,
    viewport: ZrRuntimeViewportHandle,
    output: DisplayId,
    generation: DisplayTopologyGeneration,
) -> SurfaceLeaseRequest {
    SurfaceLeaseRequest::new(window, viewport, output, generation)
}

#[test]
fn prepare_keeps_current_lease_routable_until_matching_candidate_publishes() {
    let output = output("edid:panel-a");
    let topology = topology(4, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let request = request(window(0), viewport(7), output, topology.generation());

    let first_prepared = registry
        .prepare(request.clone(), &topology)
        .expect("first prepare succeeds");
    let first = registry
        .publish(&first_prepared, &topology)
        .expect("first candidate publishes");
    let active = first.current().clone();
    assert_eq!(first.retired(), None);

    let replacement = registry
        .prepare(request, &topology)
        .expect("replacement prepare succeeds");
    assert_eq!(registry.active(&active), Ok(()));
    assert_eq!(registry.active_count(), 1);
    assert!(matches!(
        registry.prepare(replacement.request().clone(), &topology),
        Err(SurfaceLeaseError::ReplacementInFlight { .. })
    ));

    let published = registry
        .publish(&replacement, &topology)
        .expect("matching replacement publishes");
    assert_eq!(published.retired(), Some(&active));
    assert_ne!(published.current().generation(), active.generation());
    assert_eq!(
        registry.active(&active),
        Err(SurfaceLeaseError::StaleLease { lease: active })
    );
    assert_eq!(registry.active(published.current()), Ok(()));
}

#[test]
fn topology_mismatch_and_unknown_output_fail_without_reserving_a_lease() {
    let known_output = output("edid:panel-a");
    let topology = topology(2, known_output.clone());
    let mut registry = SurfaceLeaseRegistry::default();

    let stale_request = request(
        window(0),
        viewport(1),
        known_output.clone(),
        DisplayTopologyGeneration::new(1).expect("fixture topology generation is nonzero"),
    );
    assert_eq!(
        registry.prepare(stale_request, &topology),
        Err(SurfaceLeaseError::TopologyGenerationMismatch {
            requested: DisplayTopologyGeneration::new(1)
                .expect("fixture topology generation is nonzero"),
            observed: topology.generation(),
        })
    );

    let missing_output = output("edid:missing");
    let missing_request = request(
        window(0),
        viewport(1),
        missing_output.clone(),
        topology.generation(),
    );
    assert_eq!(
        registry.prepare(missing_request, &topology),
        Err(SurfaceLeaseError::OutputUnavailable {
            output: missing_output,
            topology_generation: topology.generation(),
        })
    );
    assert_eq!(registry.active_count(), 0);
    assert_eq!(registry.preparing_count(), 0);
}

#[test]
fn publish_rejects_a_candidate_prepared_against_an_older_topology_generation() {
    let output = output("edid:panel-a");
    let prepared_topology = topology(1, output.clone());
    let current_topology = topology(2, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let prepared = registry
        .prepare(
            request(
                window(0),
                viewport(1),
                output,
                prepared_topology.generation(),
            ),
            &prepared_topology,
        )
        .expect("candidate prepares against the observed topology");

    assert_eq!(
        registry.publish(&prepared, &current_topology),
        Err(SurfaceLeaseError::TopologyGenerationMismatch {
            requested: prepared_topology.generation(),
            observed: current_topology.generation(),
        })
    );
    assert_eq!(registry.preparing_count(), 1);
    assert_eq!(registry.active_count(), 0);
    registry
        .cancel(&prepared)
        .expect("stale preparation remains cancelable after rejected publication");
}

#[test]
fn cancel_restores_the_previous_active_lease_without_changing_its_generation() {
    let output = output("edid:panel-a");
    let topology = topology(1, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let request = request(window(0), viewport(3), output, topology.generation());
    let first_prepared = registry
        .prepare(request.clone(), &topology)
        .expect("first prepare succeeds");
    let active = registry
        .publish(&first_prepared, &topology)
        .expect("first candidate publishes")
        .current()
        .clone();
    let replacement = registry
        .prepare(request, &topology)
        .expect("replacement prepare succeeds");

    registry
        .cancel(&replacement)
        .expect("prepared candidate cancels");
    assert_eq!(registry.active(&active), Ok(()));
    assert_eq!(registry.active_count(), 1);
    assert_eq!(registry.preparing_count(), 0);
    assert_eq!(
        registry.publish(&replacement, &topology),
        Err(SurfaceLeaseError::StaleLease {
            lease: replacement.candidate().clone(),
        })
    );
}

#[test]
fn window_retirement_revokes_all_active_leases_before_their_final_removal() {
    let output = output("edid:panel-a");
    let topology = topology(8, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let window = window(0);
    let first_request = request(window, viewport(9), output.clone(), topology.generation());
    let second_request = request(window, viewport(3), output, topology.generation());
    let first_prepared = registry
        .prepare(first_request, &topology)
        .expect("first prepare succeeds");
    let first = registry
        .publish(&first_prepared, &topology)
        .expect("first candidate publishes")
        .current()
        .clone();
    let second_prepared = registry
        .prepare(second_request, &topology)
        .expect("second prepare succeeds");
    let second = registry
        .publish(&second_prepared, &topology)
        .expect("second candidate publishes")
        .current()
        .clone();

    let retiring = registry
        .begin_retire_window(window)
        .expect("all active window leases begin retirement together");
    assert_eq!(retiring, vec![second.clone(), first.clone()]);
    assert_eq!(
        registry.active(&first),
        Err(SurfaceLeaseError::LeaseRetiring {
            lease: first.clone()
        })
    );
    assert_eq!(
        registry.active(&second),
        Err(SurfaceLeaseError::LeaseRetiring {
            lease: second.clone()
        })
    );
    assert!(matches!(
        registry.prepare(first.request().clone(), &topology),
        Err(SurfaceLeaseError::LeaseRetiring { .. })
    ));

    registry
        .complete_retirement(&first)
        .expect("first retired lease is removed after graphics teardown");
    registry
        .complete_retirement(&second)
        .expect("second retired lease is removed after graphics teardown");
    assert_eq!(registry.active_count(), 0);
}

#[test]
fn pending_candidate_prevents_window_retirement_without_revoking_existing_state() {
    let output = output("edid:panel-a");
    let topology = topology(1, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let window = window(0);
    let prepared = registry
        .prepare(
            request(window, viewport(1), output, topology.generation()),
            &topology,
        )
        .expect("candidate prepares");

    assert_eq!(
        registry.begin_retire_window(window),
        Err(SurfaceLeaseError::WindowHasPreparedLease { window })
    );
    assert_eq!(registry.preparing_count(), 1);
    registry
        .cancel(&prepared)
        .expect("candidate cancels after rejected retirement");
}

#[test]
fn viewport_cannot_move_to_another_window_until_the_old_lease_retires() {
    let output = output("edid:panel-a");
    let topology = topology(3, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let first_request = request(
        window(0),
        viewport(7),
        output.clone(),
        topology.generation(),
    );
    let second_request = request(window(1), viewport(7), output, topology.generation());
    let first_prepared = registry
        .prepare(first_request, &topology)
        .expect("first viewport owner prepares");
    let first = registry
        .publish(&first_prepared, &topology)
        .expect("first viewport owner publishes")
        .current()
        .clone();

    assert_eq!(
        registry.prepare(second_request.clone(), &topology),
        Err(SurfaceLeaseError::ViewportAlreadyBound {
            viewport: viewport(7),
            window: window(0),
        })
    );
    assert_eq!(registry.active(&first), Ok(()));

    registry
        .begin_retirement(&first)
        .expect("old viewport owner begins graphics retirement");
    assert_eq!(
        registry.prepare(second_request.clone(), &topology),
        Err(SurfaceLeaseError::LeaseRetiring {
            lease: first.clone(),
        })
    );
    registry
        .complete_retirement(&first)
        .expect("old viewport owner completes graphics retirement");
    assert_eq!(
        registry
            .prepare(second_request, &topology)
            .expect("viewport may bind after the prior lease is fully retired")
            .candidate()
            .window(),
        window(1)
    );
}

#[test]
fn canceling_an_initial_preparation_releases_its_viewport_owner() {
    let output = output("edid:panel-a");
    let topology = topology(3, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let first = registry
        .prepare(
            request(
                window(0),
                viewport(7),
                output.clone(),
                topology.generation(),
            ),
            &topology,
        )
        .expect("first viewport owner prepares");

    registry
        .cancel(&first)
        .expect("unpublished candidate cancels cleanly");
    assert_eq!(registry.preparing_count(), 0);
    assert_eq!(registry.active_count(), 0);
    assert_eq!(
        registry
            .prepare(
                request(window(1), viewport(7), output, topology.generation()),
                &topology,
            )
            .expect("canceled viewport binding is reusable by another window")
            .candidate()
            .window(),
        window(1)
    );
}
