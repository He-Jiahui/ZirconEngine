use std::collections::BTreeMap;

use super::super::surface_defaults::{projected_surface_variant, projected_text_tone};
use super::super::surface_metrics::{
    projected_border_width, projected_corner_radius, projected_elevation, projected_z_index,
};

pub(super) struct ProjectedSurfaceStyle {
    pub(super) variant: String,
    pub(super) text_tone: String,
    pub(super) corner_radius: f32,
    pub(super) border_width: f32,
    pub(super) elevation: f32,
    pub(super) z_index: i32,
}

pub(super) fn projected_surface_style(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
    node_z_index: i32,
) -> ProjectedSurfaceStyle {
    ProjectedSurfaceStyle {
        variant: projected_surface_variant(attributes, component_role, component_variant),
        text_tone: projected_text_tone(attributes, component_role, component_variant),
        corner_radius: projected_corner_radius(attributes, component_role),
        border_width: projected_border_width(attributes, component_role, component_variant),
        elevation: projected_elevation(attributes, component_role, component_variant),
        z_index: projected_z_index(attributes, component_role, node_z_index),
    }
}
