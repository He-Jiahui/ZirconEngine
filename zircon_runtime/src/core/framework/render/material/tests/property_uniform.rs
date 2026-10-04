use super::*;

#[test]
fn material_property_uniform_payload_aligns_and_encodes_numeric_values() {
    let mut values = BTreeMap::new();
    values.insert(
        "enabled".to_string(),
        RenderMaterialPropertyValue::Bool { value: true },
    );
    values.insert(
        "gain".to_string(),
        RenderMaterialPropertyValue::Float { value: 2.5 },
    );
    values.insert(
        "rim".to_string(),
        RenderMaterialPropertyValue::Vec3 {
            value: [0.25, 0.5, 0.75],
        },
    );
    values.insert(
        "tint".to_string(),
        RenderMaterialPropertyValue::Vec4 {
            value: [1.0, 0.5, 0.25, 1.0],
        },
    );

    let payload = RenderMaterialPropertyUniformPayload::from_values(&values);

    assert_eq!(payload.layout[0].name, "enabled");
    assert_eq!(payload.layout[0].offset, 0);
    assert_eq!(u32_at(&payload.bytes, 0), 1);
    assert_eq!(payload.layout[1].name, "gain");
    assert_eq!(payload.layout[1].offset, 4);
    assert_eq!(f32_at(&payload.bytes, 4), 2.5);
    assert_eq!(payload.layout[2].name, "rim");
    assert_eq!(payload.layout[2].offset, 16);
    assert_eq!(f32_at(&payload.bytes, 16), 0.25);
    assert_eq!(f32_at(&payload.bytes, 20), 0.5);
    assert_eq!(f32_at(&payload.bytes, 24), 0.75);
    assert_eq!(payload.layout[3].name, "tint");
    assert_eq!(payload.layout[3].offset, 32);
    assert_eq!(f32_at(&payload.bytes, 32), 1.0);
    assert_eq!(f32_at(&payload.bytes, 36), 0.5);
    assert_eq!(f32_at(&payload.bytes, 40), 0.25);
    assert_eq!(f32_at(&payload.bytes, 44), 1.0);
    assert_eq!(payload.bytes.len(), 48);
    assert!(payload.unsupported.is_empty());
}

#[test]
fn material_property_uniform_payload_records_unsupported_strings() {
    let mut values = BTreeMap::new();
    values.insert(
        "debug_label".to_string(),
        RenderMaterialPropertyValue::String {
            value: "paint".to_string(),
        },
    );
    values.insert(
        "gain".to_string(),
        RenderMaterialPropertyValue::Float { value: 1.0 },
    );

    let payload = RenderMaterialPropertyUniformPayload::from_values(&values);

    assert_eq!(payload.layout.len(), 1);
    assert_eq!(payload.layout[0].name, "gain");
    assert_eq!(payload.unsupported.len(), 1);
    assert_eq!(payload.unsupported[0].name, "debug_label");
    assert_eq!(
        payload.unsupported[0].reason,
        RenderMaterialPropertyUniformUnsupportedReason::UnsupportedType
    );
}

#[test]
fn material_property_uniform_payload_reports_unsupported_diagnostics() {
    let mut values = BTreeMap::new();
    values.insert(
        "debug_label".to_string(),
        RenderMaterialPropertyValue::String {
            value: "paint".to_string(),
        },
    );

    let diagnostics =
        RenderMaterialPropertyUniformPayload::from_values(&values).unsupported_diagnostics();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].source,
        RenderMaterialDiagnosticSource::MaterialUniform
    );
    assert_eq!(diagnostics[0].path, "uniform.debug_label");
    assert_eq!(
        diagnostics[0].diagnostic,
        "material property debug_label cannot be encoded into the renderer uniform payload: unsupported property type"
    );
}

#[test]
fn material_property_uniform_payload_applies_runtime_override_block() {
    let mut values = BTreeMap::new();
    values.insert(
        "enabled".to_string(),
        RenderMaterialPropertyValue::Bool { value: true },
    );
    values.insert(
        "gain".to_string(),
        RenderMaterialPropertyValue::Float { value: 1.0 },
    );
    values.insert(
        "tint".to_string(),
        RenderMaterialPropertyValue::Vec4 {
            value: [1.0, 1.0, 1.0, 1.0],
        },
    );
    let payload = RenderMaterialPropertyUniformPayload::from_values(&values);
    let overrides = MaterialPropertyOverrideBlock::new()
        .with_value("gain", RenderMaterialPropertyValue::Float { value: 2.5 })
        .with_value(
            "tint",
            RenderMaterialPropertyValue::Vec4 {
                value: [0.25, 0.5, 0.75, 1.0],
            },
        );

    let overridden = payload.with_override_block(&overrides);

    assert_eq!(f32_at(&overridden.bytes, 4), 2.5);
    assert_eq!(f32_at(&overridden.bytes, 16), 0.25);
    assert_eq!(f32_at(&overridden.bytes, 20), 0.5);
    assert_eq!(f32_at(&overridden.bytes, 24), 0.75);
    assert_eq!(f32_at(&overridden.bytes, 28), 1.0);
    assert!(overridden.unsupported.is_empty());
}

#[test]
fn material_property_uniform_payload_reports_invalid_override_block_entries() {
    let mut values = BTreeMap::new();
    values.insert(
        "gain".to_string(),
        RenderMaterialPropertyValue::Float { value: 1.0 },
    );
    let payload = RenderMaterialPropertyUniformPayload::from_values(&values);
    let overrides = MaterialPropertyOverrideBlock::new()
        .with_value("gain", RenderMaterialPropertyValue::Bool { value: true })
        .with_value("missing", RenderMaterialPropertyValue::Float { value: 1.0 });

    let overridden = payload.with_override_block(&overrides);

    assert_eq!(f32_at(&overridden.bytes, 0), 1.0);
    assert_eq!(overridden.unsupported.len(), 2);
    assert_eq!(overridden.unsupported[0].name, "gain");
    assert_eq!(
        overridden.unsupported[0].reason,
        RenderMaterialPropertyUniformUnsupportedReason::TypeMismatch
    );
    assert_eq!(overridden.unsupported[1].name, "missing");
    assert_eq!(
        overridden.unsupported[1].reason,
        RenderMaterialPropertyUniformUnsupportedReason::UnknownProperty
    );
}

fn f32_at(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
