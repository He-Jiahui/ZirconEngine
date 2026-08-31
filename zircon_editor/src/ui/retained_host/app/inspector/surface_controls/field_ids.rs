use std::borrow::Cow;

pub(super) fn inspector_field_id(control_id: &str) -> Option<Cow<'_, str>> {
    if let Some(field_id) = control_id.strip_prefix("DynamicComponentField:") {
        return Some(Cow::Borrowed(field_id));
    }
    match control_id {
        "NameField" => Some(Cow::Borrowed("name")),
        "ParentField" => Some(Cow::Borrowed("parent")),
        "PositionXField" => Some(Cow::Borrowed("transform.translation.x")),
        "PositionYField" => Some(Cow::Borrowed("transform.translation.y")),
        "PositionZField" => Some(Cow::Borrowed("transform.translation.z")),
        _ => None,
    }
}

#[cfg(test)]
#[path = "field_ids/borrowed_field_tests.rs"]
mod borrowed_field_tests;
