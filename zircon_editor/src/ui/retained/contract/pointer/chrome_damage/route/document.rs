use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::floating::floating_document_tab_damage_frame;
use super::dock::route_document_dock_damage_frame;

pub(super) fn route_document_tab_damage_frame(
    presentation: &HostWindowPresentationData,
    surface_key: &str,
) -> Option<FrameRect> {
    if let Some(leaf) = presentation
        .host_scene_data
        .document_leaves
        .iter()
        .find(|leaf| leaf.surface_key.as_str() == surface_key)
    {
        return Some(leaf.region_frame.clone());
    }
    let document_surface_key = presentation
        .host_scene_data
        .document_dock
        .surface_key
        .as_str();
    if surface_key == "document" || surface_key == document_surface_key {
        return route_document_dock_damage_frame(presentation);
    }
    floating_document_tab_damage_frame(presentation, surface_key)
}
