use super::*;

#[test]
fn material_asset_management_record_set_sorts_and_summarizes_records() {
    let shader = asset_reference("management-shader", "res://shaders/managed.zshader");
    let albedo = asset_reference("management-albedo", "res://textures/albedo.png");
    let material_with_issues = MaterialAsset {
        name: Some("ManagedGrid".to_string()),
        shader: shader.clone(),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: BTreeMap::from([("roughness".to_string(), toml::Value::Float(0.5))]),
        texture_slots: BTreeMap::from([
            (
                "albedo".to_string(),
                MaterialTextureSlotValue::new(albedo.clone()),
            ),
            (
                "normal".to_string(),
                MaterialTextureSlotValue {
                    reference: None,
                    fallback: Some("normal".to_string()),
                    transform: None,
                    uv_channel: 0,
                },
            ),
        ]),
        validation_diagnostics: vec!["imported with generated material defaults".to_string()],
    };
    let ready_material = MaterialAsset {
        name: Some("ManagedReady".to_string()),
        shader: shader.clone(),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: BTreeMap::new(),
        texture_slots: BTreeMap::new(),
        validation_diagnostics: Vec::new(),
    };
    let first_id = ResourceId::from_stable_label("material:first");
    let second_id = ResourceId::from_stable_label("material:second");

    let overview = material_with_issues.overview();
    assert_eq!(overview.name.as_deref(), Some("ManagedGrid"));
    assert_eq!(overview.shader, shader);
    assert_eq!(overview.property_override_count, 1);
    assert_eq!(overview.texture_slot_count, 2);
    assert_eq!(overview.texture_reference_count, 1);
    assert_eq!(overview.fallback_texture_slot_count, 1);
    assert_eq!(overview.validation_error_count, 0);
    assert_eq!(overview.validation_diagnostic_count, 1);
    assert_eq!(overview.direct_reference_count, 2);

    let record_set = MaterialAssetManagementRecordSet::from_records(vec![
        ready_material.management_record(second_id),
        material_with_issues.management_record(first_id),
    ]);
    let mut expected_ids = vec![first_id, second_id];
    expected_ids.sort();

    assert_eq!(
        record_set
            .records
            .iter()
            .map(|record| record.material_id)
            .collect::<Vec<_>>(),
        expected_ids
    );
    assert_eq!(record_set.summary.material_count, 2);
    assert_eq!(record_set.summary.ready_count, 1);
    assert_eq!(record_set.summary.issue_material_count, 1);
    assert_eq!(record_set.summary.degraded_count(), 1);
    assert_eq!(record_set.summary.property_override_count, 1);
    assert_eq!(record_set.summary.texture_slot_count, 2);
    assert_eq!(record_set.summary.texture_reference_count, 1);
    assert_eq!(record_set.summary.fallback_texture_slot_count, 1);
    assert_eq!(record_set.summary.validation_error_count, 0);
    assert_eq!(record_set.summary.validation_diagnostic_count, 1);
    assert_eq!(record_set.summary.issue_row_count(), 1);
    assert_eq!(record_set.summary.direct_reference_count, 3);
}
