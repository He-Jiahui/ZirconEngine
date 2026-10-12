#[path = "attribute_values.rs"]
mod attribute_values;
#[path = "badge.rs"]
mod badge;
#[path = "binding_actions/mod.rs"]
mod binding_actions;
#[path = "button_style.rs"]
mod button_style;
#[path = "clip_frame.rs"]
mod clip_frame;
#[path = "collection_fields/mod.rs"]
mod collection_fields;
#[path = "collection_projection/mod.rs"]
mod collection_projection;
#[path = "collection_window.rs"]
mod collection_window;
#[path = "command_palette/mod.rs"]
mod command_palette;
#[path = "dialog/mod.rs"]
mod dialog;
#[path = "drag_overlay/mod.rs"]
mod drag_overlay;
#[path = "host_template_node.rs"]
mod host_template_node;
#[path = "notification_center/mod.rs"]
mod notification_center;
#[path = "popup_actions/mod.rs"]
mod popup_actions;
#[path = "popup_frame/mod.rs"]
mod popup_frame;
#[path = "progress_value.rs"]
mod progress_value;
#[path = "sample_grid.rs"]
mod sample_grid;
#[path = "selection_options/mod.rs"]
mod selection_options;
#[path = "settings_window/mod.rs"]
mod settings_window;
#[path = "showcase_actions/mod.rs"]
mod showcase_actions;
#[path = "string_lists.rs"]
mod string_lists;
#[path = "surface_defaults/mod.rs"]
mod surface_defaults;
#[path = "surface_metrics/mod.rs"]
mod surface_metrics;
#[path = "node_data/mod.rs"]
mod template_node_data;
#[path = "text_layout/mod.rs"]
mod text_layout;
#[path = "timeline_strip.rs"]
mod timeline_strip;
#[path = "transition_metadata/mod.rs"]
mod transition_metadata;
#[path = "validation_state.rs"]
mod validation_state;
#[path = "value_color.rs"]
mod value_color;
#[path = "value_media/mod.rs"]
mod value_media;
#[path = "visual_state/mod.rs"]
mod visual_state;
#[path = "visual_style/mod.rs"]
mod visual_style;
#[path = "weight_heatmap.rs"]
mod weight_heatmap;
#[path = "world_space/mod.rs"]
mod world_space;

pub(in crate::ui::retained_host::ui) use self::command_palette::projected_command_palette_option_rows;
pub(super) use self::host_template_node::host_template_node;
pub(in crate::ui::retained_host::ui) use self::notification_center::{
    projected_notification_center_metadata, projected_notification_center_metadata_from_host,
    projected_notification_center_option_rows, projected_notification_center_options,
    projected_notification_center_structured_options, projected_notification_center_value_text,
    NotificationCenterMetadata,
};
pub(in crate::ui::retained_host::ui) use self::sample_grid::projected_sample_grid_data;
pub(in crate::ui::retained_host::ui) use self::settings_window::projected_settings_window_data;
pub(in crate::ui::retained_host::ui) use self::timeline_strip::projected_timeline_strip_data;
pub(in crate::ui::retained_host::ui) use self::weight_heatmap::projected_weight_heatmap_data;

#[cfg(test)]
#[path = "tests/drag_overlay.rs"]
mod drag_overlay_tests;
#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
