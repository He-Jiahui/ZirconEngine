use std::collections::HashSet;

use wgpu::util::DeviceExt;
use zircon_math::{RenderMat4, RenderVec4, Vec4};
use zircon_scene::DisplayMode;

use crate::types::EditorOrRuntimeFrame;

use super::super::super::resources::{default_pipeline_key, ResourceStreamer};
use super::super::primitives::{render_mat4_or, render_vec4_or, ModelUniform};
use super::mesh_draw::MeshDraw;

pub(crate) fn build_mesh_draws(
    device: &wgpu::Device,
    model_layout: &wgpu::BindGroupLayout,
    streamer: &ResourceStreamer,
    frame: &EditorOrRuntimeFrame,
    virtual_geometry_enabled: bool,
) -> Vec<MeshDraw> {
    let selection: HashSet<_> = frame
        .scene
        .overlays
        .selection
        .iter()
        .map(|highlight| highlight.owner)
        .collect();
    let allowed_virtual_geometry_entities = virtual_geometry_enabled.then(|| {
        frame
            .virtual_geometry_prepare
            .as_ref()
            .map(|prepare| {
                prepare
                    .visible_entities
                    .iter()
                    .copied()
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default()
    });

    let mut draws = Vec::new();
    for mesh_instance in &frame.scene.scene.meshes {
        if let Some(allowed_entities) = allowed_virtual_geometry_entities.as_ref() {
            if !allowed_entities.contains(&mesh_instance.node_id) {
                continue;
            }
        }
        let Some(model) = streamer.model(&mesh_instance.model.id()) else {
            continue;
        };
        let material = streamer.material(&mesh_instance.material.id());
        let texture = streamer.texture(material.and_then(|material| material.base_color_texture));
        let material_tint = material
            .map(|material| material.base_color)
            .unwrap_or(Vec4::ONE);
        let pipeline_key = material
            .map(|material| material.pipeline_key.clone())
            .unwrap_or_else(default_pipeline_key);
        let tint = if selection.contains(&mesh_instance.node_id)
            && frame.scene.overlays.display_mode != DisplayMode::WireOnly
        {
            mesh_instance.tint * material_tint * Vec4::new(1.0, 0.94, 0.72, 1.0)
        } else {
            mesh_instance.tint * material_tint
        };
        for mesh in &model.meshes {
            let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-model-buffer"),
                contents: bytemuck::bytes_of(&ModelUniform {
                    model: render_mat4_or(mesh_instance.transform.matrix(), RenderMat4::IDENTITY)
                        .to_cols_array_2d(),
                    tint: render_vec4_or(tint, RenderVec4::ONE).to_array(),
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
            draws.push(MeshDraw {
                mesh: mesh.clone(),
                texture: texture.clone(),
                pipeline_key: pipeline_key.clone(),
                model_buffer,
                model_bind_group,
            });
        }
    }
    draws
}
