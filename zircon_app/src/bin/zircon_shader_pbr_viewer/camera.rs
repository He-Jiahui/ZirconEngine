use zircon_runtime::core::framework::render::{
    CameraRenderDescriptor, ProjectionMode, RenderLayerSet, ViewportCameraSnapshot,
    DEFAULT_RENDER_LAYER_MASK,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3};

pub(crate) const SPHERE_CENTER: Vec3 = Vec3::new(0.0, -0.12, 0.0);
pub(crate) const SPHERE_SCALE: [f32; 3] = [1.35, 1.35, 1.35];
pub(crate) const DEFAULT_CAMERA_RADIUS: f32 = 4.2;
pub(crate) const CAMERA_FOV_Y_RADIANS: f32 = 60.0_f32.to_radians();

const MIN_CAMERA_RADIUS: f32 = 2.4;
const MAX_CAMERA_RADIUS: f32 = 12.0;
const CAMERA_PITCH_LIMIT_DEGREES: f32 = 150.0;
const CAMERA_DRAG_DEGREES_PER_PIXEL: f32 = 0.35;

#[derive(Clone, Copy, Debug)]
pub(crate) struct OrbitCamera {
    yaw_degrees: f32,
    pitch_degrees: f32,
    radius: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw_degrees: 0.0,
            pitch_degrees: 0.0,
            radius: DEFAULT_CAMERA_RADIUS,
        }
    }
}

impl OrbitCamera {
    pub(crate) fn drag(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw_degrees += delta_x * CAMERA_DRAG_DEGREES_PER_PIXEL;
        self.pitch_degrees = (self.pitch_degrees - delta_y * CAMERA_DRAG_DEGREES_PER_PIXEL)
            .clamp(-CAMERA_PITCH_LIMIT_DEGREES, CAMERA_PITCH_LIMIT_DEGREES);
    }

    pub(crate) fn zoom(&mut self, wheel_y: f32) {
        let zoom = (1.0 - wheel_y * 0.08).clamp(0.7, 1.35);
        self.radius = (self.radius * zoom).clamp(MIN_CAMERA_RADIUS, MAX_CAMERA_RADIUS);
    }

    fn eye(self) -> Vec3 {
        let yaw = self.yaw_degrees.to_radians();
        let pitch = self.pitch_degrees.to_radians();
        let cos_pitch = pitch.cos();
        SPHERE_CENTER
            + Vec3::new(
                self.radius * yaw.sin() * cos_pitch,
                self.radius * pitch.sin(),
                self.radius * yaw.cos() * cos_pitch,
            )
    }
}

pub(crate) fn camera_render_descriptor(
    camera: &OrbitCamera,
    viewport_size: UVec2,
) -> CameraRenderDescriptor {
    let eye = camera.eye();
    let forward = (SPHERE_CENTER - eye).normalize_or_zero();
    let mut snapshot = ViewportCameraSnapshot::default();
    snapshot.transform = Transform::looking_at(eye, SPHERE_CENTER, stable_camera_up(forward));
    snapshot.projection_mode = ProjectionMode::Perspective;
    snapshot.fov_y_radians = CAMERA_FOV_Y_RADIANS;
    snapshot.z_near = 0.1;
    snapshot.z_far = 100.0;
    snapshot.apply_viewport_size(viewport_size);

    let mut descriptor = CameraRenderDescriptor::from_camera_payload(None, snapshot);
    let default_layers = RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK);
    descriptor.culling_mask = default_layers.clone();
    descriptor.volume_mask = default_layers;
    descriptor.apply_target_size(viewport_size);
    descriptor
}

fn stable_camera_up(forward: Vec3) -> Vec3 {
    if forward.dot(Vec3::Y).abs() > 0.98 {
        Vec3::Z
    } else {
        Vec3::Y
    }
}
