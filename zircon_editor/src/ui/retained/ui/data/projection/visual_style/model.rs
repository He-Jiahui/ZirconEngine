use zircon_runtime_interface::ui::style::ResolvedButtonStyle;

use super::super::transition_metadata::ProjectedTransitionMetadata;

pub(in super::super) struct ProjectedVisualStyle {
    pub(in super::super) component_category: &'static str,
    pub(in super::super) component_layout_role: &'static str,
    pub(in super::super) component_variant: String,
    pub(in super::super) surface_variant: String,
    pub(in super::super) text_tone: String,
    pub(in super::super) button_variant: String,
    pub(in super::super) button_style: ResolvedButtonStyle,
    pub(in super::super) corner_radius: f32,
    pub(in super::super) border_width: f32,
    pub(in super::super) elevation: f32,
    pub(in super::super) z_index: i32,
    pub(in super::super) transition: ProjectedTransitionMetadata,
}
