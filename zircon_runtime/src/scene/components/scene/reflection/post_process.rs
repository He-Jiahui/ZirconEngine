//! Bounded scalar views of nested post-process settings; these do not replace the profile.
use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

use super::super::PostProcessSettingsComponent;

pub(in crate::scene::components::scene) fn read_ao_intensity(
    component: &PostProcessSettingsComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Scalar(
        component.ambient_occlusion.intensity,
    ))
}

pub(in crate::scene::components::scene) fn read_bloom_threshold(
    component: &PostProcessSettingsComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Scalar(component.bloom.threshold))
}

pub(in crate::scene::components::scene) fn read_bloom_intensity(
    component: &PostProcessSettingsComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Scalar(component.bloom.intensity))
}

pub(in crate::scene::components::scene) fn read_bloom_radius(
    component: &PostProcessSettingsComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Scalar(component.bloom.radius))
}

pub(in crate::scene::components::scene) fn read_color_grading_exposure(
    component: &PostProcessSettingsComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Scalar(component.color_grading.exposure))
}
