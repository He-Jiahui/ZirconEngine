use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
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
use crate::core::runtime::{EngineTaskGraph, EngineTaskGraphOptions};
use crate::graphics::scene::render_scene::RenderSceneResourceReferenceDelta;

use super::super::{
    RenderAssetCpuBlockLease, RenderAssetSemanticBlockLoad, RenderAssetSemanticBlockLoadAdvance,
    RenderAssetSemanticLoad, RenderAssetSemanticLoadAdvance, RenderAssetSemanticLoadStage,
};
use super::*;

static NEXT_SEMANTIC_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

struct SemanticTestRoot(std::path::PathBuf);

impl SemanticTestRoot {
    fn new() -> Self {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(".codex_tmp")
            .join("render-semantic-residency-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT_SEMANTIC_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(&root)
            .unwrap_or_else(|error| panic!("failed to create semantic test root: {error}"));
        Self(root)
    }

    fn store(&self) -> RenderArtifactStore {
        RenderArtifactStore::new(self.0.clone())
    }
}

impl Drop for SemanticTestRoot {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        }
    }
}

fn loader(store: RenderArtifactStore) -> (EngineTaskGraph, RenderArtifactBlockLoader) {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .unwrap_or_else(|error| panic!("semantic test runtime failed: {error}"));
    let limits = RenderArtifactBlockLoaderLimits::new(
        8,
        16,
        4,
        1024 * 1024,
        1024 * 1024,
        RenderArtifactStoreLimits::new(1024 * 1024, 1024 * 1024),
    );
    let loader = RenderArtifactBlockLoader::new(store, limits, &runtime)
        .unwrap_or_else(|error| panic!("semantic test loader failed: {error}"));
    (runtime, loader)
}

fn manifest_loader(
    store: RenderArtifactStore,
    runtime: &EngineTaskGraph,
) -> RenderArtifactManifestLoader {
    RenderArtifactManifestLoader::new(
        store,
        RenderArtifactManifestLoaderLimits::new(
            8,
            16,
            4,
            16 * 1024 * 1024,
            RenderArtifactStoreLimits::new(1024 * 1024, 1024 * 1024),
        ),
        runtime,
    )
    .unwrap_or_else(|error| panic!("semantic manifest loader failed: {error}"))
}

fn texture_block(mip: u32, bytes: &[u8], dependency: Option<u32>) -> RenderArtifactBlockDescriptor {
    RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip, layer: 0 },
        RenderArtifactContentId::from_bytes(*blake3::hash(bytes).as_bytes()),
        RenderArtifactBlockCodec::Raw,
        bytes.len() as u64,
        bytes.len() as u64,
        256,
        Arc::from("rgba8unorm"),
        RenderArtifactResidencyClass::Bootstrap,
        dependency
            .map(|mip| RenderSubresourceId::TextureMipLayer { mip, layer: 0 })
            .into_iter()
            .collect(),
    )
}

fn wait_for_next_batch(
    mut load: RenderAssetSemanticBlockLoad,
    loader: &RenderArtifactBlockLoader,
) -> RenderAssetSemanticBlockLoad {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match load.advance(loader) {
            Ok(RenderAssetSemanticBlockLoadAdvance::Pending(next, _)) => {
                load = next;
                let diagnostics = loader.diagnostics();
                if diagnostics.queued_io_entries == 1 && diagnostics.submitted_io_tasks == 1 {
                    return load;
                }
            }
            Ok(RenderAssetSemanticBlockLoadAdvance::Deferred(_, error)) => {
                panic!("semantic load unexpectedly deferred: {error}")
            }
            Ok(RenderAssetSemanticBlockLoadAdvance::Ready(_)) => {
                panic!("semantic load skipped its dependency batch")
            }
            Err(error) => panic!("semantic load failed: {error}"),
        }
        assert!(
            Instant::now() < deadline,
            "semantic dependency batch timed out"
        );
        std::thread::yield_now();
    }
}

#[test]
fn render_semantic_route_admits_dependency_batches_in_order_and_retains_cpu_budget() {
    let resources = crate::core::resource::ResourceManager::new();
    let texture = register_resource(
        &resources,
        "textures/semantic-route.rgba",
        crate::core::resource::ResourceKind::Texture,
        Vec::new(),
    );
    let mut residency = RenderAssetResidencyManager::new();
    let mutation = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(texture, 1)],
            &resources.management_generation(),
            &resources.readiness_generation(),
            device_epoch(17, 1),
            demand_generation(1),
        )
        .unwrap_or_else(|error| panic!("semantic residency admission failed: {error:?}"));
    let ticket = request_for(mutation.requests(), texture);
    let coarse_bytes = vec![89_u8; 8 * 8 * 4];
    let fine_bytes = vec![97_u8; 16 * 16 * 4];
    let coarse = texture_block(1, &coarse_bytes, None);
    let fine = texture_block(0, &fine_bytes, Some(1));
    let manifest = RenderArtifactManifest::new(
        texture,
        ticket.asset_revision(),
        Arc::from("windows-dx12-sm6"),
        RenderArtifactLayout::texture(RenderArtifactTextureLayout::new(
            RenderArtifactTextureBlockFormat::new(Arc::from("rgba8unorm"), 1, 1, 4),
            16,
            16,
            2,
            1,
            0,
        )),
        Vec::new(),
        vec![fine.clone(), coarse.clone()],
    )
    .unwrap_or_else(|error| panic!("semantic manifest failed: {error}"));
    let root = SemanticTestRoot::new();
    let store = root.store();
    let limits = RenderArtifactStoreLimits::new(1024 * 1024, 1024 * 1024);
    for (block, bytes) in [
        (&coarse, coarse_bytes.as_slice()),
        (&fine, fine_bytes.as_slice()),
    ] {
        store
            .publish_block(block, bytes, limits)
            .unwrap_or_else(|error| panic!("semantic block publication failed: {error}"));
    }
    let (_runtime, loader) = loader(store);
    let load = RenderAssetSemanticBlockLoad::begin(
        ticket,
        &manifest,
        &loader,
        RenderArtifactIoPriority::HIGH,
        None,
    )
    .unwrap_or_else(|error| panic!("semantic route begin failed: {error}"));

    assert_eq!(loader.diagnostics().queued_io_entries, 1);
    loader
        .dispatch_io(RenderArtifactBlockIoDispatchBudget::new(1, u64::MAX))
        .unwrap_or_else(|error| panic!("coarse dispatch failed: {error}"));
    let load = wait_for_next_batch(load, &loader);
    assert_eq!(loader.diagnostics().live_entries, 2);
    loader
        .dispatch_io(RenderArtifactBlockIoDispatchBudget::new(1, u64::MAX))
        .unwrap_or_else(|error| panic!("fine dispatch failed: {error}"));

    let lease = finish_semantic_load(load, &loader, Instant::now() + Duration::from_secs(3));

    assert_eq!(lease.blocks().len(), 2);
    assert_eq!(
        lease.blocks()[0].descriptor().subresource(),
        RenderSubresourceId::TextureMipLayer { mip: 1, layer: 0 }
    );
    assert_eq!(loader.diagnostics().live_entries, 2);
    drop(lease);
    let diagnostics = loader.diagnostics();
    assert_eq!(diagnostics.live_entries, 0);
    assert_eq!(diagnostics.live_tickets, 0);
    assert_eq!(diagnostics.retained_bytes, 0);
}

#[test]
fn render_semantic_route_owns_manifest_and_blocks_until_cpu_lease_drop() {
    let resources = crate::core::resource::ResourceManager::new();
    let texture = register_resource(
        &resources,
        "textures/semantic-manifest-route.rgba",
        crate::core::resource::ResourceKind::Texture,
        Vec::new(),
    );
    let mut residency = RenderAssetResidencyManager::new();
    let mutation = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(texture, 1)],
            &resources.management_generation(),
            &resources.readiness_generation(),
            device_epoch(18, 1),
            demand_generation(2),
        )
        .unwrap_or_else(|error| panic!("semantic residency admission failed: {error:?}"));
    let ticket = request_for(mutation.requests(), texture);
    let bytes = vec![113_u8; 8 * 8 * 4];
    let block = texture_block(0, &bytes, None);
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
    let root = SemanticTestRoot::new();
    let store = root.store();
    let limits = RenderArtifactStoreLimits::new(1024 * 1024, 1024 * 1024);
    store
        .publish_block(&block, &bytes, limits)
        .unwrap_or_else(|error| panic!("semantic block publication failed: {error}"));
    store
        .publish_manifest(&manifest, limits)
        .unwrap_or_else(|error| panic!("semantic manifest publication failed: {error}"));
    let (runtime, block_loader) = loader(store.clone());
    let manifest_loader = manifest_loader(store, &runtime);
    let mut load = RenderAssetSemanticLoad::begin(
        ticket,
        Arc::from("windows-dx12-sm6"),
        &manifest_loader,
        RenderArtifactIoPriority::HIGH,
        None,
    )
    .unwrap_or_else(|error| panic!("semantic load begin failed: {error}"));

    assert_eq!(manifest_loader.diagnostics().queued_io_entries, 1);
    manifest_loader
        .dispatch_io(RenderArtifactManifestIoDispatchBudget::new(1))
        .unwrap_or_else(|error| panic!("semantic manifest dispatch failed: {error}"));
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match load.advance(&block_loader) {
            Ok(RenderAssetSemanticLoadAdvance::Pending(next, stage)) => {
                load = next;
                if matches!(stage, RenderAssetSemanticLoadStage::Blocks(_)) {
                    break;
                }
            }
            Ok(RenderAssetSemanticLoadAdvance::Deferred(_, error)) => {
                panic!("semantic load unexpectedly deferred: {error}")
            }
            Ok(RenderAssetSemanticLoadAdvance::Ready(_)) => {
                panic!("semantic load completed before block dispatch")
            }
            Err(error) => panic!("semantic load failed: {error}"),
        }
        assert!(
            Instant::now() < deadline,
            "semantic manifest load timed out"
        );
        std::thread::yield_now();
    }

    assert_eq!(manifest_loader.diagnostics().live_entries, 1);
    assert_eq!(block_loader.diagnostics().queued_io_entries, 1);
    block_loader
        .dispatch_io(RenderArtifactBlockIoDispatchBudget::new(1, u64::MAX))
        .unwrap_or_else(|error| panic!("semantic block dispatch failed: {error}"));
    let lease = loop {
        match load.advance(&block_loader) {
            Ok(RenderAssetSemanticLoadAdvance::Pending(next, _)) => load = next,
            Ok(RenderAssetSemanticLoadAdvance::Ready(lease)) => break lease,
            Ok(RenderAssetSemanticLoadAdvance::Deferred(_, error)) => {
                panic!("semantic load unexpectedly deferred: {error}")
            }
            Err(error) => panic!("semantic load failed: {error}"),
        }
        assert!(Instant::now() < deadline, "semantic block load timed out");
        std::thread::yield_now();
    };

    assert_eq!(lease.manifest().as_ref(), &manifest);
    assert_eq!(lease.blocks().len(), 1);
    assert_eq!(manifest_loader.diagnostics().live_entries, 1);
    assert_eq!(block_loader.diagnostics().live_entries, 1);
    drop(lease);
    assert_eq!(manifest_loader.diagnostics().live_entries, 0);
    assert_eq!(block_loader.diagnostics().live_entries, 0);
}

fn finish_semantic_load(
    mut load: RenderAssetSemanticBlockLoad,
    loader: &RenderArtifactBlockLoader,
    deadline: Instant,
) -> RenderAssetCpuBlockLease {
    loop {
        match load.advance(loader) {
            Ok(RenderAssetSemanticBlockLoadAdvance::Pending(next, _)) => load = next,
            Ok(RenderAssetSemanticBlockLoadAdvance::Ready(lease)) => return lease,
            Ok(RenderAssetSemanticBlockLoadAdvance::Deferred(_, error)) => {
                panic!("semantic load unexpectedly deferred: {error}")
            }
            Err(error) => panic!("semantic load failed: {error}"),
        }
        assert!(Instant::now() < deadline, "semantic load timed out");
        std::thread::yield_now();
    }
}
