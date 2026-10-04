use std::sync::Arc;

use crate::core::resource::{ResourceId, ResourceKind, UntypedResourceHandle};

use super::*;
use crate::asset::artifact::{
    RenderArtifactBlockCodec, RenderArtifactBlockDescriptor, RenderArtifactContentId,
    RenderArtifactLayout, RenderArtifactManifest, RenderArtifactManifestError,
    RenderArtifactResidencyClass, RenderArtifactTextureBlockFormat, RenderArtifactTextureLayout,
    RenderSubresourceId,
};

const TEST_TEXTURE_WIDTH: u32 = 16;
const TEST_TEXTURE_HEIGHT: u32 = 8;

fn texture_resource(label: &str) -> UntypedResourceHandle {
    UntypedResourceHandle::new(ResourceId::from_stable_label(label), ResourceKind::Texture)
}

fn texture_block(
    mip: u32,
    dependency: Option<u32>,
    residency: RenderArtifactResidencyClass,
) -> RenderArtifactBlockDescriptor {
    let decoded_bytes = u64::from((TEST_TEXTURE_WIDTH >> mip).max(1).div_ceil(4))
        * u64::from((TEST_TEXTURE_HEIGHT >> mip).max(1).div_ceil(4))
        * 16;
    RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip, layer: 0 },
        RenderArtifactContentId::from_bytes([(mip + 1) as u8; 32]),
        RenderArtifactBlockCodec::Raw,
        decoded_bytes,
        decoded_bytes,
        256,
        Arc::from("bc7-rgba-unorm-srgb"),
        residency,
        dependency
            .map(|mip| RenderSubresourceId::TextureMipLayer { mip, layer: 0 })
            .into_iter()
            .collect(),
    )
}

fn texture_layout(mip_count: u32, bootstrap_first_mip: u32) -> RenderArtifactLayout {
    RenderArtifactLayout::texture(RenderArtifactTextureLayout::new(
        RenderArtifactTextureBlockFormat::new(Arc::from("bc7-rgba-unorm-srgb"), 4, 4, 16),
        TEST_TEXTURE_WIDTH,
        TEST_TEXTURE_HEIGHT,
        mip_count,
        1,
        bootstrap_first_mip,
    ))
}

fn texture_manifest() -> RenderArtifactManifest {
    RenderArtifactManifest::new(
        texture_resource("render-plan/texture"),
        9,
        Arc::from("windows-dx12-sm6"),
        texture_layout(4, 2),
        Vec::new(),
        vec![
            texture_block(0, Some(1), RenderArtifactResidencyClass::Streamable),
            texture_block(1, Some(2), RenderArtifactResidencyClass::Streamable),
            texture_block(2, Some(3), RenderArtifactResidencyClass::Bootstrap),
            texture_block(3, None, RenderArtifactResidencyClass::Bootstrap),
        ],
    )
    .unwrap_or_else(|error| panic!("texture load-plan fixture failed: {error}"))
}

#[test]
fn render_manifest_rejects_bootstrap_dependency_on_streamable_quality() {
    let result = RenderArtifactManifest::new(
        texture_resource("render-plan/invalid-bootstrap-edge"),
        1,
        Arc::from("windows-dx12-sm6"),
        texture_layout(3, 2),
        Vec::new(),
        vec![
            texture_block(0, None, RenderArtifactResidencyClass::Streamable),
            texture_block(1, None, RenderArtifactResidencyClass::Streamable),
            texture_block(2, Some(1), RenderArtifactResidencyClass::Bootstrap),
        ],
    );

    assert!(matches!(
        result,
        Err(RenderArtifactManifestError::BootstrapDependsOnStreamable {
            subresource: RenderSubresourceId::TextureMipLayer { mip: 2, layer: 0 },
            dependency: RenderSubresourceId::TextureMipLayer { mip: 1, layer: 0 },
        })
    ));
}

#[test]
fn render_bootstrap_load_plan_excludes_streamable_blocks_and_orders_dependencies_first() {
    let plan = texture_manifest()
        .load_plan(RenderArtifactLoadScope::Bootstrap)
        .unwrap_or_else(|error| panic!("bootstrap load plan failed: {error}"));

    assert_eq!(plan.block_count(), 2);
    assert_eq!(plan.batches().len(), 2);
    assert_eq!(
        plan.batches()[0].blocks()[0].subresource(),
        RenderSubresourceId::TextureMipLayer { mip: 3, layer: 0 }
    );
    assert_eq!(
        plan.batches()[1].blocks()[0].subresource(),
        RenderSubresourceId::TextureMipLayer { mip: 2, layer: 0 }
    );
    assert_eq!(plan.total_encoded_bytes(), 32);
    assert_eq!(plan.total_decoded_bytes(), 32);
}

#[test]
fn render_all_quality_load_plan_emits_one_deterministic_batch_per_dependency_frontier() {
    let manifest = texture_manifest();
    let first = manifest
        .load_plan(RenderArtifactLoadScope::All)
        .unwrap_or_else(|error| panic!("all-quality load plan failed: {error}"));
    let second = manifest
        .load_plan(RenderArtifactLoadScope::All)
        .unwrap_or_else(|error| panic!("repeated all-quality load plan failed: {error}"));

    assert_eq!(first, second);
    assert_eq!(first.block_count(), 4);
    assert_eq!(first.batches().len(), 4);
    assert_eq!(
        first
            .batches()
            .iter()
            .flat_map(RenderArtifactLoadBatch::blocks)
            .map(RenderArtifactBlockDescriptor::subresource)
            .collect::<Vec<_>>(),
        vec![
            RenderSubresourceId::TextureMipLayer { mip: 3, layer: 0 },
            RenderSubresourceId::TextureMipLayer { mip: 2, layer: 0 },
            RenderSubresourceId::TextureMipLayer { mip: 1, layer: 0 },
            RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 },
        ]
    );
}
