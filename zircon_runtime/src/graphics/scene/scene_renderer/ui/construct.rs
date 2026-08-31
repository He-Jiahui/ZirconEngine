#[cfg(test)]
use crate::asset::ProjectAssetManager;
use crate::asset::ProjectAssetManagerAccess;
use std::sync::Arc;

use super::image::ScreenSpaceUiImageSystem;
use super::render::ScreenSpaceUiVertex;
use super::screen_space_ui_renderer::ScreenSpaceUiRenderer;
use super::text::ScreenSpaceUiTextSystem;
use crate::graphics::GraphicsError;
use crate::text::font::FontCollectionService;
#[cfg(test)]
use crate::text::font::shared_font_collection_service;

const SCREEN_SPACE_UI_SHADER: &str = include_str!("shaders/screen_space_ui.wgsl");

impl ScreenSpaceUiRenderer {
    #[cfg(test)]
    pub(crate) fn new_for_test(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        Self::new(
            ProjectAssetManagerAccess::for_test(asset_manager),
            device,
            queue,
            target_format,
        )
        .expect("test screen-space UI renderer should initialize")
    }

    #[cfg(test)]
    pub(crate) fn new(
        asset_manager: ProjectAssetManagerAccess,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_font_collection(
            asset_manager,
            device,
            target_format,
            shared_font_collection_service(),
        )
    }

    pub(crate) fn new_with_font_collection(
        asset_manager: ProjectAssetManagerAccess,
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        font_collection: Arc<FontCollectionService>,
    ) -> Result<Self, GraphicsError> {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-screen-space-ui-layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-screen-space-ui-shader"),
            source: wgpu::ShaderSource::Wgsl(SCREEN_SPACE_UI_SHADER.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-screen-space-ui-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[ScreenSpaceUiVertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let text_system = ScreenSpaceUiTextSystem::new_with_font_collection(
            asset_manager,
            device,
            target_format,
            font_collection,
        )
        .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let image_system = ScreenSpaceUiImageSystem::new(device, target_format);

        Ok(Self {
            pipeline,
            vertex_segments: Vec::new(),
            vertex_buffer_plan: None,
            image_system,
            plan_cache: Default::default(),
            text_system,
            text_prepare_report_valid: false,
            last_attachment_ops: crate::render_graph::RenderGraphAttachmentOps::load_store(),
            upload_transaction: Default::default(),
        })
    }
}
