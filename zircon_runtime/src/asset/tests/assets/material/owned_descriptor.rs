use super::*;

#[test]
fn material_owned_lighting_model_drives_standard_descriptor_without_shader_override() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Unlit Grid"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
lighting_model = "unlit"
custom_gain = 2.0
"#,
    )
    .unwrap();

    let descriptor = material.standard_material_descriptor();

    assert_eq!(
        material.lighting_model(),
        RenderMaterialLightingModel::Unlit
    );
    assert_eq!(
        descriptor.lighting_model,
        RenderMaterialLightingModel::Unlit
    );
    assert!(descriptor.unlit);
    assert!(material
        .shader_property_override("lighting_model")
        .is_none());
    assert!(material
        .shader_property_overrides()
        .all(|(name, _)| name != "lighting_model"));
    assert_eq!(
        material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );
    let encoded = material.to_toml_string().unwrap();
    assert!(!encoded.contains("separate_translucency"));
}

#[test]
fn material_owned_receive_shadows_defaults_on_and_can_opt_out_without_shader_override() {
    let default_material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Default Receiver"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"
"#,
    )
    .unwrap();
    let mut no_receive_material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "No Receiver"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
receive_shadows = false
custom_gain = 2.0
"#,
    )
    .unwrap();

    assert!(default_material.receive_shadows());
    assert!(
        default_material
            .standard_material_descriptor()
            .receive_shadows
    );
    assert!(no_receive_material
        .shader_property_override("receive_shadows")
        .is_none());
    assert!(no_receive_material
        .shader_property_overrides()
        .all(|(name, _)| name != "receive_shadows"));
    assert!(!no_receive_material.receive_shadows());
    assert!(
        !no_receive_material
            .standard_material_descriptor()
            .receive_shadows
    );
    assert_eq!(
        no_receive_material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );

    let encoded = no_receive_material.to_toml_string().unwrap();
    assert!(encoded.contains("receive_shadows = false"));
    no_receive_material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(true));
    let encoded = no_receive_material.to_toml_string().unwrap();
    assert!(!encoded.contains("receive_shadows"));
}

#[test]
fn material_owned_separate_translucency_marks_only_the_material_descriptor() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Half Resolution Glass"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
alpha_mode = "blend"
separate_translucency = true
custom_gain = 2.0
"#,
    )
    .unwrap();

    let descriptor = material.standard_material_descriptor();

    assert!(material.separate_translucency());
    assert!(descriptor.separate_translucency);
    assert!(material
        .shader_property_override("separate_translucency")
        .is_none());
    assert!(material
        .shader_property_overrides()
        .all(|(name, _)| name != "separate_translucency"));
    assert_eq!(
        material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );
    let encoded = material.to_toml_string().unwrap();
    assert!(encoded.contains("separate_translucency = true"));
}

#[test]
fn material_owned_cast_shadows_defaults_on_and_can_opt_out_without_shader_override() {
    let default_material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Default Caster"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"
"#,
    )
    .unwrap();
    let mut no_cast_material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "No Caster"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
cast_shadows = false
custom_gain = 2.0
"#,
    )
    .unwrap();

    assert!(default_material.cast_shadows());
    assert!(default_material.standard_material_descriptor().cast_shadows);
    assert!(no_cast_material
        .shader_property_override("cast_shadows")
        .is_none());
    assert!(no_cast_material
        .shader_property_overrides()
        .all(|(name, _)| name != "cast_shadows"));
    assert!(!no_cast_material.cast_shadows());
    assert!(!no_cast_material.standard_material_descriptor().cast_shadows);
    assert_eq!(
        no_cast_material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );

    let encoded = no_cast_material.to_toml_string().unwrap();
    assert!(encoded.contains("cast_shadows = false"));
    no_cast_material
        .property_values
        .insert("cast_shadows".to_string(), toml::Value::Boolean(true));
    let encoded = no_cast_material.to_toml_string().unwrap();
    assert!(!encoded.contains("cast_shadows"));
}

#[test]
fn material_owned_sort_fields_drive_standard_descriptor_without_shader_override() {
    let mut material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Queue Shifted"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
render_queue = -20
material_queue = 7
depth_bias = -0.25
custom_gain = 2.0
"#,
    )
    .unwrap();

    let descriptor = material.standard_material_descriptor();

    assert_eq!(material.render_queue(), -20);
    assert_eq!(material.material_queue(), 7);
    assert_eq!(material.depth_bias(), -0.25);
    assert_eq!(descriptor.render_queue, -20);
    assert_eq!(
        descriptor.render_queue_value,
        Some(RenderQueueValue::new(1_980))
    );
    assert_eq!(
        descriptor.resolved_render_queue_value(),
        RenderQueueValue::new(1_980)
    );
    assert_eq!(descriptor.material_queue, 7);
    assert_eq!(descriptor.depth_bias, -0.25);
    for name in ["render_queue", "material_queue", "depth_bias"] {
        assert!(material.shader_property_override(name).is_none());
        assert!(material
            .shader_property_overrides()
            .all(|(property, _)| property != name));
    }
    assert_eq!(
        material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );

    let encoded = material.to_toml_string().unwrap();
    assert!(encoded.contains("render_queue = -20"));
    assert!(encoded.contains("material_queue = 7"));
    assert!(encoded.contains("depth_bias = -0.25"));

    material
        .property_values
        .insert("render_queue".to_string(), toml::Value::Integer(0));
    material
        .property_values
        .insert("material_queue".to_string(), toml::Value::Integer(0));
    material
        .property_values
        .insert("depth_bias".to_string(), toml::Value::Float(0.0));
    let encoded = material.to_toml_string().unwrap();
    assert!(!encoded.contains("render_queue"));
    assert!(!encoded.contains("material_queue"));
    assert!(!encoded.contains("depth_bias"));
}

#[test]
fn material_owned_render_queue_value_resolves_unity_queue_override() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Late Opaque"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
render_queue = 2900
"#,
    )
    .unwrap();

    let descriptor = material.standard_material_descriptor();

    assert_eq!(material.render_queue(), 2_900);
    assert_eq!(
        material.render_queue_value(),
        Some(RenderQueueValue::new(2_900))
    );
    assert_eq!(descriptor.render_queue, 2_900);
    assert_eq!(
        descriptor.render_queue_value,
        Some(RenderQueueValue::new(2_900))
    );
    assert_eq!(
        descriptor.resolved_render_queue_value(),
        RenderQueueValue::new(2_900)
    );
    assert!(material.validation_errors().iter().all(|error| !matches!(
        error,
        RenderMaterialValidationError::RenderQueueAlphaModeConflict { .. }
    )));
}

#[test]
fn material_owned_render_queue_reports_blend_queue_alpha_conflict() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Broken Glass Queue"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
render_queue = 2000

[overrides.alpha_mode]
mode = "blend"
"#,
    )
    .unwrap();

    let errors = material.validation_errors();

    assert_eq!(
        material.render_queue_value(),
        Some(RenderQueueValue::new(2_000))
    );
    assert!(errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::RenderQueueAlphaModeConflict {
            source,
            path,
            alpha_mode,
            render_queue,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.render_queue"
            && alpha_mode == "blend"
            && *render_queue == 2_000
            && expected == "transparent material queue greater than 2500"
    )));

    let report = material.readiness_report();
    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::RenderQueueAlphaModeConflict { .. }
    )));
}

#[test]
fn material_owned_taa_reactive_mask_strength_drives_standard_descriptor_without_shader_override() {
    let mut material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Responsive Glass"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
taa_reactive_mask_strength = 0.65
custom_gain = 2.0
"#,
    )
    .unwrap();

    let descriptor = material.standard_material_descriptor();

    assert_eq!(material.taa_reactive_mask_strength(), 0.65);
    assert_eq!(descriptor.taa_reactive_mask_strength, 0.65);
    assert!(material
        .shader_property_override("taa_reactive_mask_strength")
        .is_none());
    assert!(material
        .shader_property_overrides()
        .all(|(property, _)| property != "taa_reactive_mask_strength"));
    assert_eq!(
        material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );

    let encoded = material.to_toml_string().unwrap();
    assert!(encoded.contains("taa_reactive_mask_strength"));
    assert_eq!(
        MaterialAsset::from_toml_str(&encoded)
            .unwrap()
            .taa_reactive_mask_strength(),
        0.65
    );

    material.property_values.insert(
        "taa_reactive_mask_strength".to_string(),
        toml::Value::Float(0.0),
    );
    let encoded = material.to_toml_string().unwrap();
    assert!(!encoded.contains("taa_reactive_mask_strength"));
}

#[test]
fn material_owned_occlusion_strength_defaults_and_drives_standard_descriptor() {
    let default_material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Default Occlusion"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"
"#,
    )
    .unwrap();
    let mut material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Occlusion Strength"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
occlusion_strength = 0.25
custom_gain = 2.0
"#,
    )
    .unwrap();

    assert_eq!(default_material.occlusion_strength(), 1.0);
    assert_eq!(
        default_material
            .standard_material_descriptor()
            .occlusion_strength,
        1.0
    );
    assert_eq!(material.occlusion_strength(), 0.25);
    assert_eq!(
        material.standard_material_descriptor().occlusion_strength,
        0.25
    );
    assert!(material
        .shader_property_override("occlusion_strength")
        .is_none());
    assert!(material
        .shader_property_overrides()
        .all(|(property, _)| property != "occlusion_strength"));
    assert_eq!(
        material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );

    let encoded = material.to_toml_string().unwrap();
    assert!(encoded.contains("occlusion_strength = 0.25"));
    assert_eq!(
        MaterialAsset::from_toml_str(&encoded)
            .unwrap()
            .occlusion_strength(),
        0.25
    );

    material
        .property_values
        .insert("occlusion_strength".to_string(), toml::Value::Float(1.0));
    let encoded = material.to_toml_string().unwrap();
    assert!(!encoded.contains("occlusion_strength"));
}

#[test]
fn material_owned_normal_scale_defaults_serializes_and_drives_standard_descriptor() {
    let default_material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Default Normal Scale"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"
"#,
    )
    .unwrap();
    let mut material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Normal Scale"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
normal_scale = 0.35
custom_gain = 2.0
"#,
    )
    .unwrap();

    assert_eq!(default_material.normal_scale(), 1.0);
    assert_eq!(
        default_material.standard_material_descriptor().normal_scale,
        1.0
    );
    assert_eq!(material.normal_scale(), 0.35);
    assert_eq!(material.standard_material_descriptor().normal_scale, 0.35);
    assert!(material.shader_property_override("normal_scale").is_none());
    assert!(material
        .shader_property_overrides()
        .all(|(property, _)| property != "normal_scale"));
    assert_eq!(
        material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );

    let encoded = material.to_toml_string().unwrap();
    assert!(encoded.contains("normal_scale = 0.35"));
    assert_eq!(
        MaterialAsset::from_toml_str(&encoded)
            .unwrap()
            .normal_scale(),
        0.35
    );

    material
        .property_values
        .insert("normal_scale".to_string(), toml::Value::Float(1.0));
    let encoded = material.to_toml_string().unwrap();
    assert!(!encoded.contains("normal_scale"));
}
