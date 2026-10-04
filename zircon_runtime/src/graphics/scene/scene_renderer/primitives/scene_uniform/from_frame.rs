use crate::core::framework::render::{ProjectionMode, SkyboxMode, ViewProjectionMatrixPair};
use crate::core::math::{Mat4, RenderMat4, RenderVec3, UVec2};

use crate::graphics::scene::scene_renderer::temporal::velocity::velocity_camera_params::VelocityCameraParams;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::ViewportRenderRegion;

use super::super::fallback::{render_mat4_or, render_vec3_or, render_vec4_or};
use super::SceneUniform;

impl SceneUniform {
    pub(crate) fn from_frame(frame: &ViewportRenderFrame) -> Self {
        let camera = frame.effective_camera();
        let (ambient_color, lightmapped_ambient_color) = if frame.preview().lighting_enabled {
            authored_ambient_colors(frame, RenderVec3::splat(0.2))
        } else {
            let fallback = RenderVec3::splat(0.55).extend(1.0).to_array();
            (fallback, fallback)
        };

        let matrix_pair =
            ViewProjectionMatrixPair::from_camera(&camera, frame.render_region().local_size());
        let view_proj = matrix_pair.clip_from_world_jittered;
        let view_proj_unjittered = matrix_pair.clip_from_world_unjittered;
        let (previous_view_proj_unjittered, motion_params) =
            previous_motion_view_projection(frame, &camera, view_proj_unjittered);
        let skybox = &frame.environment().skybox;
        let sky_params = skybox.procedural;
        let authored_environment_rotation = skybox.rotation_radians();
        // 太阳方向与环境采样尾字段共用此有限旋转角；非有限作者值回退到零，
        // 避免两个使用方依据不同的角度生成本帧 uniform。
        let environment_rotation = if authored_environment_rotation.is_finite() {
            authored_environment_rotation
        } else {
            0.0
        };
        let environment_rotation_sin_cos = if environment_rotation == 0.0 {
            [0.0, 1.0, 0.0, 0.0]
        } else {
            let (environment_rotation_sin, environment_rotation_cos) =
                environment_rotation.sin_cos();
            [environment_rotation_sin, environment_rotation_cos, 1.0, 0.0]
        };
        let resolved_sun = sky_params.resolved_sun();
        let scene_sun_direction =
            resolved_sun.direction_for_sampling_rotation(environment_rotation);
        let source_cubemap_environment = frame.source_cubemap_environment();
        let has_ibl_source = match skybox.mode {
            SkyboxMode::Disabled => false,
            SkyboxMode::ProceduralGradient => true,
            SkyboxMode::SourceCubemap => source_cubemap_environment.is_some(),
        };
        let has_source_cubemap_irradiance = source_cubemap_environment
            .is_some_and(|environment| environment.irradiance_cube().is_some());
        let environment_sample_params = match skybox.mode {
            SkyboxMode::Disabled | SkyboxMode::ProceduralGradient => {
                [skybox.mode as u32 as f32, 0.0, 0.0, 0.0]
            }
            SkyboxMode::SourceCubemap => source_cubemap_environment
                .map(|environment| {
                    [
                        skybox.mode as u32 as f32,
                        environment.mip_chain.source_face_size() as f32,
                        environment.mip_chain.pmrem_face_size() as f32,
                        environment.mip_chain.pmrem_mip_count() as f32,
                    ]
                })
                .unwrap_or([skybox.mode as u32 as f32, 0.0, 0.0, 0.0]),
        };
        Self {
            view_proj: render_mat4_or(view_proj, RenderMat4::IDENTITY).to_cols_array_2d(),
            view_proj_unjittered: render_mat4_or(view_proj_unjittered, RenderMat4::IDENTITY)
                .to_cols_array_2d(),
            inverse_view_proj: render_mat4_or(view_proj_unjittered.inverse(), RenderMat4::IDENTITY)
                .to_cols_array_2d(),
            ambient_color,
            lightmapped_ambient_color,
            previous_view_proj_unjittered,
            motion_params,
            jitter_params: jitter_params(&camera),
            camera_world_position: render_vec3_or(camera.transform.translation, RenderVec3::ZERO)
                .extend(0.0)
                .to_array(),
            camera_view_direction: camera_view_direction(&camera),
            sky_horizon_color: render_vec4_or(
                sky_params.horizon_color,
                crate::core::math::Vec4::ZERO,
            )
            .to_array(),
            sky_zenith_color: render_vec4_or(
                sky_params.zenith_color,
                crate::core::math::Vec4::ZERO,
            )
            .to_array(),
            sky_ground_color: render_vec4_or(
                sky_params.ground_color,
                crate::core::math::Vec4::ZERO,
            )
            .to_array(),
            sky_sun_direction: scene_sun_direction.to_array(),
            sky_sun_color_radius: [
                sky_params.sun_color.x,
                sky_params.sun_color.y,
                sky_params.sun_color.z,
                sky_params.sun_angular_radius_radians,
            ],
            sky_sun_params: resolved_sun.intensity_and_cosines.to_array(),
            environment_params: [
                if has_source_cubemap_irradiance {
                    1.0
                } else {
                    0.0
                },
                skybox.intensity().max(0.0),
                environment_rotation,
                if has_ibl_source { 1.0 } else { 0.0 },
            ],
            environment_sample_params,
            environment_rotation_sin_cos,
        }
    }

    pub(crate) fn from_hit_proxy_frame(frame: &ViewportRenderFrame, pixel: UVec2) -> Option<Self> {
        let crop = hit_proxy_clip_crop(pixel, frame.render_region())?;
        let mut uniform = Self::from_frame(frame);
        let jittered = crop * Mat4::from_cols_array_2d(&uniform.view_proj);
        let unjittered = crop * Mat4::from_cols_array_2d(&uniform.view_proj_unjittered);
        uniform.view_proj = render_mat4_or(jittered, RenderMat4::IDENTITY).to_cols_array_2d();
        uniform.view_proj_unjittered =
            render_mat4_or(unjittered, RenderMat4::IDENTITY).to_cols_array_2d();
        uniform.inverse_view_proj =
            render_mat4_or(unjittered.inverse(), RenderMat4::IDENTITY).to_cols_array_2d();
        Some(uniform)
    }
}

fn hit_proxy_clip_crop(pixel: UVec2, region: ViewportRenderRegion) -> Option<Mat4> {
    let origin = region.physical_position();
    let size = region.physical_size();
    if pixel.x < origin.x || pixel.y < origin.y {
        return None;
    }
    let local = pixel - origin;
    if local.x >= size.x || local.y >= size.y {
        return None;
    }
    let width = size.x as f32;
    let height = size.y as f32;
    let center_x = 2.0 * (local.x as f32 + 0.5) / width - 1.0;
    let center_y = 1.0 - 2.0 * (local.y as f32 + 0.5) / height;
    Some(Mat4::from_cols_array_2d(&[
        [width, 0.0, 0.0, 0.0],
        [0.0, height, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-width * center_x, -height * center_y, 0.0, 1.0],
    ]))
}

fn previous_motion_view_projection(
    frame: &ViewportRenderFrame,
    camera: &crate::core::framework::render::ViewportCameraSnapshot,
    fallback_view_proj: Mat4,
) -> ([[f32; 4]; 4], [f32; 4]) {
    let Some(previous_camera) = frame.previous_motion_vector_camera() else {
        return (
            render_mat4_or(fallback_view_proj, RenderMat4::IDENTITY).to_cols_array_2d(),
            [0.0, 0.0, 0.0, 0.0],
        );
    };
    let params =
        VelocityCameraParams::from_cameras(frame.viewport_size, camera, previous_camera, true);
    if !params.is_enabled() {
        return (
            render_mat4_or(fallback_view_proj, RenderMat4::IDENTITY).to_cols_array_2d(),
            [0.0, 0.0, 0.0, 0.0],
        );
    }

    (params.previous_clip_from_world(), [1.0, 0.0, 0.0, 0.0])
}

fn jitter_params(camera: &crate::core::framework::render::ViewportCameraSnapshot) -> [f32; 4] {
    [
        camera.temporal_jitter.offset_pixels.x,
        camera.temporal_jitter.offset_pixels.y,
        camera.temporal_jitter.sequence_index as f32,
        if camera.temporal_jitter.sequence_index > 0 {
            1.0
        } else {
            0.0
        },
    ]
}

fn camera_view_direction(
    camera: &crate::core::framework::render::ViewportCameraSnapshot,
) -> [f32; 4] {
    let view_direction = render_vec3_or(
        camera.transform.rotation * crate::core::math::Vec3::Z,
        RenderVec3::Z,
    )
    .normalize_or_zero();
    [
        view_direction.x,
        view_direction.y,
        view_direction.z,
        match camera.projection_mode {
            ProjectionMode::Orthographic => 1.0,
            ProjectionMode::Perspective => 0.0,
        },
    ]
}

fn authored_ambient_colors(
    frame: &crate::graphics::types::ViewportRenderFrame,
    fallback: RenderVec3,
) -> ([f32; 4], [f32; 4]) {
    if frame.ambient_lights().is_empty() {
        let fallback = fallback.extend(1.0).to_array();
        return (fallback, fallback);
    }

    let (ambient, lightmapped_ambient) = frame.ambient_lights().iter().fold(
        (RenderVec3::ZERO, RenderVec3::ZERO),
        |accumulated, light| {
            let radiance = render_vec3_or(light.color * light.intensity, RenderVec3::ZERO);
            (
                accumulated.0 + radiance,
                accumulated.1
                    + if light.affects_lightmapped_meshes {
                        radiance
                    } else {
                        RenderVec3::ZERO
                    },
            )
        },
    );
    (
        ambient.extend(1.0).to_array(),
        lightmapped_ambient.extend(1.0).to_array(),
    )
}

#[cfg(test)]
#[path = "tests/from_frame.rs"]
mod tests;
