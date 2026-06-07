use bytemuck::bytes_of;

use crate::core::framework::render::RenderDirectionalLightSnapshot;
use crate::core::math::{is_finite_mat4, is_finite_vec3, view_matrix, Mat4, Real, Transform, Vec3};
use crate::graphics::scene::resources::GpuMeshVertex;
use crate::graphics::scene::scene_renderer::attachment_ops::depth_attachment_operations;
use crate::graphics::scene::scene_renderer::mesh::MeshDraw;
use crate::graphics::scene::scene_renderer::primitives::SceneUniform;
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::RenderGraphAttachmentOps;

use super::shadow_map_shader_source::SHADOW_MAP_SHADER;

const DEFAULT_SHADOW_LIGHT_DIRECTION_COMPONENTS: [Real; 3] = [-0.4, -1.0, -0.25];
const DEFAULT_SHADOW_LIGHT_COLOR_INTENSITY: Real = 1.8;
pub(crate) const SHADOW_RECEIVER_DEPTH_BIAS: f32 = 0.003;
pub(crate) const SHADOW_RECEIVER_MIN_VISIBILITY: f32 = 0.38;
pub(crate) const DEFERRED_SHADOW_RECEIVER_DEPTH_BIAS: f32 = SHADOW_RECEIVER_DEPTH_BIAS;
pub(crate) const DEFERRED_SHADOW_RECEIVER_MIN_VISIBILITY: f32 = SHADOW_RECEIVER_MIN_VISIBILITY;
const MIN_SHADOW_ORTHOGRAPHIC_HALF_EXTENT: Real = 4.0;
const SHADOW_CAMERA_DISTANCE_SCALE: Real = 2.0;
const SHADOW_CAMERA_FAR_PADDING: Real = 64.0;
const SHADOW_CAMERA_NEAR_PLANE: Real = 0.1;
const SHADOW_CAMERA_MIN_FAR_PLANE: Real = 1.0;
const SHADOW_UP_ALIGNMENT_LIMIT: Real = 0.95;
const SHADOW_DEPTH_BIAS_CONSTANT: i32 = 2;
const SHADOW_DEPTH_BIAS_SLOPE_SCALE: f32 = 2.0;
const SHADOW_DEPTH_BIAS_CLAMP: f32 = 0.0;

pub(crate) struct ShadowMapRenderer {
    pipeline: wgpu::RenderPipeline,
    alpha_mask_pipeline: wgpu::RenderPipeline,
    scene_uniform_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,
}

impl ShadowMapRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        model_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-shadow-map-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADOW_MAP_SHADER.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-shadow-map-layout"),
            bind_group_layouts: &[Some(scene_layout), Some(model_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-shadow-map-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[GpuMeshVertex::layout()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::super::core::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: SHADOW_DEPTH_BIAS_CONSTANT,
                    slope_scale: SHADOW_DEPTH_BIAS_SLOPE_SCALE,
                    clamp: SHADOW_DEPTH_BIAS_CLAMP,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: None,
            multiview_mask: None,
            cache: None,
        });
        let alpha_mask_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-shadow-map-alpha-mask-layout"),
            bind_group_layouts: &[Some(scene_layout), Some(model_layout), Some(texture_layout)],
            immediate_size: 0,
        });
        let alpha_mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-shadow-map-alpha-mask-pipeline"),
            layout: Some(&alpha_mask_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[GpuMeshVertex::layout()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::super::core::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: SHADOW_DEPTH_BIAS_CONSTANT,
                    slope_scale: SHADOW_DEPTH_BIAS_SLOPE_SCALE,
                    clamp: SHADOW_DEPTH_BIAS_CLAMP,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_alpha_mask"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[],
            }),
            multiview_mask: None,
            cache: None,
        });
        let scene_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-shadow-map-scene-uniform"),
            size: std::mem::size_of::<SceneUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-shadow-map-scene-bind-group"),
            layout: scene_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            alpha_mask_pipeline,
            scene_uniform_buffer,
            scene_bind_group,
        }
    }

    pub(crate) fn record_with_attachment_ops<'a, I>(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        pass_name: &str,
        shadow_map_view: &wgpu::TextureView,
        frame: &ViewportRenderFrame,
        mesh_draws: I,
        attachment_ops: RenderGraphAttachmentOps,
    ) where
        I: IntoIterator<Item = &'a MeshDraw>,
    {
        let scene_uniform = self.scene_uniform_for_frame(frame);
        if let Some(scene_uniform) = scene_uniform {
            queue.write_buffer(&self.scene_uniform_buffer, 0, bytes_of(&scene_uniform));
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(pass_name),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: shadow_map_view,
                depth_ops: Some(depth_attachment_operations(attachment_ops, 1.0)),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if scene_uniform.is_none() {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.scene_bind_group, &[]);
        for draw in mesh_draws {
            if draw.is_alpha_mask() {
                pass.set_pipeline(&self.alpha_mask_pipeline);
                draw.bind_texture(&mut pass);
            } else {
                pass.set_pipeline(&self.pipeline);
            }
            draw.bind_model(&mut pass);
            draw.bind_geometry_buffers(&mut pass);
            draw.record_indexed_draw(&mut pass);
        }
    }

    pub(crate) fn scene_uniform_for_frame(
        &self,
        frame: &ViewportRenderFrame,
    ) -> Option<SceneUniform> {
        shadow_light(frame).map(|light| shadow_scene_uniform(frame, light))
    }
}

#[derive(Clone, Copy)]
struct ShadowLight {
    direction: Vec3,
    color: Vec3,
}

fn shadow_light(frame: &ViewportRenderFrame) -> Option<ShadowLight> {
    if !frame.preview().lighting_enabled {
        return None;
    }
    frame
        .directional_lights()
        .first()
        .map(shadow_light_from_directional)
        .or_else(|| {
            Some(ShadowLight {
                direction: default_shadow_light_direction(),
                color: Vec3::splat(DEFAULT_SHADOW_LIGHT_COLOR_INTENSITY),
            })
        })
}

fn shadow_light_from_directional(light: &RenderDirectionalLightSnapshot) -> ShadowLight {
    ShadowLight {
        direction: sanitize_direction(light.direction),
        color: sanitize_color(light.color * light.intensity),
    }
}

fn shadow_scene_uniform(frame: &ViewportRenderFrame, light: ShadowLight) -> SceneUniform {
    let view_proj = shadow_view_projection(frame, light.direction);
    let view_proj = finite_mat4_or_identity(view_proj);
    let view_proj_cols = view_proj.to_cols_array_2d();
    SceneUniform {
        view_proj: view_proj_cols,
        inverse_view_proj: finite_mat4_or_identity(view_proj.inverse()).to_cols_array_2d(),
        light_dir: light.direction.extend(0.0).to_array(),
        light_color: light.color.extend(1.0).to_array(),
        ambient_color: Vec3::ZERO.extend(1.0).to_array(),
        previous_view_proj: view_proj_cols,
        motion_params: [0.0, 0.0, 0.0, 0.0],
    }
}

fn shadow_view_projection(frame: &ViewportRenderFrame, direction: Vec3) -> Mat4 {
    let (center, radius) = shadow_bounds_from_frame(frame).unwrap_or_else(|| {
        let center = finite_vec3_or(frame.camera().transform.translation, Vec3::ZERO);
        (center, MIN_SHADOW_ORTHOGRAPHIC_HALF_EXTENT)
    });
    let half_extent = radius.max(MIN_SHADOW_ORTHOGRAPHIC_HALF_EXTENT);
    let distance = half_extent * SHADOW_CAMERA_DISTANCE_SCALE + SHADOW_CAMERA_FAR_PADDING;
    let far_plane = (distance + half_extent + SHADOW_CAMERA_FAR_PADDING)
        .max(SHADOW_CAMERA_NEAR_PLANE + SHADOW_CAMERA_MIN_FAR_PLANE);
    let eye = center - direction * distance;
    let transform = Transform::looking_at(eye, center, stable_shadow_up(direction));
    let view = view_matrix(transform);
    let projection = Mat4::orthographic_rh(
        -half_extent,
        half_extent,
        -half_extent,
        half_extent,
        SHADOW_CAMERA_NEAR_PLANE,
        far_plane,
    );
    projection * view
}

fn shadow_bounds_from_frame(frame: &ViewportRenderFrame) -> Option<(Vec3, Real)> {
    let mut center = Vec3::ZERO;
    let mut count = 0usize;
    for mesh in frame.meshes() {
        let translation = mesh.transform.translation;
        if is_finite_vec3(translation) {
            center += translation;
            count += 1;
        }
    }
    if count == 0 {
        return None;
    }
    center /= count as Real;

    let mut radius: Real = 0.0;
    for mesh in frame.meshes() {
        let translation = mesh.transform.translation;
        if is_finite_vec3(translation) {
            radius = radius.max((translation - center).length());
        }
    }
    Some((center, radius.max(MIN_SHADOW_ORTHOGRAPHIC_HALF_EXTENT)))
}

fn stable_shadow_up(direction: Vec3) -> Vec3 {
    if direction.dot(Vec3::Y).abs() > SHADOW_UP_ALIGNMENT_LIMIT {
        Vec3::X
    } else {
        Vec3::Y
    }
}

fn sanitize_direction(direction: Vec3) -> Vec3 {
    if is_finite_vec3(direction) && direction.length_squared() > f32::EPSILON {
        direction.normalize_or_zero()
    } else {
        default_shadow_light_direction()
    }
}

fn sanitize_color(color: Vec3) -> Vec3 {
    finite_vec3_or(color, Vec3::splat(DEFAULT_SHADOW_LIGHT_COLOR_INTENSITY))
}

fn finite_vec3_or(value: Vec3, fallback: Vec3) -> Vec3 {
    if is_finite_vec3(value) {
        value
    } else {
        fallback
    }
}

fn finite_mat4_or_identity(value: Mat4) -> Mat4 {
    if is_finite_mat4(value) {
        value
    } else {
        Mat4::IDENTITY
    }
}

fn default_shadow_light_direction() -> Vec3 {
    Vec3::from_array(DEFAULT_SHADOW_LIGHT_DIRECTION_COMPONENTS).normalize_or_zero()
}

#[cfg(test)]
mod tests {
    use super::{sanitize_direction, stable_shadow_up};
    use crate::core::math::Vec3;

    #[test]
    fn shadow_direction_falls_back_when_invalid() {
        let direction = sanitize_direction(Vec3::new(f32::NAN, 0.0, 0.0));

        assert!(direction.is_finite());
        assert!(direction.length_squared() > 0.9);
    }

    #[test]
    fn shadow_up_avoids_parallel_light_axis() {
        assert_eq!(stable_shadow_up(Vec3::Y), Vec3::X);
        assert_eq!(stable_shadow_up(Vec3::Z), Vec3::Y);
    }

    #[test]
    fn shadow_map_shader_keeps_opaque_depth_path_and_alpha_mask_cutoff_path() {
        assert!(super::SHADOW_MAP_SHADER.contains("@vertex"));
        assert!(super::SHADOW_MAP_SHADER.contains("fn fs_alpha_mask"));
        assert!(super::SHADOW_MAP_SHADER.contains("discard"));
        assert!(super::SHADOW_MAP_SHADER.contains("shadow_params.y"));
    }
}
