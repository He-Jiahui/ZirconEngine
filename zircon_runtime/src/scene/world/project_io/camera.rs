use crate::asset::assets::{
    SceneCameraAsset, SceneCameraTargetAsset, ScenePostProcessSettingsAsset, SceneViewportRectAsset,
};
use crate::asset::project::ProjectManager;
use crate::core::framework::render::{RenderCameraTarget, RenderViewportRect};
use crate::core::resource::TextureMarker;
use crate::scene::components::CameraComponent;

use super::SceneProjectError;
use super::references::{handle_for_reference, reference_for_handle};
pub(super) fn camera_target_from_asset(
    project: &ProjectManager,
    target: SceneCameraTargetAsset,
) -> Result<RenderCameraTarget, SceneProjectError> {
    match target {
        SceneCameraTargetAsset::PrimarySurface => Ok(RenderCameraTarget::PrimarySurface),
        SceneCameraTargetAsset::Texture { texture } => {
            Ok(RenderCameraTarget::Texture(handle_for_reference::<
                TextureMarker,
            >(
                project, &texture
            )?))
        }
        SceneCameraTargetAsset::Headless { size } => Ok(RenderCameraTarget::Headless {
            size: crate::core::math::UVec2::new(size[0], size[1]),
        }),
    }
}

fn camera_target_to_asset(
    project: &ProjectManager,
    target: RenderCameraTarget,
) -> Result<SceneCameraTargetAsset, SceneProjectError> {
    match target {
        RenderCameraTarget::PrimarySurface => Ok(SceneCameraTargetAsset::PrimarySurface),
        RenderCameraTarget::Texture(texture) => Ok(SceneCameraTargetAsset::Texture {
            texture: reference_for_handle(project, texture.id(), "camera texture target")?,
        }),
        RenderCameraTarget::Headless { size } => Ok(SceneCameraTargetAsset::Headless {
            size: [size.x, size.y],
        }),
    }
}

pub(super) fn viewport_rect_from_asset(viewport: SceneViewportRectAsset) -> RenderViewportRect {
    RenderViewportRect {
        physical_position: crate::core::math::UVec2::new(
            viewport.physical_position[0],
            viewport.physical_position[1],
        ),
        physical_size: crate::core::math::UVec2::new(
            viewport.physical_size[0],
            viewport.physical_size[1],
        ),
        depth_min: viewport.depth_min,
        depth_max: viewport.depth_max,
    }
}

fn viewport_rect_to_asset(viewport: RenderViewportRect) -> SceneViewportRectAsset {
    SceneViewportRectAsset {
        physical_position: [viewport.physical_position.x, viewport.physical_position.y],
        physical_size: [viewport.physical_size.x, viewport.physical_size.y],
        depth_min: viewport.depth_min,
        depth_max: viewport.depth_max,
    }
}

pub(super) fn camera_to_asset(
    project: &ProjectManager,
    camera: CameraComponent,
    post_process_settings: Option<ScenePostProcessSettingsAsset>,
) -> Result<SceneCameraAsset, SceneProjectError> {
    Ok(SceneCameraAsset {
        core_pipeline: camera.core_pipeline,
        projection_mode: camera.projection_mode,
        fov_y_radians: camera.fov_y_radians,
        ortho_size: camera.ortho_size,
        z_near: camera.z_near,
        z_far: camera.z_far,
        target: camera_target_to_asset(project, camera.target)?,
        viewport: camera.viewport.map(viewport_rect_to_asset),
        order: camera.order,
        active: camera.is_active,
        hdr: camera.hdr,
        exposure_ev100: camera.exposure_ev100,
        clear_color: camera.clear_color,
        msaa_samples: camera.msaa_samples,
        post_process_settings,
    })
}
