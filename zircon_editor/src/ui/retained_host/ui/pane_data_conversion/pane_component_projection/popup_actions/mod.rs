mod action_buttons;
mod action_ids;
mod drag_payloads;
mod menu;
mod model;
mod popup_state;

use std::collections::BTreeMap;

use crate::ui::template_runtime::RetainedUiHostBindingProjection;
use zircon_runtime_interface::ui::component::UiComponentDescriptor;

use self::action_buttons::projected_actions;
use self::action_ids::projected_action_ids;
use self::drag_payloads::accepted_drag_payloads;
use self::menu::projected_popup_menu;
pub(super) use self::model::ProjectedPopupActions;
use self::popup_state::projected_popup_state;
use super::drag_overlay::ProjectedDragOverlayData;

pub(super) fn projected_popup_actions(
    control_id: &str,
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
    bindings: &[RetainedUiHostBindingProjection],
    component_descriptor: Option<&UiComponentDescriptor>,
    drag_overlay: &ProjectedDragOverlayData,
    disabled: bool,
    frame_x: f32,
    frame_y: f32,
    frame_width: f32,
    frame_height: f32,
) -> ProjectedPopupActions {
    let menu = projected_popup_menu(attributes);
    let popup_state = projected_popup_state(
        attributes,
        component_role,
        drag_overlay,
        frame_x,
        frame_y,
        frame_width,
        frame_height,
    );
    let action_ids = projected_action_ids(
        control_id,
        bindings,
        component_descriptor,
        disabled,
        popup_state.popup_open,
    );

    ProjectedPopupActions {
        menu_items: menu.menu_items,
        structured_menu_items: menu.structured_menu_items,
        popup_open: popup_state.popup_open,
        has_popup_anchor: popup_state.has_popup_anchor,
        popup_anchor_x: popup_state.popup_anchor_x,
        popup_anchor_y: popup_state.popup_anchor_y,
        frame: popup_state.frame,
        actions: projected_actions(
            control_id,
            component_role,
            attributes,
            bindings,
            component_descriptor,
        ),
        accepted_drag_payloads: accepted_drag_payloads(component_descriptor),
        dispatch_kind: action_ids.dispatch_kind,
        action_id: action_ids.action_id,
        binding_id: action_ids.binding_id,
        begin_drag_action_id: action_ids.begin_drag_action_id,
        drag_action_id: action_ids.drag_action_id,
        end_drag_action_id: action_ids.end_drag_action_id,
        commit_action_id: action_ids.commit_action_id,
        edit_action_id: action_ids.edit_action_id,
    }
}
