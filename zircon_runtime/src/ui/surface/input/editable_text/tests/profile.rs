#[test]
fn editable_profile_uses_only_fixed_content_free_counter_names() {
    let source = include_str!("../profile.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);
    for name in [
        "ui_text.edit.state_materializations",
        "ui_text.edit.state_materialized_bytes",
        "ui_text.edit.property_value_clones",
        "ui_text.edit.property_value_clone_bytes",
        "ui_text.edit.property_projections",
        "ui_text.edit.property_projected_bytes",
        "ui_text.edit.committed_projections",
        "ui_text.edit.state_only_projections",
        "ui_text.edit.composition_projections",
        "ui_text.edit.visible_preedit_bytes",
        "ui_text.edit.component_payloads",
        "ui_text.edit.component_payload_bytes",
    ] {
        assert!(production.contains(name), "missing fixed counter {name}");
    }
    assert!(!production.contains("target"));
    assert!(!production.contains("property_name"));
    assert!(!production.contains("source_text"));
}
