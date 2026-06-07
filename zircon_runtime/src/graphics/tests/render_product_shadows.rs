use crate::core::framework::render::{
    FallbackSkyboxKind, PostProcessGraphResourceNames, PreviewEnvironmentExtract,
    RenderFrameExtract, RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::math::Vec4;
use crate::render_graph::{
    QueueLane, RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
    RenderGraphResourceAccessKind, RenderGraphResourceDesc, RenderGraphResourceKind,
};
use crate::rhi::{TextureFormat, TextureUsage};
use crate::RenderPipelineAsset;

#[test]
fn shadow_map_pass_stays_live_as_depth_only_graph_contract() {
    for compiled in [
        RenderPipelineAsset::default_forward_plus()
            .compile(&test_extract())
            .unwrap(),
        RenderPipelineAsset::default_deferred()
            .compile(&test_extract())
            .unwrap(),
    ] {
        let shadow_pass = compiled
            .graph
            .passes()
            .iter()
            .find(|pass| pass.name == "shadow-map")
            .expect("default Core3d pipelines should compile a shadow-map pass");

        assert!(
            !shadow_pass.culled,
            "shadow-map is a side-effectful renderer slot and must stay visible until concrete shadow consumers sample it"
        );
        assert!(shadow_pass.flags.has_side_effects);
        assert_eq!(shadow_pass.executor_id.as_deref(), Some("shadow.map"));
        assert_eq!(shadow_pass.queue, QueueLane::Graphics);

        let shadow_write = pass_resource_access(
            &compiled,
            "shadow-map",
            "shadow-map",
            RenderGraphResourceAccessKind::Write,
        );
        assert_eq!(
            shadow_write.attachment_ops,
            Some(RenderGraphAttachmentOps {
                load: RenderGraphAttachmentLoadOp::Clear,
                store: RenderGraphAttachmentStoreOp::Store,
            }),
            "shadow map placeholder should clear/store its depth attachment contract"
        );

        let shadow_lifetime = graph_resource_lifetime(&compiled, "shadow-map");
        assert_eq!(
            shadow_lifetime.kind,
            RenderGraphResourceKind::TransientTexture
        );
        assert!(matches!(
            &shadow_lifetime.desc,
            RenderGraphResourceDesc::Texture(desc)
                if desc.format == TextureFormat::Depth32Float
                    && desc.sample_count == 1
                    && desc.usage.contains(TextureUsage::RENDER_ATTACHMENT)
                    && desc.usage.contains(TextureUsage::SAMPLED)
                    && desc.usage.contains(TextureUsage::COPY_SRC)
                    && !desc.usage.contains(TextureUsage::STORAGE)
        ));
    }
}

#[test]
fn deferred_lighting_reads_shadow_map_for_receiver_sampling() {
    let compiled = RenderPipelineAsset::default_deferred()
        .compile(&test_extract())
        .unwrap();

    pass_resource_access(
        &compiled,
        "deferred-lighting",
        PostProcessGraphResourceNames::SHADOW_MAP,
        RenderGraphResourceAccessKind::Read,
    );
}

#[test]
fn forward_mesh_passes_read_shadow_map_for_receiver_sampling() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .unwrap();

    for pass_name in ["opaque-mesh", "alpha-mask-mesh", "transparent-mesh"] {
        pass_resource_access(
            &compiled,
            pass_name,
            PostProcessGraphResourceNames::SHADOW_MAP,
            RenderGraphResourceAccessKind::Read,
        );
    }
}

#[test]
fn deferred_transparent_mesh_reads_shadow_map_for_receiver_sampling() {
    let compiled = RenderPipelineAsset::default_deferred()
        .compile(&test_extract())
        .unwrap();

    pass_resource_access(
        &compiled,
        "transparent-mesh",
        PostProcessGraphResourceNames::SHADOW_MAP,
        RenderGraphResourceAccessKind::Read,
    );
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}

fn pass_resource_access<'a>(
    compiled: &'a crate::CompiledRenderPipeline,
    pass_name: &str,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> &'a crate::render_graph::RenderGraphPassResourceAccess {
    compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .and_then(|pass| {
            pass.resources
                .iter()
                .find(|resource| resource.name == resource_name && resource.access == access)
        })
        .unwrap_or_else(|| panic!("pass `{pass_name}` should {access:?} `{resource_name}`"))
}

fn graph_resource_lifetime<'a>(
    compiled: &'a crate::CompiledRenderPipeline,
    resource_name: &str,
) -> &'a crate::render_graph::RenderGraphResourceLifetime {
    compiled
        .graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == resource_name)
        .unwrap_or_else(|| panic!("compiled graph should contain resource `{resource_name}`"))
}
