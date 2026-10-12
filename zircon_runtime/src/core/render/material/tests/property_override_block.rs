use super::*;

#[test]
fn material_property_override_block_keeps_transparent_value_map_shape() {
    let block = MaterialPropertyOverrideBlock::new()
        .with_value("gain", RenderMaterialPropertyValue::Float { value: 2.5 });

    let encoded = serde_json::to_string(&block).expect("override block should serialize");
    let decoded: MaterialPropertyOverrideBlock =
        serde_json::from_str(&encoded).expect("override block should deserialize");

    assert!(encoded.contains("gain"));
    assert!(!encoded.contains("values"));
    assert_eq!(decoded, block);
    assert!(serde_json::from_str::<MaterialPropertyOverrideBlock>("{}")
        .expect("empty override map should deserialize")
        .is_empty());
}
