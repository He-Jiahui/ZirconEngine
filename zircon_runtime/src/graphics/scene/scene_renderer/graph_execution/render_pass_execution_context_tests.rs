use super::{
    RenderPassExecutionContext, preserve_physical_output_attachment_ops_for_partitioned_viewport,
};
use crate::core::framework::render::{
    CameraRenderDescriptor, PostProcessGraphResourceNames, RenderViewportRect,
    ViewportCameraSnapshot,
};
use crate::core::math::UVec2;
use crate::graphics::RenderPassExecutorId;
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::{
    QueueLane, RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
    RenderGraphBuilder, RenderGraphPassResourceAccess, RenderGraphResource,
    RenderGraphResourceAccessId, RenderGraphResourceAccessKind, RenderGraphResourceKind,
    RenderPassId,
};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

#[test]
fn metadata_context_reports_missing_gpu_payload() {
    let mut context = RenderPassExecutionContext::new(
        "particle-render",
        RenderPassExecutorId::new("particle.transparent"),
    );

    assert!(context.gpu().is_none());
    assert_eq!(
        context.require_gpu().unwrap_err(),
        "render pass executor `particle.transparent` for pass `particle-render` requires renderer GPU context"
    );
}

#[test]
fn metadata_context_exposes_only_cardinality_matched_compiled_access_ids() {
    let pass_id = RenderPassId::from_index(3, 7);
    let access_ids = [RenderGraphResourceAccessId::new(pass_id, 0)];
    let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "opaque-mesh",
        RenderPassExecutorId::new("mesh.opaque"),
        QueueLane::Graphics,
        Default::default(),
        vec![RenderGraphPassResourceAccess {
            name: "scene-color".to_string(),
            kind: RenderGraphResourceKind::TransientTexture,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: Some(RenderGraphAttachmentOps::clear_store()),
        }],
    )
    .with_compiled_access_ids(pass_id, &access_ids)
    .expect("one compiled access ID matches the declared resource row");

    assert_eq!(context.compiled_access_ids(), Some(access_ids.as_slice()));
    let error = RenderPassExecutionContext::new("empty", RenderPassExecutorId::new("mesh.empty"))
        .with_compiled_access_ids(pass_id, &access_ids)
        .expect_err("an unmatched compiled access list must fail closed");
    assert!(error.contains("access identity count"));
}

#[test]
fn metadata_context_rejects_foreign_or_out_of_order_compiled_access_ids() {
    let pass_id = RenderPassId::from_index(3, 7);
    let foreign_pass_id = RenderPassId::from_index(4, 8);
    let resources = vec![
        RenderGraphPassResourceAccess {
            name: "scene-color".to_string(),
            kind: RenderGraphResourceKind::TransientTexture,
            access: RenderGraphResourceAccessKind::Read,
            attachment_ops: None,
        },
        RenderGraphPassResourceAccess {
            name: "scene-depth".to_string(),
            kind: RenderGraphResourceKind::TransientTexture,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: Some(RenderGraphAttachmentOps::clear_store()),
        },
    ];

    let foreign_error = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "opaque-mesh",
        RenderPassExecutorId::new("mesh.opaque"),
        QueueLane::Graphics,
        Default::default(),
        resources.clone(),
    )
    .with_compiled_access_ids(
        pass_id,
        &[
            RenderGraphResourceAccessId::new(foreign_pass_id, 0),
            RenderGraphResourceAccessId::new(foreign_pass_id, 1),
        ],
    )
    .expect_err("foreign-pass identities must not enter an executor context");
    assert!(foreign_error.contains("belongs to pass"));

    let reversed_error = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "opaque-mesh",
        RenderPassExecutorId::new("mesh.opaque"),
        QueueLane::Graphics,
        Default::default(),
        resources,
    )
    .with_compiled_access_ids(
        pass_id,
        &[
            RenderGraphResourceAccessId::new(pass_id, 1),
            RenderGraphResourceAccessId::new(pass_id, 0),
        ],
    )
    .expect_err("out-of-order identities must not enter an executor context");
    assert!(reversed_error.contains("access ordinal"));
}

#[test]
fn metadata_context_exposes_attachment_ops_for_written_resource() {
    let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "transparent-mesh",
        RenderPassExecutorId::new("mesh.transparent"),
        QueueLane::Graphics,
        Default::default(),
        vec![
            RenderGraphPassResourceAccess {
                name: "scene-color".to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            },
            RenderGraphPassResourceAccess {
                name: "scene-color".to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Write,
                attachment_ops: Some(RenderGraphAttachmentOps::load_store()),
            },
        ],
    );

    assert_eq!(
        context.attachment_ops_for_write("scene-color"),
        Some(RenderGraphAttachmentOps::load_store())
    );
    assert_eq!(context.attachment_ops_for_write("scene-depth"), None);
}

#[test]
fn partitioned_physical_output_clear_loads_existing_target_contents() {
    let viewport_size = UVec2::new(640, 360);
    let render_region = split_viewport_region(viewport_size);
    let clear_discard = RenderGraphAttachmentOps::clear_discard();

    assert_eq!(
        preserve_physical_output_attachment_ops_for_partitioned_viewport(
            PostProcessGraphResourceNames::FINAL_COLOR,
            clear_discard,
            render_region,
            viewport_size,
        ),
        RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Discard,
        }
    );
}

#[test]
fn full_physical_output_and_transient_resources_preserve_graph_attachment_ops() {
    let viewport_size = UVec2::new(640, 360);
    let full_region = ViewportRenderRegion::full_target(viewport_size);
    let split_region = split_viewport_region(viewport_size);

    assert_eq!(
        preserve_physical_output_attachment_ops_for_partitioned_viewport(
            PostProcessGraphResourceNames::FINAL_COLOR,
            RenderGraphAttachmentOps::clear_store(),
            full_region,
            viewport_size,
        ),
        RenderGraphAttachmentOps::clear_store()
    );
    assert_eq!(
        preserve_physical_output_attachment_ops_for_partitioned_viewport(
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphAttachmentOps::clear_store(),
            split_region,
            viewport_size,
        ),
        RenderGraphAttachmentOps::clear_store()
    );
}

#[test]
fn metadata_context_reports_declared_texture_reads() {
    let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "opaque-mesh",
        RenderPassExecutorId::new("mesh.opaque"),
        QueueLane::Graphics,
        Default::default(),
        vec![
            RenderGraphPassResourceAccess {
                name: "shadow-atlas".to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            },
            RenderGraphPassResourceAccess {
                name: "scene-color".to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Write,
                attachment_ops: Some(RenderGraphAttachmentOps::load_store()),
            },
        ],
    );

    assert!(context.reads_texture("shadow-atlas"));
    assert!(context.reads_transient_texture("shadow-atlas"));
    assert!(!context.reads_texture("scene-color"));
}

fn split_viewport_region(viewport_size: UVec2) -> ViewportRenderRegion {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    camera.viewport_rect = Some(RenderViewportRect::new(
        UVec2::new(viewport_size.x / 2, 0),
        UVec2::new(viewport_size.x / 2, viewport_size.y),
    ));
    ViewportRenderRegion::from_camera(Some(&camera), viewport_size)
}

#[test]
fn metadata_context_keeps_external_reads_out_of_transient_texture_reads() {
    let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "opaque-mesh",
        RenderPassExecutorId::new("mesh.opaque"),
        QueueLane::Graphics,
        Default::default(),
        vec![RenderGraphPassResourceAccess {
            name: "shadow-atlas".to_string(),
            kind: RenderGraphResourceKind::External,
            access: RenderGraphResourceAccessKind::Read,
            attachment_ops: None,
        }],
    );

    assert!(context.reads_texture("shadow-atlas"));
    assert!(!context.reads_transient_texture("shadow-atlas"));
}

#[test]
fn metadata_context_resolves_pass_resource_handles() {
    let mut builder = RenderGraphBuilder::new("resolver-context");
    let depth = builder.create_texture(TextureDesc::new(
        "scene-depth",
        32,
        32,
        TextureFormat::Depth32Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let backbuffer = builder.import_external_resource("backbuffer");
    let depth_prepass = builder.add_pass("depth-prepass", QueueLane::Graphics);
    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);
    builder.write_texture(depth_prepass, depth).unwrap();
    builder.read_texture(opaque, depth).unwrap();
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(present, color).unwrap();
    builder.write_external(present, backbuffer).unwrap();

    let graph = builder.compile().unwrap();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "opaque")
        .unwrap();
    let context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            pass.name.clone(),
            RenderPassExecutorId::new(pass.executor_id.clone().unwrap_or_default()),
            pass.queue,
            pass.declared_queue,
            pass.flags,
            pass.dependencies.clone(),
            {
                let mut resources = pass.resources.clone();
                resources.push(RenderGraphPassResourceAccess {
                    name: "backbuffer".to_string(),
                    kind: RenderGraphResourceKind::External,
                    access: RenderGraphResourceAccessKind::Read,
                    attachment_ops: None,
                });
                resources
            },
        )
        .with_resource_resolver(&graph, pass.id);

    let depth_resource = RenderGraphResource::TransientTexture(depth);
    let color_resource = RenderGraphResource::TransientTexture(color);
    let backbuffer_resource = RenderGraphResource::External(backbuffer);

    assert_eq!(
        context
            .resource_declaration(depth_resource)
            .unwrap()
            .name
            .as_str(),
        "scene-depth"
    );
    assert_eq!(
        context
            .resource_lifetime(color_resource)
            .unwrap()
            .name
            .as_str(),
        "scene-color"
    );
    assert!(context.declares_resource_access(depth_resource, RenderGraphResourceAccessKind::Read));
    assert_eq!(
        context
            .resource_resolver()
            .and_then(|resolver| resolver.pass_resource_declaration_by_name(
                "scene-depth",
                RenderGraphResourceAccessKind::Read
            ))
            .unwrap()
            .resource,
        depth_resource
    );
    assert!(
        context
            .resource_resolver()
            .and_then(|resolver| resolver.pass_resource_declaration_by_name(
                "scene-depth",
                RenderGraphResourceAccessKind::Write
            ))
            .is_none()
    );
    assert!(context.declares_resource_access(color_resource, RenderGraphResourceAccessKind::Write));
    assert!(
        !context
            .declares_resource_access(backbuffer_resource, RenderGraphResourceAccessKind::Write)
    );
    assert!(
        !context.reads_texture("backbuffer"),
        "resolver-backed name queries must follow the compiled pass contract instead of stale context resource rows"
    );
}

#[test]
fn resolver_backed_name_access_ignores_stale_context_resource_rows() {
    let mut builder = RenderGraphBuilder::new("resolver-name-access");
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_external_resource("viewport-output");
    let write = builder.add_pass("write", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);
    builder.write_texture(write, color).unwrap();
    builder.read_texture(present, color).unwrap();
    builder.write_external(present, output).unwrap();
    let graph = builder.compile().unwrap();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "write")
        .unwrap();
    let mut resources = pass.resources.clone();
    resources.push(RenderGraphPassResourceAccess {
        name: "viewport-output".to_string(),
        kind: RenderGraphResourceKind::External,
        access: RenderGraphResourceAccessKind::Read,
        attachment_ops: None,
    });
    let context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            pass.name.clone(),
            RenderPassExecutorId::new(pass.executor_id.clone().unwrap_or_default()),
            pass.queue,
            pass.declared_queue,
            pass.flags,
            pass.dependencies.clone(),
            resources,
        )
        .with_resource_resolver(&graph, pass.id);

    assert!(
        !context
            .declares_resource_name_access("viewport-output", RenderGraphResourceAccessKind::Read)
    );
    assert!(
        context.declares_resource_name_access("scene-color", RenderGraphResourceAccessKind::Write)
    );
}
