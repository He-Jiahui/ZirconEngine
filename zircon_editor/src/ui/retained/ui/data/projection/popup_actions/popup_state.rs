use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_f64};
use super::super::drag_overlay::ProjectedDragOverlayData;
use super::super::popup_frame::projected_popup_frame;

pub(super) struct ProjectedPopupState {
    pub(super) popup_open: bool,
    pub(super) has_popup_anchor: bool,
    pub(super) popup_anchor_x: f32,
    pub(super) popup_anchor_y: f32,
    pub(super) frame: host_contract::TemplateNodeFrameData,
}

pub(super) fn projected_popup_state(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    drag_overlay: &ProjectedDragOverlayData,
    frame_x: f32,
    frame_y: f32,
    frame_width: f32,
    frame_height: f32,
) -> ProjectedPopupState {
    let popup_open = attributes
        .get("popup_open")
        .or_else(|| attributes.get("open"))
        .and_then(value_as_bool)
        .unwrap_or(false);
    let popup_open = drag_overlay.popup_open.unwrap_or(popup_open);
    let popup_anchor_x = attributes
        .get("popup_anchor_x")
        .and_then(value_as_f64)
        .map(|value| value as f32);
    let popup_anchor_y = attributes
        .get("popup_anchor_y")
        .and_then(value_as_f64)
        .map(|value| value as f32);
    let has_popup_anchor = popup_anchor_x.is_some() && popup_anchor_y.is_some();
    let frame = projected_popup_frame(
        attributes,
        component_role,
        popup_open,
        popup_anchor_x,
        popup_anchor_y,
        frame_x,
        frame_y,
        frame_width,
        frame_height,
    );

    ProjectedPopupState {
        popup_open,
        has_popup_anchor,
        popup_anchor_x: popup_anchor_x.unwrap_or(0.0),
        popup_anchor_y: popup_anchor_y.unwrap_or(0.0),
        frame,
    }
}
