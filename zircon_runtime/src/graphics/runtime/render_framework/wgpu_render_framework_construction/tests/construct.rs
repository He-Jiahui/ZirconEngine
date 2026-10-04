use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::graphics::{
    HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput, HybridGiRuntimePrepareOutput,
    HybridGiRuntimeProvider, HybridGiRuntimeProviderRegistration, HybridGiRuntimeState,
    HybridGiRuntimeUpdate, VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
    VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeProvider,
    VirtualGeometryRuntimeProviderRegistration, VirtualGeometryRuntimeState,
    VirtualGeometryRuntimeUpdate,
};

use super::{
    select_hybrid_gi_runtime_provider, select_provider, select_virtual_geometry_runtime_provider,
    selected_advanced_provider_availability,
};

struct CloneCountingProvider {
    id: String,
    priority: i32,
    clone_count: Arc<AtomicUsize>,
}

impl Clone for CloneCountingProvider {
    fn clone(&self) -> Self {
        self.clone_count.fetch_add(1, Ordering::Relaxed);
        Self {
            id: self.id.clone(),
            priority: self.priority,
            clone_count: Arc::clone(&self.clone_count),
        }
    }
}

#[derive(Debug)]
struct NoopVirtualGeometryProvider;

impl VirtualGeometryRuntimeProvider for NoopVirtualGeometryProvider {
    fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState> {
        Box::new(NoopVirtualGeometryState)
    }
}

#[derive(Debug)]
struct NoopVirtualGeometryState;

impl VirtualGeometryRuntimeState for NoopVirtualGeometryState {
    fn prepare_frame(
        &mut self,
        _input: VirtualGeometryRuntimePrepareInput<'_>,
    ) -> VirtualGeometryRuntimePrepareOutput {
        VirtualGeometryRuntimePrepareOutput::default()
    }

    fn update_after_render(
        &mut self,
        _feedback: VirtualGeometryRuntimeFeedback,
    ) -> VirtualGeometryRuntimeUpdate {
        VirtualGeometryRuntimeUpdate::default()
    }
}

#[derive(Debug)]
struct NoopHybridGiProvider;

impl HybridGiRuntimeProvider for NoopHybridGiProvider {
    fn create_state(&self) -> Box<dyn HybridGiRuntimeState> {
        Box::new(NoopHybridGiState)
    }
}

struct NoopHybridGiState;

impl HybridGiRuntimeState for NoopHybridGiState {
    fn prepare_frame(
        &mut self,
        _input: HybridGiRuntimePrepareInput<'_>,
    ) -> HybridGiRuntimePrepareOutput {
        HybridGiRuntimePrepareOutput::default()
    }

    fn update_after_render(&mut self, _feedback: HybridGiRuntimeFeedback) -> HybridGiRuntimeUpdate {
        HybridGiRuntimeUpdate::default()
    }
}

fn virtual_geometry_registration(
    provider_id: &str,
    priority: i32,
) -> VirtualGeometryRuntimeProviderRegistration {
    VirtualGeometryRuntimeProviderRegistration::new(
        provider_id,
        Arc::new(NoopVirtualGeometryProvider),
    )
    .with_priority(priority)
}

fn hybrid_gi_registration(provider_id: &str, priority: i32) -> HybridGiRuntimeProviderRegistration {
    HybridGiRuntimeProviderRegistration::new(provider_id, Arc::new(NoopHybridGiProvider))
        .with_priority(priority)
}

#[test]
fn advanced_provider_selection_rejects_duplicate_virtual_geometry_ids() {
    let error = select_virtual_geometry_runtime_provider(vec![
        virtual_geometry_registration("same", 0),
        virtual_geometry_registration("same", 10),
    ])
    .expect_err("duplicate provider ids must be rejected");

    assert!(error
        .to_string()
        .contains("virtual_geometry provider `same` registered more than once"));
}

#[test]
fn advanced_provider_selection_rejects_priority_ties() {
    let error = select_hybrid_gi_runtime_provider(vec![
        hybrid_gi_registration("first", 5),
        hybrid_gi_registration("second", 5),
    ])
    .expect_err("priority ties must be rejected");

    assert!(error
        .to_string()
        .contains("hybrid_global_illumination provider priority tie at 5"));
}

#[test]
fn advanced_provider_selection_uses_highest_priority_and_reports_ids() {
    let virtual_geometry = select_virtual_geometry_runtime_provider(vec![
        virtual_geometry_registration("low", 1),
        virtual_geometry_registration("high", 20),
    ])
    .expect("virtual geometry provider selection should succeed")
    .expect("one virtual geometry provider should be selected");
    let hybrid_gi = select_hybrid_gi_runtime_provider(vec![hybrid_gi_registration("gi", 7)])
        .expect("hybrid gi provider selection should succeed")
        .expect("one hybrid gi provider should be selected");

    assert_eq!(virtual_geometry.provider_id(), "high");
    assert_eq!(hybrid_gi.provider_id(), "gi");

    let availability =
        selected_advanced_provider_availability(Some(&hybrid_gi), Some(&virtual_geometry));
    assert_eq!(
        availability.virtual_geometry_provider_id.as_deref(),
        Some("high")
    );
    assert_eq!(availability.hybrid_gi_provider_id.as_deref(), Some("gi"));
}

#[test]
fn provider_selection_moves_the_owned_winner_without_cloning() {
    let clone_count = Arc::new(AtomicUsize::new(0));
    let selected = select_provider(
        "test",
        vec![
            CloneCountingProvider {
                id: "low".into(),
                priority: 1,
                clone_count: Arc::clone(&clone_count),
            },
            CloneCountingProvider {
                id: "high".into(),
                priority: 10,
                clone_count: Arc::clone(&clone_count),
            },
        ],
        |provider| provider.id.as_str(),
        |provider| provider.priority,
    )
    .expect("provider selection should succeed")
    .expect("one provider should be selected");

    assert_eq!(selected.id, "high");
    assert_eq!(clone_count.load(Ordering::Relaxed), 0);
}
