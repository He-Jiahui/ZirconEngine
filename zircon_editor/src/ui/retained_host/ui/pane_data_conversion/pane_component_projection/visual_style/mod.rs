use std::collections::BTreeMap;

mod button;
mod component;
mod model;
mod surface;

pub(super) use self::model::ProjectedVisualStyle;

pub(super) fn projected_visual_style(
    component: &str,
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
    node_z_index: i32,
    popup_open: bool,
) -> ProjectedVisualStyle {
    let component_style =
        component::projected_component_style(component, component_role, attributes);
    let button_style = button::projected_button_style(attributes, component_role);
    let surface_style = surface::projected_surface_style(
        attributes,
        component_role,
        component_style.variant.as_str(),
        node_z_index,
    );

    ProjectedVisualStyle {
        component_category: component_style.category,
        component_layout_role: component_style.layout_role,
        component_variant: component_style.variant,
        surface_variant: surface_style.variant,
        text_tone: surface_style.text_tone,
        button_variant: button_style.variant,
        button_style: button_style.resolved,
        corner_radius: surface_style.corner_radius,
        border_width: surface_style.border_width,
        elevation: surface_style.elevation,
        z_index: surface_style.z_index,
        transition: super::transition_metadata::projected_transition_metadata(
            attributes,
            component_role,
            popup_open,
        ),
    }
}
