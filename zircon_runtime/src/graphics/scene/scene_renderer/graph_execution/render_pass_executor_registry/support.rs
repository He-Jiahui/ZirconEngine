use crate::core::framework::render::{RenderFrameExtract, RenderWorldSnapshotHandle};
use crate::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassStage,
};
use crate::render_graph::QueueLane;
use crate::scene::world::World;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
};

use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderPassExecutionContext, RenderPassExecutor,
};

pub(super) fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

pub(super) fn import_test_texture(
    resources: &mut RenderGraphExecutionResources,
    device: &wgpu::Device,
    name: &'static str,
) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(name),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    resources.import_texture_view(
        name,
        texture.create_view(&wgpu::TextureViewDescriptor::default()),
    );
}

pub(super) fn import_test_buffer(
    resources: &mut RenderGraphExecutionResources,
    device: &wgpu::Device,
    name: &'static str,
    size: u64,
) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(name),
        size: size.max(std::mem::size_of::<u32>() as u64),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::UNIFORM
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    resources.insert_buffer(name, buffer);
}

pub(super) fn test_ui_extract() -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("screen-space-ui-attachment-ops"),
        list: UiRenderList {
            commands: vec![UiRenderCommand {
                node_id: UiNodeId::new(1),
                kind: UiRenderCommandKind::Quad,
                frame: UiFrame::new(0.0, 0.0, 8.0, 8.0),
                clip_frame: None,
                z_index: 0,
                style: UiResolvedStyle {
                    background_color: Some("#ff0000".to_string()),
                    ..UiResolvedStyle::default()
                },
                text_layout: None,
                text: None,
                image: None,
                opacity: 1.0,
            }],
        },
    }
}

pub(super) fn plugin_virtual_geometry_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plugin.virtual_geometry.registry",
        Vec::new(),
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::DepthPrepass,
            "plugin-virtual-geometry-registry",
            QueueLane::Graphics,
        )
        .with_executor_id("virtual-geometry.prepare")
        .with_side_effects()],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
}

pub(super) fn explicit_virtual_geometry_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if context.executor_id.as_str() == "virtual-geometry.prepare" {
        return Err("explicit virtual geometry executor called".to_string());
    }
    Ok(())
}

pub(super) struct ContextMutatingExecutor;

impl RenderPassExecutor for ContextMutatingExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        context.pass_name.push_str(":executed");
        Ok(())
    }
}
