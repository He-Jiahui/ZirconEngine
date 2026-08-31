use std::sync::Arc;

use crate::asset::artifact::{
    RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1, RenderArtifactBlockCodec, RenderArtifactBlockDescriptor,
    RenderArtifactContentId, RenderArtifactLayout, RenderArtifactManifest,
    RenderArtifactMeshBounds, RenderArtifactMeshIndexFormat, RenderArtifactMeshLayout,
    RenderArtifactMeshLodLayout, RenderArtifactMeshVertexFormat, RenderArtifactResidencyClass,
    RenderArtifactTextureBlockFormat, RenderArtifactTextureLayout, RenderSubresourceId,
};
use crate::core::framework::render::RenderMeshTopology;
use crate::core::resource::{ResourceId, ResourceKind, UntypedResourceHandle};

use super::contract::{
    RenderAssetGpuUploadBudgetClass, RenderAssetGpuUploadLimits, RenderAssetGpuUploadPlanError,
};
use super::plan::{PreparedRenderAssetGpuUpload, UploadSourceBlock, prepare_upload};

fn resource(label: &str, kind: ResourceKind) -> UntypedResourceHandle {
    UntypedResourceHandle::new(ResourceId::from_stable_label(label), kind)
}

fn descriptor(
    seed: u8,
    subresource: RenderSubresourceId,
    bytes: u64,
    format: &str,
    residency: RenderArtifactResidencyClass,
) -> RenderArtifactBlockDescriptor {
    RenderArtifactBlockDescriptor::new(
        subresource,
        RenderArtifactContentId::from_bytes([seed; 32]),
        RenderArtifactBlockCodec::Raw,
        bytes,
        bytes,
        256,
        Arc::from(format),
        residency,
        Vec::new(),
    )
}

fn source(descriptor: &RenderArtifactBlockDescriptor) -> UploadSourceBlock {
    UploadSourceBlock::new(
        descriptor.clone(),
        Arc::from(vec![
            descriptor.content_id().as_bytes()[0];
            descriptor.decoded_bytes() as usize
        ]),
    )
}

#[test]
fn texture_tail_upload_remaps_source_mips_to_one_compact_physical_chain() {
    let layout = RenderArtifactTextureLayout::new(
        RenderArtifactTextureBlockFormat::new(Arc::from("rgba8unorm"), 1, 1, 4),
        16,
        8,
        3,
        2,
        1,
    );
    let mut blocks = Vec::new();
    for mip in 0..3 {
        let subresource_layout = layout
            .subresource_layout(mip, 0)
            .unwrap_or_else(|| panic!("missing test mip {mip}"));
        for layer in 0..2 {
            blocks.push(descriptor(
                (mip * 2 + layer + 1) as u8,
                RenderSubresourceId::TextureMipLayer { mip, layer },
                subresource_layout.decoded_bytes(),
                "rgba8unorm",
                if mip >= 1 {
                    RenderArtifactResidencyClass::Bootstrap
                } else {
                    RenderArtifactResidencyClass::Streamable
                },
            ));
        }
    }
    let manifest = RenderArtifactManifest::new(
        resource("gpu-upload/texture-tail", ResourceKind::Texture),
        1,
        Arc::from("windows-dx12-sm6"),
        RenderArtifactLayout::texture(layout),
        Vec::new(),
        blocks.clone(),
    )
    .unwrap_or_else(|error| panic!("texture test manifest failed: {error}"));
    let selected = blocks[2..].iter().map(source).collect();

    let (prepared, quote) = prepare_upload(
        &manifest,
        selected,
        RenderAssetGpuUploadLimits::new(8, 1024, 1024),
    )
    .unwrap_or_else(|error| panic!("texture upload planning failed: {error}"));
    let PreparedRenderAssetGpuUpload::Texture(texture) = prepared else {
        panic!("texture manifest produced a mesh upload")
    };

    assert_eq!(texture.source_mips, 1..3);
    assert_eq!((texture.width, texture.height), (8, 4));
    assert_eq!(texture.layer_count, 2);
    assert_eq!(texture.uploads.len(), 4);
    assert_eq!(texture.uploads[0].region.mip_level, 0);
    assert_eq!(texture.uploads[2].region.mip_level, 1);
    assert_eq!(quote.subresource_count(), 4);
    assert_eq!(quote.staging_bytes(), 320);
    assert_eq!(quote.destination_bytes(), 320);
}

#[test]
fn mesh_lods_pack_into_two_asset_buffers_without_payload_repacking() {
    let layout = RenderArtifactMeshLayout::new(
        Arc::from(RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1),
        RenderArtifactMeshVertexFormat::StaticMeshV1,
        RenderArtifactMeshIndexFormat::Uint32,
        1,
        vec![
            RenderArtifactMeshLodLayout::new(
                0,
                RenderMeshTopology::TriangleList,
                2,
                3,
                192,
                RenderArtifactMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
            ),
            RenderArtifactMeshLodLayout::new(
                1,
                RenderMeshTopology::TriangleList,
                1,
                3,
                96,
                RenderArtifactMeshBounds::from_min_max([-0.5; 3], [0.5; 3]),
            ),
        ],
    );
    let block0 = descriptor(
        1,
        RenderSubresourceId::MeshLod { lod: 0 },
        204,
        RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1,
        RenderArtifactResidencyClass::Streamable,
    );
    let block1 = descriptor(
        2,
        RenderSubresourceId::MeshLod { lod: 1 },
        108,
        RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1,
        RenderArtifactResidencyClass::Bootstrap,
    );
    let manifest = RenderArtifactManifest::new(
        resource("gpu-upload/mesh-lods", ResourceKind::Mesh),
        3,
        Arc::from("windows-dx12-sm6"),
        RenderArtifactLayout::mesh(layout),
        Vec::new(),
        vec![block0.clone(), block1.clone()],
    )
    .unwrap_or_else(|error| panic!("mesh test manifest failed: {error}"));

    let (prepared, quote) = prepare_upload(
        &manifest,
        vec![source(&block1), source(&block0)],
        RenderAssetGpuUploadLimits::new(2, 312, 312),
    )
    .unwrap_or_else(|error| panic!("mesh upload planning failed: {error}"));
    let PreparedRenderAssetGpuUpload::Mesh(mesh) = prepared else {
        panic!("mesh manifest produced a texture upload")
    };

    assert_eq!(mesh.lods.len(), 2);
    assert_eq!(mesh.vertex_buffer_bytes, 288);
    assert_eq!(mesh.index_buffer_bytes, 24);
    assert_eq!(mesh.lods[0].vertex_destination_offset, 0);
    assert_eq!(mesh.lods[1].vertex_destination_offset, 192);
    assert_eq!(mesh.lods[0].index_destination_offset, 0);
    assert_eq!(mesh.lods[1].index_destination_offset, 12);
    assert_eq!(mesh.lods[0].vertex_source_range, 0..192);
    assert_eq!(mesh.lods[0].index_source_range, 192..204);
    assert_eq!(quote.staging_bytes(), 312);
    assert_eq!(quote.destination_bytes(), 312);
}

#[test]
fn upload_budget_rejects_the_whole_asset_before_native_resource_creation() {
    let layout = RenderArtifactTextureLayout::new(
        RenderArtifactTextureBlockFormat::new(Arc::from("rgba8unorm"), 1, 1, 4),
        8,
        8,
        1,
        1,
        0,
    );
    let block = descriptor(
        7,
        RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 },
        256,
        "rgba8unorm",
        RenderArtifactResidencyClass::Bootstrap,
    );
    let manifest = RenderArtifactManifest::new(
        resource("gpu-upload/budget", ResourceKind::Texture),
        1,
        Arc::from("windows-dx12-sm6"),
        RenderArtifactLayout::texture(layout),
        Vec::new(),
        vec![block.clone()],
    )
    .unwrap_or_else(|error| panic!("budget test manifest failed: {error}"));

    let result = prepare_upload(
        &manifest,
        vec![source(&block)],
        RenderAssetGpuUploadLimits::new(1, 255, 1024),
    );

    assert!(matches!(
        result,
        Err(RenderAssetGpuUploadPlanError::BudgetExceeded {
            class: RenderAssetGpuUploadBudgetClass::Staging,
            requested: 256,
            limit: 255,
        })
    ));
}

#[test]
fn upload_execution_uses_neutral_rhi_batches_and_retains_the_cpu_lease() {
    let source = include_str!("submit.rs");

    assert!(source.contains("cpu_lease: super::super::RenderAssetCpuArtifactLease"));
    assert!(source.contains("device: &dyn RenderDevice"));
    assert_eq!(
        source.matches("device.write_texture_batch(batch)").count(),
        1
    );
    assert_eq!(
        source.matches("device.write_buffer_batch(batch)").count(),
        1
    );
    assert!(source.contains("texture: TextureHandle"));
    assert!(source.contains("vertex_buffer: BufferHandle"));
    assert!(!source.contains("wgpu::"));
    assert!(!source.contains("RenderBackend"));
    assert!(!source.contains("queue.submit"));
    assert!(!source.contains("create_buffer_init"));
}

#[test]
fn residency_manager_owns_active_in_flight_and_retirement_artifacts() {
    let source = include_str!("../gpu_residency.rs");
    let manager = include_str!("../manager.rs");

    assert!(source.contains("retiring_uploads: HashMap<SubmissionTicket"));
    assert!(source.contains("ready_retirements: VecDeque<RenderAssetGpuArtifact>"));
    assert!(source.contains("max_ready_retirements"));
    assert!(source.contains("ensure_ready_retirement_capacity(1)"));
    assert_eq!(
        source
            .matches("ready_retirements.push_back(artifact)")
            .count(),
        1
    );
    assert!(source.contains("entry.active_artifact.replace(artifact)"));
    assert!(source.contains("entry.pending_upload.take()"));
    assert!(manager.contains("self.gpu.detach_entry(&mut entry)"));
    assert!(manager.contains("self.gpu.detach_pending_upload(entry)"));
    assert!(manager.contains("ensure_ready_retirement_capacity(requested_retirements)"));
    assert!(
        manager
            .find("ensure_ready_retirement_capacity(requested_retirements)")
            .unwrap_or(usize::MAX)
            < manager
                .find("self.issue_reference_change_tickets(&mut prepared)")
                .unwrap_or(usize::MAX)
    );

    let artifact = include_str!("submit.rs");
    assert!(artifact.contains("retirement_progress: u8"));
    assert!(artifact.contains("Result<(), (Self, RhiError)>"));
}
