use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::{Quat, Real, Vec2, Vec3, Vec4};
use crate::scene::{SceneResult, World};

use super::values::{
    expect_bool, expect_i32, expect_quat, expect_scalar, expect_vec2, expect_vec3, expect_vec4,
    validate_quat_array,
};

impl World {
    pub(in crate::scene::world) fn compiled_property_expect_scalar(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Real> {
        expect_scalar(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_i32(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<i32> {
        expect_i32(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_bool(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        expect_bool(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_vec2(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Vec2> {
        expect_vec2(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_vec3(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Vec3> {
        expect_vec3(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_vec4(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Vec4> {
        expect_vec4(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_expect_quat(
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Quat> {
        expect_quat(value, property_path)
    }

    pub(in crate::scene::world) fn compiled_property_validate_quat_array(
        value: [Real; 4],
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<()> {
        validate_quat_array(value, property_path)
    }
}
