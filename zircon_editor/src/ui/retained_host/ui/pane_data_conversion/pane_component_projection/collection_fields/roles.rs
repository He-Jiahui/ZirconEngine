use zircon_runtime_interface::ui::component::{UiValue, UiValueKind};

use super::type_tokens::collection_type_is_numeric;

pub(super) fn collection_field_role(declared_type: &str, value: Option<&UiValue>) -> &'static str {
    let declared_type = declared_type.to_ascii_lowercase();
    if declared_type.contains("bool") {
        return "checkbox";
    }
    if declared_type.contains("asset") {
        return "asset-field";
    }
    if declared_type.contains("instance") || declared_type.contains("object") {
        return "object-field";
    }
    if declared_type.contains("color") {
        return "color-field";
    }
    if declared_type.contains("vec") || declared_type.contains("vector") {
        return "vector-field";
    }
    if collection_type_is_numeric(&declared_type) {
        return "number-field";
    }
    if declared_type.contains("ref") {
        return "reference-field";
    }

    match value.map(UiValue::kind) {
        Some(UiValueKind::Bool) => "checkbox",
        Some(UiValueKind::Int) | Some(UiValueKind::Float) => "number-field",
        Some(UiValueKind::Color) => "color-field",
        Some(UiValueKind::Vec2) | Some(UiValueKind::Vec3) | Some(UiValueKind::Vec4) => {
            "vector-field"
        }
        Some(UiValueKind::AssetRef) => "asset-field",
        Some(UiValueKind::InstanceRef) => "object-field",
        _ => "text-field",
    }
}

pub(super) fn collection_field_checked(value: &UiValue) -> bool {
    matches!(value, UiValue::Bool(true))
}
