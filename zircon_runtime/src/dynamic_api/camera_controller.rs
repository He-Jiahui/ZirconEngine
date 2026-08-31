use crate::core::framework::camera_controller::OrbitCameraInput;
use crate::core::framework::render::{ProjectionMode, RenderFrameExtract, ViewportCameraSnapshot};
use crate::core::math::{clamp_viewport_size, is_finite_quat, is_finite_vec3, UVec2, Vec2, Vec3};
use crate::input::camera_controller::OrbitCameraController;
use crate::scene::Scene;
use zircon_runtime_interface::{
    ZrRuntimeViewportCameraV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1,
    ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_PERSPECTIVE_V1,
};

const MIN_EDITOR_CAMERA_SCALE: f32 = 1.0e-6;
const MAX_EDITOR_CAMERA_ROTATION_NORMALIZATION_ERROR: f32 = 1.0e-3;

#[derive(Clone, Copy, Debug)]
enum DragState {
    Orbit { last: Vec2 },
    Pan { last: Vec2 },
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeCameraController {
    viewport_size: UVec2,
    orbit: OrbitCameraController,
    drag: Option<DragState>,
    editor_camera: Option<ZrRuntimeViewportCameraV1>,
}

impl RuntimeCameraController {
    pub(super) fn new(viewport_size: UVec2) -> Self {
        Self {
            viewport_size: clamp_viewport_size(viewport_size),
            orbit: OrbitCameraController::with_target(Vec3::ZERO),
            drag: None,
            editor_camera: None,
        }
    }

    pub(super) fn viewport_size(&self) -> UVec2 {
        self.viewport_size
    }

    pub(super) fn resize(&mut self, size: UVec2) {
        self.viewport_size = clamp_viewport_size(size);
    }

    pub(super) fn set_orbit_target(&mut self, target: Vec3) {
        self.orbit.set_target(target);
    }

    pub(super) fn apply_editor_camera(
        &mut self,
        camera: ZrRuntimeViewportCameraV1,
    ) -> Result<bool, &'static str> {
        validate_editor_camera(camera)?;
        if self.editor_camera == Some(camera) {
            return Ok(false);
        }
        self.editor_camera = Some(camera);
        Ok(true)
    }

    pub(super) fn apply_editor_camera_to_extract(&self, extract: &mut RenderFrameExtract) {
        let Some(camera) = self.editor_camera else {
            return;
        };
        apply_editor_camera_to_snapshot(&mut extract.view.camera, camera);
        if let Some(descriptor) = extract.view.selected_camera_descriptor_mut() {
            apply_editor_camera_to_snapshot(&mut descriptor.camera, camera);
        }
    }

    pub(super) fn pointer_moved(&mut self, scene: &mut Scene, position: Vec2) {
        match self.drag.take() {
            Some(DragState::Orbit { last }) => {
                self.apply_orbit(scene, last, position);
                self.drag = Some(DragState::Orbit { last: position });
            }
            Some(DragState::Pan { last }) => {
                self.apply_pan(scene, last, position);
                self.drag = Some(DragState::Pan { last: position });
            }
            None => {}
        }
    }

    pub(super) fn left_pressed(&mut self, _position: Vec2) {}

    pub(super) fn left_released(&mut self) {}

    pub(super) fn right_pressed(&mut self, position: Vec2) {
        self.drag = Some(DragState::Orbit { last: position });
    }

    pub(super) fn right_released(&mut self) {
        self.drag = None;
    }

    pub(super) fn middle_pressed(&mut self, position: Vec2) {
        self.drag = Some(DragState::Pan { last: position });
    }

    pub(super) fn middle_released(&mut self) {
        self.drag = None;
    }

    pub(super) fn scrolled(&mut self, scene: &mut Scene, delta: f32) {
        self.apply_zoom(scene, delta);
    }

    fn apply_orbit(&mut self, scene: &mut Scene, previous: Vec2, current: Vec2) {
        let camera = scene.active_camera();
        let Some(transform) = scene.local_transform(camera) else {
            return;
        };
        let output = self.orbit.update(
            transform,
            OrbitCameraInput::orbit(previous, current).with_viewport_size(self.viewport_size),
        );
        let _ = scene.update_transform(camera, output.transform);
    }

    fn apply_pan(&mut self, scene: &mut Scene, previous: Vec2, current: Vec2) {
        let camera = scene.active_camera();
        let Some(transform) = scene.local_transform(camera) else {
            return;
        };
        let output = self.orbit.update(
            transform,
            OrbitCameraInput::pan(previous, current).with_viewport_size(self.viewport_size),
        );
        let _ = scene.update_transform(camera, output.transform);
    }

    fn apply_zoom(&mut self, scene: &mut Scene, delta: f32) {
        let camera = scene.active_camera();
        let Some(transform) = scene.local_transform(camera) else {
            return;
        };
        let output = self.orbit.update(
            transform,
            OrbitCameraInput::zoom(delta).with_viewport_size(self.viewport_size),
        );
        let _ = scene.update_transform(camera, output.transform);
    }
}

fn validate_editor_camera(camera: ZrRuntimeViewportCameraV1) -> Result<(), &'static str> {
    if camera.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err("unsupported runtime viewport camera version");
    }
    let rotation_length_squared = camera.transform.rotation.length_squared();
    if !is_finite_vec3(camera.transform.translation)
        || !is_finite_quat(camera.transform.rotation)
        || !is_finite_vec3(camera.transform.scale)
        || camera.transform.scale.abs().min_element() <= MIN_EDITOR_CAMERA_SCALE
        || (rotation_length_squared - 1.0).abs() > MAX_EDITOR_CAMERA_ROTATION_NORMALIZATION_ERROR
    {
        return Err("invalid runtime viewport camera transform");
    }
    if !camera.fov_y_radians.is_finite()
        || !camera.ortho_size.is_finite()
        || !camera.z_near.is_finite()
        || !camera.z_far.is_finite()
        || camera.z_near <= 0.0
        || camera.z_far <= camera.z_near
    {
        return Err("invalid runtime viewport camera projection");
    }
    match camera.projection_kind {
        ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_PERSPECTIVE_V1
            if camera.fov_y_radians > 0.0 && camera.fov_y_radians < std::f32::consts::PI => {}
        ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1 if camera.ortho_size > 0.0 => {}
        ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_PERSPECTIVE_V1
        | ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1 => {
            return Err("invalid runtime viewport camera projection geometry");
        }
        _ => return Err("unknown runtime viewport camera projection kind"),
    }
    Ok(())
}

fn apply_editor_camera_to_snapshot(
    snapshot: &mut ViewportCameraSnapshot,
    camera: ZrRuntimeViewportCameraV1,
) {
    snapshot.transform = camera.transform;
    snapshot.projection_mode = match camera.projection_kind {
        ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1 => ProjectionMode::Orthographic,
        _ => ProjectionMode::Perspective,
    };
    snapshot.fov_y_radians = camera.fov_y_radians;
    snapshot.ortho_size = camera.ortho_size;
    snapshot.z_near = camera.z_near;
    snapshot.z_far = camera.z_far;
    snapshot.projection_override = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_runtime_camera_controller_scroll_uses_runtime_orbit_controller() {
        let mut scene = Scene::new();
        let camera = scene.active_camera();
        let before = scene.find_node(camera).unwrap().transform;
        let mut controller = RuntimeCameraController::new(UVec2::new(800, 600));

        controller.set_orbit_target(Vec3::ZERO);
        controller.scrolled(&mut scene, 1.0);

        let after = scene.find_node(camera).unwrap().transform;
        assert!(after.translation.length() < before.translation.length());
    }

    #[test]
    fn dynamic_runtime_camera_controller_reads_only_the_camera_transform() {
        let source = include_str!("camera_controller.rs");
        let full_node_read = ["scene.find_node(", "scene.active_camera())"].concat();
        assert!(
            !source.contains(&full_node_read),
            "camera input must not project and clone a full SceneNode"
        );
        let local_transform_read = ["scene.", "local_transform(camera)"].concat();
        assert_eq!(source.matches(&local_transform_read).count(), 3);
    }

    #[test]
    fn editor_camera_override_changes_only_the_render_extract_view() {
        let scene = Scene::new();
        let active_camera = scene.active_camera();
        let world_transform_before = scene.world_transform(active_camera).unwrap();
        let mut extract = scene.to_render_frame_extract();
        let original_pipeline = extract.view.camera.core_pipeline;
        let original_exposure = extract.view.camera.exposure_ev100;
        let mut controller = RuntimeCameraController::new(UVec2::new(800, 600));
        let camera = ZrRuntimeViewportCameraV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            crate::core::math::Transform::from_translation(Vec3::new(4.0, 5.0, 6.0)),
            ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1,
            60.0_f32.to_radians(),
            12.0,
            0.25,
            500.0,
        );

        assert!(controller.apply_editor_camera(camera).unwrap());
        controller.apply_editor_camera_to_extract(&mut extract);

        assert_eq!(extract.view.camera.transform, camera.transform);
        assert_eq!(
            extract.view.camera.projection_mode,
            ProjectionMode::Orthographic
        );
        assert_eq!(extract.view.camera.core_pipeline, original_pipeline);
        assert_eq!(extract.view.camera.exposure_ev100, original_exposure);
        assert_eq!(
            scene.world_transform(active_camera),
            Some(world_transform_before)
        );
    }
}
