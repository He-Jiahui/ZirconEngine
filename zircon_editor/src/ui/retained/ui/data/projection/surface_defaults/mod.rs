#[path = "../../surface_defaults/alert_dialog.rs"]
mod alert_dialog;
#[path = "badge.rs"]
mod badge;
#[path = "chip.rs"]
mod chip;
#[path = "../../surface_defaults/component_variant.rs"]
mod component_variant;
#[path = "shared.rs"]
mod shared;
#[path = "skeleton.rs"]
mod skeleton;
#[path = "surface.rs"]
mod surface;
#[path = "text_tone.rs"]
mod text_tone;
#[path = "validation.rs"]
mod validation;

use std::collections::BTreeMap;

pub(super) fn projected_component_variant(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
) -> String {
    component_variant::projected_component_variant(attributes, component_role)
}

pub(super) fn projected_surface_variant(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> String {
    surface::projected_surface_variant(attributes, component_role, component_variant)
}

pub(super) fn projected_text_tone(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> String {
    text_tone::projected_text_tone(attributes, component_role, component_variant)
}

pub(super) fn projected_validation_level(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    disabled: bool,
    has_component_descriptor: bool,
) -> String {
    validation::projected_validation_level(
        attributes,
        component_role,
        disabled,
        has_component_descriptor,
    )
}
