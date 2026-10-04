use super::*;

#[test]
fn material_subsurface_profile_index_rejects_out_of_gpu_table_range() {
    let mut values = BTreeMap::new();
    values.insert(
        SUBSURFACE_PROFILE_PROPERTY.to_string(),
        toml::Value::Integer(16),
    );

    assert_eq!(subsurface_profile_index(&values), None);
    assert_eq!(subsurface_profile_validation_errors(&values).len(), 1);
}
