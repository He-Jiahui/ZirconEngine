use super::TextureCopyRegion;

#[test]
fn texture_copy_region_defaults_to_one_layer_and_can_select_a_contiguous_range() {
    let default_region = TextureCopyRegion::new(4, 2);
    let layered_region = default_region.with_depth_or_array_layers(6);

    assert_eq!(default_region.depth_or_array_layers, 1);
    assert_eq!(layered_region.depth_or_array_layers, 6);
    assert_eq!(layered_region.width, 4);
    assert_eq!(layered_region.height, 2);
}

#[test]
fn legacy_serialized_region_defaults_to_one_layer_without_expanding_new_output() {
    let legacy = r#"{"mip_level":0,"origin_x":0,"origin_y":0,"origin_z":0,"width":4,"height":2,"aspect":"All"}"#;
    let region: TextureCopyRegion = serde_json::from_str(legacy).unwrap();

    assert_eq!(region.depth_or_array_layers, 1);
    assert!(!serde_json::to_string(&region)
        .unwrap()
        .contains("depth_or_array_layers"));
}
