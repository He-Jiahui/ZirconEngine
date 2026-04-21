use crate::ui::workbench::view::ViewDescriptor;

use super::super::asset_editor_sessions::ui_asset_editor_view_descriptor;
use super::super::startup::welcome_view_descriptor;
use super::activity_views::activity_view_descriptors::activity_view_descriptors;
use super::activity_windows::activity_window_descriptors::activity_window_descriptors;

pub(crate) fn builtin_view_descriptors() -> Vec<ViewDescriptor> {
    let mut descriptors = activity_view_descriptors();
    descriptors.extend(activity_window_descriptors());
    descriptors.push(ui_asset_editor_view_descriptor());
    descriptors.push(welcome_view_descriptor());
    descriptors
}
