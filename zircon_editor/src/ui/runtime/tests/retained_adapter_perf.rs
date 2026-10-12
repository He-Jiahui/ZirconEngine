#[test]
fn retained_projection_maps_properties_once_and_reuses_parsed_options() {
    let source = include_str!("../retained_adapter.rs");
    let cloned_attributes = ["node", ".attributes", ".clone()"].concat();
    let duplicated_options = ["options_text_", "attribute(&node.attributes"].concat();

    assert!(!source.contains(&cloned_attributes));
    assert!(!source.contains(&duplicated_options));
}
