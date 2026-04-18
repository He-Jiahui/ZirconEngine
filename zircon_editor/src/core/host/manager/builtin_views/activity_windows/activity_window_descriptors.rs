use crate::view::ViewDescriptor;

use super::asset_browser_view_descriptor::asset_browser_view_descriptor;
use super::prefab_view_descriptor::prefab_view_descriptor;

pub(in crate::core::host::manager::builtin_views) fn activity_window_descriptors() -> Vec<ViewDescriptor>
{
    vec![prefab_view_descriptor(), asset_browser_view_descriptor()]
}
