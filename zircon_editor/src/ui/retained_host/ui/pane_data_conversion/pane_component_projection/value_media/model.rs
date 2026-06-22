use crate::ui::retained_host::primitives::{Color, Image};

// Projection bundle for value-oriented retained-host fields.
pub(in super::super) struct ProjectedValueMedia {
    pub(in super::super) value_text: String,
    pub(in super::super) value_number: f64,
    pub(in super::super) value_percent: f32,
    pub(in super::super) value_color: Color,
    pub(in super::super) media_source: String,
    pub(in super::super) icon_name: String,
    pub(in super::super) has_preview_image: bool,
    pub(in super::super) preview_image: Image,
    pub(in super::super) vector_components: Vec<f32>,
}
