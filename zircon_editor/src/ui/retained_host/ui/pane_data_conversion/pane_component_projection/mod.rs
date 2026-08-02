mod attribute_values;
mod badge;
mod binding_actions;
mod button_style;
mod clip_frame;
mod collection_fields;
mod collection_projection;
mod collection_window;
mod command_palette;
mod dialog;
mod drag_overlay;
mod host_template_node;
mod notification_center;
mod popup_actions;
mod popup_frame;
pub(crate) mod preview_images;
mod progress_value;
mod sample_grid;
mod selection_options;
mod showcase_actions;
mod string_lists;
mod surface_defaults;
mod surface_metrics;
mod template_node_data;
mod text_layout;
mod timeline_strip;
mod transition_metadata;
mod validation_state;
mod value_color;
mod value_media;
mod visual_state;
mod visual_style;
mod weight_heatmap;
mod world_space;

pub(in crate::ui::retained_host::ui) use self::command_palette::{
    projected_command_palette_options, projected_command_palette_structured_options,
};
pub(super) use self::host_template_node::host_template_node;
pub(in crate::ui::retained_host::ui) use self::notification_center::{
    NotificationCenterMetadata, projected_notification_center_metadata,
    projected_notification_center_metadata_from_host, projected_notification_center_option_rows,
    projected_notification_center_options, projected_notification_center_structured_options,
    projected_notification_center_value_text,
};
pub(in crate::ui::retained_host::ui) use self::sample_grid::projected_sample_grid_data;
pub(in crate::ui::retained_host::ui) use self::timeline_strip::projected_timeline_strip_data;
pub(in crate::ui::retained_host::ui) use self::weight_heatmap::projected_weight_heatmap_data;

#[cfg(test)]
mod drag_overlay_tests;
#[cfg(test)]
mod tests;
