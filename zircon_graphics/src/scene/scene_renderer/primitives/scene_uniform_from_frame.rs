use zircon_math::{view_matrix, Mat4, Real, RenderMat4, RenderVec3};
use zircon_scene::ProjectionMode;

use crate::types::EditorOrRuntimeFrame;

use super::fallback::{render_mat4_or, render_vec3_or};
use super::scene_uniform::SceneUniform;

impl SceneUniform {
    pub(crate) fn from_frame(frame: &EditorOrRuntimeFrame, aspect: Real) -> Self {
        let camera = &frame.scene.scene.camera;
        let projection = match camera.projection_mode {
            ProjectionMode::Perspective => Mat4::perspective_rh(
                camera.fov_y_radians,
                aspect.max(0.001),
                camera.z_near.max(0.001),
                camera.z_far,
            ),
            ProjectionMode::Orthographic => {
                let half_height = camera.ortho_size.max(0.01);
                let half_width = half_height * aspect.max(0.001);
                Mat4::orthographic_rh(
                    -half_width,
                    half_width,
                    -half_height,
                    half_height,
                    camera.z_near.max(0.001),
                    camera.z_far,
                )
            }
        };
        let view = view_matrix(camera.transform);
        let (light_dir, light_color, ambient_color) = if frame.scene.preview.lighting_enabled {
            if let Some(light) = frame.scene.scene.lights.first() {
                (
                    render_vec3_or(
                        light.direction,
                        RenderVec3::new(-0.4, -1.0, -0.25).normalize_or_zero(),
                    )
                    .extend(0.0)
                    .to_array(),
                    render_vec3_or(light.color * light.intensity, RenderVec3::splat(1.8))
                        .extend(1.0)
                        .to_array(),
                    RenderVec3::splat(0.22).extend(1.0).to_array(),
                )
            } else {
                (
                    RenderVec3::new(-0.4, -1.0, -0.25)
                        .normalize_or_zero()
                        .extend(0.0)
                        .to_array(),
                    RenderVec3::splat(1.8).extend(1.0).to_array(),
                    RenderVec3::splat(0.2).extend(1.0).to_array(),
                )
            }
        } else {
            (
                RenderVec3::new(0.0, -1.0, 0.0).extend(0.0).to_array(),
                RenderVec3::ZERO.extend(1.0).to_array(),
                RenderVec3::splat(0.55).extend(1.0).to_array(),
            )
        };

        Self {
            view_proj: render_mat4_or(projection * view, RenderMat4::IDENTITY).to_cols_array_2d(),
            light_dir,
            light_color,
            ambient_color,
        }
    }
}
