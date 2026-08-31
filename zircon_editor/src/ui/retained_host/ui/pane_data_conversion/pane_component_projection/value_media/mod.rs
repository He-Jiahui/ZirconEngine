use std::collections::BTreeMap;

use super::super::pane_value_conversion::value_as_f64;
use super::drag_overlay::ProjectedDragOverlayData;
use super::progress_value::projected_value_percent;
use super::value_color::projected_value_color;

mod icon;
mod media;
mod model;
mod number;
mod text;
mod vector;

pub(super) use self::model::ProjectedValueMedia;

pub(super) fn projected_value_media(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
    drag_overlay: &ProjectedDragOverlayData,
) -> ProjectedValueMedia {
    let value_text = text::projected_value_text(component_role, attributes, drag_overlay);
    let value_number = number::projected_value_number(attributes);
    let value_percent = projected_value_percent(
        component_role,
        value_number,
        attributes
            .get("value_percent")
            .or_else(|| attributes.get("progress_percent"))
            .and_then(value_as_f64),
        attributes.get("min").and_then(value_as_f64),
        attributes.get("max").and_then(value_as_f64),
    );
    let media_source = media::projected_media_source(component_role, attributes);
    let icon_name = icon::projected_icon_name(component_role, attributes);
    let has_preview_image = !media_source.trim().is_empty() || !icon_name.trim().is_empty();

    ProjectedValueMedia {
        value_text,
        has_clear_action: component_role == "search-field"
            && attributes
                .get("has_clear_action")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false),
        layout_stepper: component_role == "number-field"
            && attributes
                .get("layout_stepper")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false),
        value_number,
        value_percent,
        value_color: projected_value_color(component_role, attributes),
        media_source,
        icon_name,
        has_preview_image,
        preview_image: Default::default(),
        vector_components: vector::projected_vector_components(attributes),
    }
}
