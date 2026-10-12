use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::asset::artifact::{
    RenderArtifactBlockCodec, RenderArtifactBlockDescriptor, RenderArtifactBlockIoDispatchBudget,
    RenderArtifactBlockLoader, RenderArtifactBlockLoaderLimits, RenderArtifactContentId,
    RenderArtifactIoPriority, RenderArtifactLayout, RenderArtifactManifest,
    RenderArtifactManifestIoDispatchBudget, RenderArtifactManifestLoader,
    RenderArtifactManifestLoaderLimits, RenderArtifactResidencyClass, RenderArtifactStore,
    RenderArtifactStoreLimits, RenderArtifactTextureBlockFormat, RenderArtifactTextureLayout,
    RenderSubresourceId,
};
use crate::core::resource::{ResourceKind, ResourceManager};
use crate::core::runtime::{EngineTaskGraph, EngineTaskGraphOptions};
use crate::graphics::scene::render_scene::RenderSceneResourceReferenceDelta;

use super::super::*;
use super::{demand_generation, device_epoch, register_resource, request_for};

static NEXT_EXECUTOR_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

struct ExecutorTestRoot(std::path::PathBuf);

impl ExecutorTestRoot {
    fn new() -> Self {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(".codex_tmp")
            .join("render-semantic-executor-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT_EXECUTOR_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(&root)
            .unwrap_or_else(|error| panic!("failed to create executor test root: {error}"));
        Self(root)
    }

    fn store(&self) -> RenderArtifactStore {
        RenderArtifactStore::new(self.0.clone())
    }
}

impl Drop for ExecutorTestRoot {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        }
    }
}

fn loaders(
    store: RenderArtifactStore,
    runtime: &EngineTaskGraph,
) -> (RenderArtifactManifestLoader, RenderArtifactBlockLoader) {
    let store_limits = RenderArtifactStoreLimits::new(1024 * 1024, 1024 * 1024);
    let manifest = RenderArtifactManifestLoader::new(
        store.clone(),
        RenderArtifactManifestLoaderLimits::new(8, 16, 4, 16 * 1024 * 1024, store_limits),
        runtime,
    )
    .unwrap_or_else(|error| panic!("semantic manifest loader failed: {error}"));
    let blocks = RenderArtifactBlockLoader::new(
        store,
        RenderArtifactBlockLoaderLimits::new(8, 16, 4, 1024 * 1024, 1024 * 1024, store_limits),
        runtime,
    )
    .unwrap_or_else(|error| panic!("semantic block loader failed: {error}"));
    (manifest, blocks)
}

fn executor_limits(max_in_flight: usize) -> RenderAssetSemanticExecutorLimits {
    RenderAssetSemanticExecutorLimits::new(
        max_in_flight,
        RenderAssetGpuUploadLimits::new(8, 1024 * 1024, 1024 * 1024),
    )
}

fn maintenance_budget(max_load_advances: usize) -> RenderAssetSemanticExecutorMaintenanceBudget {
    RenderAssetSemanticExecutorMaintenanceBudget::new(
        RenderArtifactManifestIoDispatchBudget::new(2),
        RenderArtifactBlockIoDispatchBudget::new(2, 1024 * 1024),
        max_load_advances,
        2,
        2,
    )
}

fn texture_block(bytes: &[u8]) -> RenderArtifactBlockDescriptor {
    RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 },
        RenderArtifactContentId::from_bytes(*blake3::hash(bytes).as_bytes()),
        RenderArtifactBlockCodec::Raw,
        bytes.len() as u64,
        bytes.len() as u64,
        256,
        Arc::from("rgba8unorm"),
        RenderArtifactResidencyClass::Bootstrap,
        Vec::new(),
    )
}

#[test]
fn semantic_executor_capacity_is_atomic_and_cancel_drops_manifest_ticket() {
    let resources = ResourceManager::new();
    let first = register_resource(
        &resources,
        "textures/executor-first.rgba",
        ResourceKind::Texture,
        Vec::new(),
    );
    let second = register_resource(
        &resources,
        "textures/executor-second.rgba",
        ResourceKind::Texture,
        Vec::new(),
    );
    let mut residency = RenderAssetResidencyManager::new();
    let mutation = residency
        .apply_scene_reference_deltas(
            &[
                RenderSceneResourceReferenceDelta::acquire(first, 1),
                RenderSceneResourceReferenceDelta::acquire(second, 1),
            ],
            &resources.management_generation(),
            &resources.readiness_generation(),
            device_epoch(51, 1),
            demand_generation(1),
        )
        .unwrap_or_else(|error| panic!("residency admission failed: {error:?}"));
    let first_ticket = request_for(mutation.requests(), first);
    let second_ticket = request_for(mutation.requests(), second);
    let root = ExecutorTestRoot::new();
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .unwrap_or_else(|error| panic!("semantic executor runtime failed: {error}"));
    let (manifest_loader, block_loader) = loaders(root.store(), &runtime);
    let mut executor = RenderAssetSemanticExecutor::try_new(
        manifest_loader.clone(),
        block_loader,
        Arc::from("windows-dx12-sm6"),
        executor_limits(1),
        &runtime,
    )
    .unwrap_or_else(|error| panic!("semantic executor failed: {error}"));

    executor
        .admit(
            first_ticket.clone(),
            RenderArtifactIoPriority::HIGH,
            None,
            &mut residency,
        )
        .unwrap_or_else(|error| panic!("first semantic admission failed: {error}"));
    assert!(matches!(
        executor.admit(
            second_ticket.clone(),
            RenderArtifactIoPriority::HIGH,
            None,
            &mut residency,
        ),
        Err(RenderAssetSemanticExecutorAdmissionError::CapacityExceeded { capacity: 1 })
    ));
    assert_eq!(
        residency.state(&first_ticket),
        Some(RenderAssetResidencyState::Reading)
    );
    assert_eq!(
        residency.state(&second_ticket),
        Some(RenderAssetResidencyState::QueuedIo)
    );
    assert_eq!(manifest_loader.diagnostics().live_tickets, 1);

    let replacement_resources = ResourceManager::new();
    register_resource(
        &replacement_resources,
        "textures/executor-first.rgba",
        ResourceKind::Texture,
        Vec::new(),
    );
    let wrong_publication = super::super::RenderAssetResidencyTicket::from_parts(
        first_ticket.id(),
        first_ticket.resource(),
        first_ticket.asset_revision(),
        replacement_resources
            .readiness_generation()
            .row_identity(first.id())
            .expect("replacement row"),
        first_ticket.demand_generation(),
        first_ticket.device(),
        first_ticket.scope(),
        first_ticket.route(),
    );
    assert!(!executor.cancel(&wrong_publication));
    assert_eq!(manifest_loader.diagnostics().live_tickets, 1);
    assert_eq!(executor.diagnostics().in_flight(), 1);
    assert!(executor.cancel(&first_ticket));
    assert_eq!(manifest_loader.diagnostics().live_tickets, 0);
    assert_eq!(executor.diagnostics().in_flight(), 0);
}

#[test]
fn semantic_executor_advances_to_a_neutral_upload_plan_without_submitting_rhi_work() {
    let resources = ResourceManager::new();
    let texture = register_resource(
        &resources,
        "textures/executor-ready.rgba",
        ResourceKind::Texture,
        Vec::new(),
    );
    let mut residency = RenderAssetResidencyManager::new();
    let mutation = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(texture, 1)],
            &resources.management_generation(),
            &resources.readiness_generation(),
            device_epoch(52, 1),
            demand_generation(2),
        )
        .unwrap_or_else(|error| panic!("residency admission failed: {error:?}"));
    let ticket = request_for(mutation.requests(), texture);
    let bytes = vec![131_u8; 8 * 8 * 4];
    let block = texture_block(&bytes);
    let manifest = RenderArtifactManifest::new(
        texture,
        ticket.asset_revision(),
        Arc::from("windows-dx12-sm6"),
        RenderArtifactLayout::texture(RenderArtifactTextureLayout::new(
            RenderArtifactTextureBlockFormat::new(Arc::from("rgba8unorm"), 1, 1, 4),
            8,
            8,
            1,
            1,
            0,
        )),
        Vec::new(),
        vec![block.clone()],
    )
    .unwrap_or_else(|error| panic!("semantic manifest failed: {error}"));
    let root = ExecutorTestRoot::new();
    let store = root.store();
    let store_limits = RenderArtifactStoreLimits::new(1024 * 1024, 1024 * 1024);
    store
        .publish_block(&block, &bytes, store_limits)
        .unwrap_or_else(|error| panic!("semantic block publication failed: {error}"));
    store
        .publish_manifest(&manifest, store_limits)
        .unwrap_or_else(|error| panic!("semantic manifest publication failed: {error}"));
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .unwrap_or_else(|error| panic!("semantic executor runtime failed: {error}"));
    let (manifest_loader, block_loader) = loaders(store, &runtime);
    let mut executor = RenderAssetSemanticExecutor::try_new(
        manifest_loader,
        block_loader,
        Arc::from("windows-dx12-sm6"),
        executor_limits(2),
        &runtime,
    )
    .unwrap_or_else(|error| panic!("semantic executor failed: {error}"));
    executor
        .admit(
            ticket.clone(),
            RenderArtifactIoPriority::HIGH,
            None,
            &mut residency,
        )
        .unwrap_or_else(|error| panic!("semantic admission failed: {error}"));

    let deadline = Instant::now() + Duration::from_secs(3);
    while executor.diagnostics().ready_uploads() == 0 {
        let report = executor
            .maintain(maintenance_budget(1), &mut residency)
            .unwrap_or_else(|error| panic!("semantic maintenance failed: {error}"));
        assert!(report.load_advances() <= 1);
        assert!(Instant::now() < deadline, "semantic executor timed out");
        std::thread::yield_now();
    }

    assert_eq!(
        residency.state(&ticket),
        Some(RenderAssetResidencyState::QueuedUpload)
    );
    let plan = executor
        .take_next_ready_upload()
        .expect("neutral upload plan");
    assert_eq!(plan.ticket(), ticket);
    assert_eq!(plan.kind(), RenderAssetGpuUploadPlanKind::Texture);
    assert_eq!(executor.diagnostics().ready_uploads(), 0);
}

#[test]
fn semantic_executor_prepare_admission_failure_requeues_the_cpu_lease() {
    let resources = ResourceManager::new();
    let texture = register_resource(
        &resources,
        "textures/executor-prepare-backpressure.rgba",
        ResourceKind::Texture,
        Vec::new(),
    );
    let mut residency = RenderAssetResidencyManager::new();
    let mutation = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(texture, 1)],
            &resources.management_generation(),
            &resources.readiness_generation(),
            device_epoch(53, 1),
            demand_generation(3),
        )
        .unwrap_or_else(|error| panic!("residency admission failed: {error:?}"));
    let ticket = request_for(mutation.requests(), texture);
    let bytes = vec![137_u8; 8 * 8 * 4];
    let block = texture_block(&bytes);
    let manifest = RenderArtifactManifest::new(
        texture,
        ticket.asset_revision(),
        Arc::from("windows-dx12-sm6"),
        RenderArtifactLayout::texture(RenderArtifactTextureLayout::new(
            RenderArtifactTextureBlockFormat::new(Arc::from("rgba8unorm"), 1, 1, 4),
            8,
            8,
            1,
            1,
            0,
        )),
        Vec::new(),
        vec![block.clone()],
    )
    .unwrap_or_else(|error| panic!("semantic manifest failed: {error}"));
    let root = ExecutorTestRoot::new();
    let store = root.store();
    let store_limits = RenderArtifactStoreLimits::new(1024 * 1024, 1024 * 1024);
    store
        .publish_block(&block, &bytes, store_limits)
        .unwrap_or_else(|error| panic!("semantic block publication failed: {error}"));
    store
        .publish_manifest(&manifest, store_limits)
        .unwrap_or_else(|error| panic!("semantic manifest publication failed: {error}"));
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .unwrap_or_else(|error| panic!("semantic executor runtime failed: {error}"));
    let (manifest_loader, block_loader) = loaders(store, &runtime);
    let mut executor = RenderAssetSemanticExecutor::try_new(
        manifest_loader,
        block_loader,
        Arc::from("windows-dx12-sm6"),
        executor_limits(2),
        &runtime,
    )
    .unwrap_or_else(|error| panic!("semantic executor failed: {error}"));
    executor
        .admit(
            ticket.clone(),
            RenderArtifactIoPriority::HIGH,
            None,
            &mut residency,
        )
        .unwrap_or_else(|error| panic!("semantic admission failed: {error}"));
    let load_only_budget = RenderAssetSemanticExecutorMaintenanceBudget::new(
        RenderArtifactManifestIoDispatchBudget::new(2),
        RenderArtifactBlockIoDispatchBudget::new(2, 1024 * 1024),
        1,
        0,
        0,
    );
    let deadline = Instant::now() + Duration::from_secs(3);
    while executor.diagnostics().ready_cpu() == 0 {
        executor
            .maintain(load_only_budget, &mut residency)
            .unwrap_or_else(|error| panic!("semantic load maintenance failed: {error}"));
        assert!(Instant::now() < deadline, "semantic CPU lease timed out");
        std::thread::yield_now();
    }

    executor.close_prepare_admission_for_test();
    let prepare_only_budget = RenderAssetSemanticExecutorMaintenanceBudget::new(
        RenderArtifactManifestIoDispatchBudget::new(1),
        RenderArtifactBlockIoDispatchBudget::new(1, 1),
        0,
        1,
        0,
    );
    assert!(matches!(
        executor.maintain(prepare_only_budget, &mut residency),
        Err(RenderAssetSemanticExecutorMaintenanceError::PrepareExecution(_))
    ));
    assert_eq!(executor.diagnostics().ready_cpu(), 1);
    assert_eq!(executor.diagnostics().preparing_uploads(), 0);
    assert_eq!(
        residency.state(&ticket),
        Some(RenderAssetResidencyState::ReadyCpu)
    );
}

#[test]
fn semantic_executor_close_is_idempotent_and_returns_every_owned_ticket() {
    let resources = ResourceManager::new();
    let texture = register_resource(
        &resources,
        "textures/executor-close.rgba",
        ResourceKind::Texture,
        Vec::new(),
    );
    let mut residency = RenderAssetResidencyManager::new();
    let mutation = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(texture, 1)],
            &resources.management_generation(),
            &resources.readiness_generation(),
            device_epoch(54, 1),
            demand_generation(4),
        )
        .unwrap_or_else(|error| panic!("residency admission failed: {error:?}"));
    let ticket = request_for(mutation.requests(), texture);
    let root = ExecutorTestRoot::new();
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .unwrap_or_else(|error| panic!("semantic executor runtime failed: {error}"));
    let (manifest_loader, block_loader) = loaders(root.store(), &runtime);
    let mut executor = RenderAssetSemanticExecutor::try_new(
        manifest_loader,
        block_loader,
        Arc::from("windows-dx12-sm6"),
        executor_limits(2),
        &runtime,
    )
    .unwrap_or_else(|error| panic!("semantic executor failed: {error}"));
    executor
        .admit(
            ticket.clone(),
            RenderArtifactIoPriority::HIGH,
            None,
            &mut residency,
        )
        .unwrap_or_else(|error| panic!("semantic admission failed: {error}"));

    let close = executor.close();
    assert_eq!(close.tickets(), std::slice::from_ref(&ticket));
    assert_eq!(close.manifest_loader().cancelled_tickets, 1);
    assert_eq!(executor.diagnostics().in_flight(), 0);
    assert!(executor.close().tickets().is_empty());
    assert!(matches!(
        executor.admit(ticket, RenderArtifactIoPriority::HIGH, None, &mut residency),
        Err(RenderAssetSemanticExecutorAdmissionError::Closed)
    ));
    assert!(matches!(
        executor.maintain(maintenance_budget(1), &mut residency),
        Err(RenderAssetSemanticExecutorMaintenanceError::Closed)
    ));
}
