use super::*;

#[test]
fn material_owned_receive_shadows_reports_non_bool_override() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Invalid Receiver"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
receive_shadows = "no"
"#,
    )
    .unwrap();

    assert!(material.receive_shadows());
    assert!(material.validation_errors().iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.receive_shadows"
            && name == "receive_shadows"
            && expected == "bool"
    )));
}

#[test]
fn material_owned_cast_shadows_reports_non_bool_override() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Invalid Caster"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
cast_shadows = "no"
"#,
    )
    .unwrap();

    assert!(material.cast_shadows());
    assert!(material.validation_errors().iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.cast_shadows"
            && name == "cast_shadows"
            && expected == "bool"
    )));
}

#[test]
fn material_owned_sort_fields_report_invalid_override_types() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Invalid Queue Fields"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
render_queue = 2.5
material_queue = "front"
depth_bias = "near"
"#,
    )
    .unwrap();

    let errors = material.validation_errors();

    assert_eq!(material.render_queue(), 0);
    assert_eq!(material.material_queue(), 0);
    assert_eq!(material.depth_bias(), 0.0);
    assert!(errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.render_queue"
            && name == "render_queue"
            && expected == "i32"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.material_queue"
            && name == "material_queue"
            && expected == "i32"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.depth_bias"
            && name == "depth_bias"
            && expected == "number"
    )));
}

#[test]
fn material_owned_taa_reactive_mask_strength_reports_invalid_override() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Invalid Responsive"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
taa_reactive_mask_strength = 2.0
"#,
    )
    .unwrap();

    assert_eq!(material.taa_reactive_mask_strength(), 0.0);
    assert!(material.validation_errors().iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.taa_reactive_mask_strength"
            && name == "taa_reactive_mask_strength"
            && expected == "number in 0..=1"
    )));
}

#[test]
fn material_asset_reports_invalid_lighting_model_as_material_validation_error() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Invalid Lighting"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
lighting_model = "toonish"
"#,
    )
    .unwrap();

    let errors = material.validation_errors();

    assert!(errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::InvalidLightingModel { path, value }
            if path == "overrides.lighting_model" && value == "toonish"
    )));
}
