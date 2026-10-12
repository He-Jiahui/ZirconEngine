#[path = "settings_window/chord_control.rs"]
mod chord_control;
#[path = "settings_window/color_controls.rs"]
mod color_controls;
#[path = "settings_window/commands.rs"]
mod commands;
#[path = "settings_window/enum_controls.rs"]
mod enum_controls;
#[path = "settings_window/field_control.rs"]
mod field_control;
#[path = "settings_window/geometry.rs"]
mod geometry;
#[path = "../../../contract/paint/settings_window/persistence_health.rs"]
mod persistence_health;
#[path = "settings_window/scrollbars.rs"]
mod scrollbars;
#[path = "settings_window/text_control.rs"]
mod text_control;
#[path = "settings_window/visible_rows.rs"]
mod visible_rows;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_settings_window_commands;
