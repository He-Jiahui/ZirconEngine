use crate::core::math::{Real, Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::AmbientLight",
    script_visibility = "public"
)]
pub struct AmbientLight {
    pub color: Vec3,
    pub intensity: Real,
    pub affects_lightmapped_meshes: bool,
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: Vec3::splat(1.0),
            intensity: 80.0,
            affects_lightmapped_meshes: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::DirectionalLight",
    script_visibility = "public"
)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.4, -1.0, -0.25).normalize_or_zero(),
            color: Vec3::splat(1.0),
            intensity: 2.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::PointLight",
    script_visibility = "public"
)]
pub struct PointLight {
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Vec3::splat(1.0),
            intensity: 4.0,
            range: 8.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::RectLight",
    script_visibility = "public"
)]
pub struct RectLight {
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
    pub size: Vec2,
}

impl Default for RectLight {
    fn default() -> Self {
        Self {
            color: Vec3::splat(1.0),
            intensity: 1_000_000.0,
            range: 20.0,
            size: Vec2::new(1.0, 1.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::SpotLight",
    script_visibility = "public"
)]
pub struct SpotLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
    pub inner_angle_radians: Real,
    pub outer_angle_radians: Real,
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::splat(1.0),
            intensity: 8.0,
            range: 12.0,
            inner_angle_radians: 0.3,
            outer_angle_radians: 0.55,
        }
    }
}
