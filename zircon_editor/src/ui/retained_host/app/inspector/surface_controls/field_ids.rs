pub(super) fn inspector_field_id(control_id: &str) -> Option<String> {
    if let Some(field_id) = control_id.strip_prefix("DynamicComponentField:") {
        return Some(field_id.to_string());
    }
    match control_id {
        "NameField" => Some("name".to_string()),
        "ParentField" => Some("parent".to_string()),
        "PositionXField" => Some("transform.translation.x".to_string()),
        "PositionYField" => Some("transform.translation.y".to_string()),
        "PositionZField" => Some("transform.translation.z".to_string()),
        _ => None,
    }
}
