use crate::core::framework::render::ProjectionMode;
use crate::core::math::{view_matrix, Mat4, Real, RenderMat4, RenderVec3};

use crate::graphics::types::ViewportRenderFrame;

use super::super::fallback::{render_mat4_or, render_vec3_or};
use super::SceneUniform;

impl SceneUniform {
    pub(crate) fn from_frame(frame: &ViewportRenderFrame, aspect: Real) -> Self {
        let camera = frame.camera();
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
        let (light_dir, light_color, ambient_color) = if frame.preview().lighting_enabled {
            if let Some(light) = frame.directional_lights().first() {
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
                    authored_ambient_color(frame, RenderVec3::splat(0.22)),
                )
            } else {
                (
                    RenderVec3::new(-0.4, -1.0, -0.25)
                        .normalize_or_zero()
                        .extend(0.0)
                        .to_array(),
                    RenderVec3::splat(1.8).extend(1.0).to_array(),
                    authored_ambient_color(frame, RenderVec3::splat(0.2)),
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

fn authored_ambient_color(
    frame: &crate::graphics::types::ViewportRenderFrame,
    fallback: RenderVec3,
) -> [f32; 4] {
    if frame.ambient_lights().is_empty() {
        return fallback.extend(1.0).to_array();
    }

    frame
        .ambient_lights()
        .iter()
        .fold(RenderVec3::ZERO, |accumulated, light| {
            accumulated + render_vec3_or(light.color * light.intensity, RenderVec3::ZERO)
        })
        .extend(1.0)
        .to_array()
}

#[cfg(test)]
mod tests {
    use super::SceneUniform;
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderAmbientLightSnapshot,
        RenderFrameExtract, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec3, Vec4};
    use crate::graphics::types::ViewportRenderFrame;

    #[test]
    fn scene_uniform_uses_authored_ambient_light_when_lighting_is_enabled() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            empty_scene_snapshot(),
        );
        extract.post_process.preview.lighting_enabled = true;
        extract
            .lighting
            .ambient_lights
            .push(RenderAmbientLightSnapshot {
                color: Vec3::new(0.05, 0.06, 0.07),
                intensity: 0.35,
                renderer_degraded: false,
                degradation_reason: None,
            });
        extract
            .lighting
            .ambient_lights
            .push(RenderAmbientLightSnapshot {
                color: Vec3::new(0.01, 0.02, 0.03),
                intensity: 0.5,
                renderer_degraded: false,
                degradation_reason: None,
            });
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

        let uniform = SceneUniform::from_frame(&frame, 1.0);

        assert_close(uniform.ambient_color[0], 0.0225);
        assert_close(uniform.ambient_color[1], 0.031);
        assert_close(uniform.ambient_color[2], 0.0395);
        assert_eq!(uniform.ambient_color[3], 1.0);
    }

    fn empty_scene_snapshot() -> RenderSceneSnapshot {
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    projection_mode: ProjectionMode::Perspective,
                    ..ViewportCameraSnapshot::default()
                },
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        }
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }
}
