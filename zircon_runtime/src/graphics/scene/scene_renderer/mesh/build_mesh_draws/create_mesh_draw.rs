use std::sync::Arc;

use crate::core::math::{RenderVec4, Vec4};
use wgpu::util::DeviceExt;

use crate::graphics::scene::resources::{GpuMeshResource, GpuTextureResource, PipelineKey};

use super::super::super::primitives::{render_vec4_or, ModelUniform};
use super::super::mesh_draw::{MeshDraw, VirtualGeometrySubmissionDetail};

pub(super) fn create_mesh_draw(
    device: &wgpu::Device,
    model_layout: &wgpu::BindGroupLayout,
    mesh: Arc<GpuMeshResource>,
    texture: Arc<GpuTextureResource>,
    pipeline_key: PipelineKey,
    model_matrix: [[f32; 4]; 4],
    draw_tint: Vec4,
    first_index: u32,
    draw_index_count: u32,
    indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_args_offset: u64,
    virtual_geometry_submission_detail: Option<VirtualGeometrySubmissionDetail>,
) -> MeshDraw {
    let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-model-buffer"),
        contents: bytemuck::bytes_of(&ModelUniform {
            model: model_matrix,
            tint: render_vec4_or(draw_tint, RenderVec4::ONE).to_array(),
        }),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-model-bind-group"),
        layout: model_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: model_buffer.as_entire_binding(),
        }],
    });

    MeshDraw {
        mesh,
        first_index,
        draw_index_count,
        indirect_args_buffer,
        indirect_args_offset,
        virtual_geometry_submission_key: virtual_geometry_submission_detail
            .map(|detail| (detail.entity, detail.page_id)),
        virtual_geometry_submission_detail,
        texture,
        pipeline_key,
        model_buffer,
        model_bind_group,
    }
}
