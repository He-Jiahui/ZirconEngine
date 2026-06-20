use super::super::globals::PaneSurfaceHostContext;
use super::super::surface_hit_test::TemplateNodePointerHit;
use super::asset::dispatch_asset_template_node_primary_press;
use super::helpers::action_or_control_id;
use super::route::{primary_activation_route, TemplatePrimaryActivationRoute};
use crate::ui::retained_host::callback_dispatch::WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID;

pub(in crate::ui::retained_host::host_contract) fn dispatch_template_node_primary_press(
    pane_host: &PaneSurfaceHostContext<'_>,
    hit: TemplateNodePointerHit,
) {
    match primary_activation_route(&hit) {
        TemplatePrimaryActivationRoute::TextInputFocusOnly => {}
        TemplatePrimaryActivationRoute::Inspector => {
            pane_host.invoke_inspector_control_clicked(hit.control_id)
        }
        TemplatePrimaryActivationRoute::Asset => {
            dispatch_asset_template_node_primary_press(pane_host, hit)
        }
        TemplatePrimaryActivationRoute::Welcome => {
            pane_host.invoke_welcome_control_clicked(action_or_control_id(&hit))
        }
        TemplatePrimaryActivationRoute::Showcase => {
            pane_host.invoke_component_showcase_control_activated(hit.control_id, hit.action_id)
        }
        TemplatePrimaryActivationRoute::CommandPaletteOption => pane_host
            .invoke_surface_control_edited(
                hit.control_id,
                WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID.into(),
                hit.value_text,
            ),
        TemplatePrimaryActivationRoute::WorkbenchOption => pane_host
            .invoke_component_showcase_option_selected(
                hit.control_id,
                hit.action_id,
                hit.value_text,
            ),
        TemplatePrimaryActivationRoute::WorkbenchMenuItem
        | TemplatePrimaryActivationRoute::SurfaceAction => {
            pane_host.invoke_surface_control_clicked(hit.control_id, hit.action_id)
        }
        TemplatePrimaryActivationRoute::SurfaceBinding => {
            pane_host.invoke_surface_control_clicked(hit.control_id, hit.binding_id)
        }
    }
}
