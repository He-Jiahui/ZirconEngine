use zircon_math::{Transform, UVec2, Vec3};
use zircon_scene::{ProjectionMode, Scene, ViewOrientation, ViewportCameraSnapshot};

use super::{constants::MIN_CAMERA_DISTANCE, SceneViewportController};

const DEFAULT_CAMERA_DISTANCE: f32 = 8.0;
const DEFAULT_ORTHO_SIZE: f32 = 5.0;

impl SceneViewportController {
    pub(in crate::editing::viewport::controller) fn current_camera(
        &self,
        scene: &Scene,
    ) -> ViewportCameraSnapshot {
        self.state.camera.clone().unwrap_or_else(|| {
            build_scene_camera_snapshot(
                scene,
                self.state.settings.projection_mode,
                self.state.viewport.size,
                DEFAULT_ORTHO_SIZE,
            )
        })
    }

    pub(in crate::editing::viewport::controller) fn set_projection_mode(
        &mut self,
        projection_mode: ProjectionMode,
    ) {
        self.state.settings.projection_mode = projection_mode;
        if let Some(camera) = self.state.camera.as_mut() {
            camera.projection_mode = projection_mode;
        }
    }

    pub(in crate::editing::viewport::controller) fn apply_viewport_size(
        &mut self,
        viewport_size: UVec2,
    ) {
        self.state.viewport = zircon_graphics::ViewportState::new(viewport_size);
        if let Some(camera) = self.state.camera.as_mut() {
            camera.apply_viewport_size(self.state.viewport.size);
        }
    }

    pub(in crate::editing::viewport::controller) fn reset_camera_from_scene(
        &mut self,
        scene: Option<&Scene>,
    ) {
        self.state.camera = scene.map(|scene| {
            build_scene_camera_snapshot(
                scene,
                self.state.settings.projection_mode,
                self.state.viewport.size,
                self.state
                    .camera
                    .as_ref()
                    .map(|camera| camera.ortho_size)
                    .unwrap_or(DEFAULT_ORTHO_SIZE),
            )
        });
    }

    pub(in crate::editing::viewport::controller) fn align_view(
        &mut self,
        orientation: ViewOrientation,
    ) {
        self.state.settings.view_orientation = orientation;
        if orientation == ViewOrientation::User {
            return;
        }

        let distance = self.camera_distance();
        let (direction, up) = orientation_basis(orientation);
        let eye = self.state.orbit_target + direction * distance;
        let mut camera = self.state.camera.clone().unwrap_or_default();
        camera.transform = Transform::looking_at(eye, self.state.orbit_target, up);
        camera.projection_mode = self.state.settings.projection_mode;
        camera.apply_viewport_size(self.state.viewport.size);
        self.state.camera = Some(camera);
    }

    pub(in crate::editing::viewport::controller) fn camera_distance(&self) -> f32 {
        self.state
            .camera
            .as_ref()
            .map(|camera| {
                camera
                    .transform
                    .translation
                    .distance(self.state.orbit_target)
            })
            .unwrap_or(DEFAULT_CAMERA_DISTANCE)
            .max(MIN_CAMERA_DISTANCE)
    }
}

fn build_scene_camera_snapshot(
    scene: &Scene,
    projection_mode: ProjectionMode,
    viewport_size: UVec2,
    ortho_size: f32,
) -> ViewportCameraSnapshot {
    let camera_entity = scene.active_camera();
    let node = scene.find_node(camera_entity);
    let component = node.and_then(|node| node.camera.as_ref());
    let transform = scene
        .world_transform(camera_entity)
        .or_else(|| node.map(|node| node.transform))
        .unwrap_or_else(default_camera_transform);
    let mut snapshot = ViewportCameraSnapshot {
        transform,
        projection_mode,
        fov_y_radians: component
            .map(|camera| camera.fov_y_radians)
            .unwrap_or(60.0_f32.to_radians()),
        ortho_size,
        z_near: component.map(|camera| camera.z_near).unwrap_or(0.1),
        z_far: component.map(|camera| camera.z_far).unwrap_or(200.0),
        aspect_ratio: 16.0 / 9.0,
    };
    snapshot.apply_viewport_size(viewport_size);
    snapshot
}

fn orientation_basis(orientation: ViewOrientation) -> (Vec3, Vec3) {
    match orientation {
        ViewOrientation::PosX => (Vec3::X, Vec3::Y),
        ViewOrientation::NegX => (-Vec3::X, Vec3::Y),
        ViewOrientation::PosY => (Vec3::Y, Vec3::Z),
        ViewOrientation::NegY => (-Vec3::Y, -Vec3::Z),
        ViewOrientation::PosZ => (Vec3::Z, Vec3::Y),
        ViewOrientation::NegZ => (-Vec3::Z, Vec3::Y),
        ViewOrientation::User => (Vec3::Z, Vec3::Y),
    }
}

fn default_camera_transform() -> Transform {
    Transform::looking_at(Vec3::new(0.0, 2.5, 8.0), Vec3::ZERO, Vec3::Y)
}
