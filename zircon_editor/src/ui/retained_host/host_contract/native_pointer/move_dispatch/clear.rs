use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::template_hover_damage::{
    activity_reference_hover_damage, browser_reference_hover_damage, template_hover_damage,
};
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;

pub(super) fn clear_hovered_template_move(ui: &UiHostWindow) -> NativePointerDispatchResult {
    let before = ui.get_pane_interaction_generation();
    let pane_host = ui.global::<PaneSurfaceHostContext>();
    if before.activity_asset_references_hovered_index >= 0
        || before.activity_asset_used_by_hovered_index >= 0
    {
        pane_host.invoke_asset_reference_pointer_left("activity".into());
    }
    if before.browser_asset_references_hovered_index >= 0
        || before.browser_asset_used_by_hovered_index >= 0
    {
        pane_host.invoke_asset_reference_pointer_left("browser".into());
    }
    ui.clear_hovered_template_node_for_pointer_move();
    let after = ui.get_pane_interaction_generation();
    let reference_damage = merge_hover_damage(
        browser_reference_hover_damage(before.as_ref(), after.as_ref()),
        activity_reference_hover_damage(before.as_ref(), after.as_ref()),
    );
    let damage = merge_hover_damage(
        template_hover_damage(before.as_ref(), after.as_ref()),
        reference_damage,
    );
    if let Some(damage) = damage {
        return NativePointerDispatchResult::region(damage);
    }
    NativePointerDispatchResult::idle()
}

fn merge_hover_damage(
    template_damage: Option<crate::ui::retained_host::host_contract::data::FrameRect>,
    reference_damage: Option<crate::ui::retained_host::host_contract::data::FrameRect>,
) -> Option<crate::ui::retained_host::host_contract::data::FrameRect> {
    match (template_damage, reference_damage) {
        (Some(template_damage), Some(reference_damage)) => {
            Some(union_frame(&template_damage, &reference_damage))
        }
        (Some(damage), None) | (None, Some(damage)) => Some(damage),
        (None, None) => None,
    }
}
