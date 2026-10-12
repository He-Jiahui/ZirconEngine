use zircon_runtime_interface::ui::component::{UiValue, UiValueKind};

use super::type_tokens::CollectionTypeTraits;

pub(super) fn collection_field_role(
    traits: CollectionTypeTraits,
    value: Option<&UiValue>,
) -> &'static str {
    if traits.is_boolean() {
        return "checkbox";
    }
    if traits.is_asset() {
        return "asset-field";
    }
    if traits.is_object_like() {
        return "object-field";
    }
    if traits.is_color() {
        return "color-field";
    }
    if traits.is_vector() {
        return "vector-field";
    }
    if traits.is_numeric() {
        return "number-field";
    }
    if traits.is_reference() {
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
