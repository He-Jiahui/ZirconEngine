use std::sync::Arc;

use crate::core::framework::scene::Mobility;
use crate::core::math::{RenderVec4, Vec4};
use wgpu::util::DeviceExt;

use crate::graphics::scene::resources::{
    GpuMaterialUniformResource, GpuMeshResource, GpuTextureResource, PipelineKey,
};

use super::super::super::primitives::{render_vec4_or, ModelUniform};
use super::super::mesh_draw::{MeshDraw, MeshDrawGeometrySource, VirtualGeometrySubmissionDetail};

pub(super) fn create_mesh_draw(
    device: &wgpu::Device,
    model_layout: &wgpu::BindGroupLayout,
    mesh: Arc<GpuMeshResource>,
    geometry_source: MeshDrawGeometrySource,
    mobility: Mobility,
    texture: Arc<GpuTextureResource>,
    material_uniform: Arc<GpuMaterialUniformResource>,
    pipeline_key: PipelineKey,
    cast_shadows: bool,
    receive_shadows: bool,
    model_matrix: [[f32; 4]; 4],
    previous_model_matrix: [[f32; 4]; 4],
    has_previous_motion_vector_transform: bool,
    draw_tint: Vec4,
    first_index: u32,
    draw_index_count: u32,
    indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_args_offset: u64,
    virtual_geometry_submission_detail: Option<VirtualGeometrySubmissionDetail>,
) -> MeshDraw {
    let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-model-buffer"),
        contents: bytemuck::bytes_of(&model_uniform_from_draw_state(
            &pipeline_key,
            receive_shadows,
            model_matrix,
            previous_model_matrix,
            has_previous_motion_vector_transform,
            draw_tint,
        )),
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

    MeshDraw::new(
        mesh,
        geometry_source,
        mobility,
        first_index,
        draw_index_count,
        indirect_args_buffer,
        indirect_args_offset,
        virtual_geometry_submission_detail,
        texture,
        material_uniform,
        pipeline_key,
        cast_shadows,
        model_buffer,
        model_bind_group,
        has_previous_motion_vector_transform,
    )
}

fn model_uniform_from_draw_state(
    pipeline_key: &PipelineKey,
    receive_shadows: bool,
    model_matrix: [[f32; 4]; 4],
    previous_model_matrix: [[f32; 4]; 4],
    has_previous_motion_vector_transform: bool,
    draw_tint: Vec4,
) -> ModelUniform {
    ModelUniform {
        model: model_matrix,
        tint: render_vec4_or(draw_tint, RenderVec4::ONE).to_array(),
        shadow_params: shadow_params_from_material_state(pipeline_key, receive_shadows),
        previous_model: previous_model_matrix,
        motion_params: [
            if has_previous_motion_vector_transform {
                1.0
            } else {
                0.0
            },
            0.0,
            0.0,
            0.0,
        ],
    }
}

fn shadow_params_from_material_state(
    pipeline_key: &PipelineKey,
    receive_shadows: bool,
) -> [f32; 4] {
    let alpha_cutoff = pipeline_key
        .alpha_cutoff_bits
        .map(f32::from_bits)
        .filter(|cutoff| cutoff.is_finite())
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);

    [
        if pipeline_key.is_alpha_mask() {
            1.0
        } else {
            0.0
        },
        alpha_cutoff,
        if receive_shadows { 1.0 } else { 0.0 },
        0.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::{model_uniform_from_draw_state, shadow_params_from_material_state};
    use crate::core::math::Vec4;
    use crate::graphics::scene::resources::default_pipeline_key;

    #[test]
    fn shadow_params_encode_alpha_mask_cutoff_for_shadow_casters() {
        let mut key = default_pipeline_key();
        key.alpha_mask = true;
        key.alpha_cutoff_bits = Some(0.42f32.to_bits());

        assert_eq!(
            shadow_params_from_material_state(&key, true),
            [1.0, 0.42, 1.0, 0.0]
        );
    }

    #[test]
    fn shadow_params_clamp_invalid_alpha_cutoff_to_default_policy() {
        let mut key = default_pipeline_key();
        key.alpha_mask = true;
        key.alpha_cutoff_bits = Some(f32::NAN.to_bits());

        assert_eq!(
            shadow_params_from_material_state(&key, true),
            [1.0, 0.5, 1.0, 0.0]
        );
    }

    #[test]
    fn shadow_params_encode_material_shadow_receiver_toggle() {
        let key = default_pipeline_key();

        assert_eq!(
            shadow_params_from_material_state(&key, true),
            [0.0, 0.5, 1.0, 0.0]
        );
        assert_eq!(
            shadow_params_from_material_state(&key, false),
            [0.0, 0.5, 0.0, 0.0]
        );
    }

    #[test]
    fn model_uniform_appends_previous_motion_transform_without_moving_existing_fields() {
        let key = default_pipeline_key();
        let current = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [4.0, 5.0, 6.0, 1.0],
        ];
        let previous = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [1.0, 2.0, 3.0, 1.0],
        ];

        let uniform = model_uniform_from_draw_state(
            &key,
            true,
            current,
            previous,
            true,
            Vec4::new(0.25, 0.5, 0.75, 1.0),
        );

        assert_eq!(uniform.model, current);
        assert_eq!(uniform.tint, [0.25, 0.5, 0.75, 1.0]);
        assert_eq!(uniform.shadow_params, [0.0, 0.5, 1.0, 0.0]);
        assert_eq!(uniform.previous_model, previous);
        assert_eq!(uniform.motion_params, [1.0, 0.0, 0.0, 0.0]);
    }
}
