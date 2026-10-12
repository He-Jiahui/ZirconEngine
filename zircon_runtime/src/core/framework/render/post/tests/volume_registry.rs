use crate::core::framework::render::{
    RenderBloomSettings, RenderColorGradingSettings, RenderPostProcessEffectStackSettings,
};

use super::super::resolved_stack::RenderResolvedPostProcessSettings;
use super::super::volume_component::{
    interp_float_lerp, VolumeComponentDescriptor, VolumeParamSchema, VolumeParamValue,
    BUILTIN_POST_PROCESS_VOLUME_COMPONENTS,
};
use super::{VolumeComponentRegistry, VolumeRegistryError};

const INVALID_PARAM_NAME_PARAMS: [VolumeParamSchema; 1] = [VolumeParamSchema::new(
    "",
    VolumeParamValue::Float(0.0),
    interp_float_lerp,
)];
const DUPLICATE_PARAM_NAME_PARAMS: [VolumeParamSchema; 2] = [
    VolumeParamSchema::new("value", VolumeParamValue::Float(0.0), interp_float_lerp),
    VolumeParamSchema::new("value", VolumeParamValue::Float(1.0), interp_float_lerp),
];

fn read_empty(_settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    Vec::new()
}

fn apply_ok(
    _settings: &mut RenderResolvedPostProcessSettings,
    _component_id: &'static str,
    _values: &[VolumeParamValue],
) -> Result<(), super::super::volume_component::VolumeComponentApplyError> {
    Ok(())
}

#[test]
fn render_volume_registry_exposes_builtin_post_process_components() {
    let registry = VolumeComponentRegistry::with_builtin_post_process_components();
    let expected_ids = [
        "lighting.volumetric-fog",
        "post.ambient-occlusion",
        "post.depth-of-field",
        "post.motion-blur",
        "post.bloom",
        "post.exposure",
        "post.screen-space-reflection",
        "post.screen-space-fog",
        "post.color-grading",
        "post.tonemap",
        "post.vignette",
        "post.grain",
        "post.dither",
        "post.chromatic-aberration",
        "post.color-lookup",
        "post.blur",
    ];

    assert_eq!(registry.len(), expected_ids.len());
    for component_id in expected_ids {
        let descriptor = registry
            .get(component_id)
            .unwrap_or_else(|| panic!("missing built-in volume component `{component_id}`"));
        assert!(
            !descriptor.params.is_empty(),
            "component `{component_id}` should expose at least one parameter"
        );
    }
}

#[test]
fn render_volume_registry_default_stack_matches_existing_defaults() {
    let registry = VolumeComponentRegistry::with_builtin_post_process_components();

    let settings = registry.default_resolved_post_process_settings().unwrap();

    assert_eq!(settings.bloom, RenderBloomSettings::default());
    assert_eq!(
        settings.ambient_occlusion,
        crate::core::framework::render::AoSourceSettings::default()
    );
    assert_eq!(
        settings.color_grading,
        RenderColorGradingSettings::default()
    );
    assert_eq!(
        settings.effect_stack,
        RenderPostProcessEffectStackSettings::default()
    );
}

#[test]
fn render_volume_registry_rejects_duplicate_component_id() {
    let depth_of_field = BUILTIN_POST_PROCESS_VOLUME_COMPONENTS
        .iter()
        .find(|descriptor| descriptor.component_id == "post.depth-of-field")
        .copied()
        .expect("depth-of-field must remain a built-in volume component");
    let mut registry = VolumeComponentRegistry::new();
    registry.register(depth_of_field).unwrap();

    assert_eq!(
        registry.register(depth_of_field),
        Err(VolumeRegistryError::DuplicateComponentId {
            component_id: "post.depth-of-field".to_string(),
        })
    );
    assert_eq!(registry.len(), 1);
    assert_eq!(
        registry
            .get("post.depth-of-field")
            .map(|descriptor| descriptor.component_id),
        Some("post.depth-of-field")
    );
}

#[test]
fn render_volume_registry_index_preserves_registration_order() {
    let bloom = BUILTIN_POST_PROCESS_VOLUME_COMPONENTS
        .iter()
        .find(|descriptor| descriptor.component_id == "post.bloom")
        .copied()
        .expect("bloom must remain a built-in volume component");
    let depth_of_field = BUILTIN_POST_PROCESS_VOLUME_COMPONENTS
        .iter()
        .find(|descriptor| descriptor.component_id == "post.depth-of-field")
        .copied()
        .expect("depth-of-field must remain a built-in volume component");
    let mut registry = VolumeComponentRegistry::new();
    registry.register(bloom).unwrap();
    registry.register(depth_of_field).unwrap();

    let component_ids = registry
        .iter()
        .map(|descriptor| descriptor.component_id)
        .collect::<Vec<_>>();

    assert_eq!(component_ids, ["post.bloom", "post.depth-of-field"]);
    assert_eq!(
        registry
            .get("post.depth-of-field")
            .map(|descriptor| descriptor.component_id),
        Some("post.depth-of-field")
    );
}

#[test]
fn render_volume_registry_rejects_invalid_component_or_param_names() {
    let mut registry = VolumeComponentRegistry::new();

    assert_eq!(
        registry.register(VolumeComponentDescriptor::new(
            "",
            &[],
            read_empty,
            apply_ok
        )),
        Err(VolumeRegistryError::EmptyComponentId)
    );

    assert_eq!(
        registry.register(VolumeComponentDescriptor::new(
            "post.invalid",
            &INVALID_PARAM_NAME_PARAMS,
            read_empty,
            apply_ok,
        )),
        Err(VolumeRegistryError::EmptyParamName {
            component_id: "post.invalid".to_string(),
        })
    );
}

#[test]
fn render_volume_registry_rejects_duplicate_param_names() {
    let mut registry = VolumeComponentRegistry::new();

    assert_eq!(
        registry.register(VolumeComponentDescriptor::new(
            "post.duplicate-param",
            &DUPLICATE_PARAM_NAME_PARAMS,
            read_empty,
            apply_ok,
        )),
        Err(VolumeRegistryError::DuplicateParamName {
            component_id: "post.duplicate-param".to_string(),
            param_name: "value".to_string(),
        })
    );
}
