use super::super::surface_hit_test::TemplateNodePointerHit;
use super::super::template_input_semantics::hit_is_text_input;
use crate::ui::retained_host::asset_control_ids::asset_dispatch_source;
use crate::ui::retained_host::callback_dispatch::WORKBENCH_COMMAND_PALETTE_CONTROL_ID;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) enum TemplatePrimaryActivationRoute {
    TextInputFocusOnly,
    Inspector,
    Asset,
    Welcome,
    Showcase,
    CommandPaletteOption,
    WorkbenchOption,
    WorkbenchMenuItem,
    SurfaceBinding,
    SurfaceAction,
}

pub(in crate::ui::retained_host::host_contract) fn primary_activation_route(
    hit: &TemplateNodePointerHit,
) -> TemplatePrimaryActivationRoute {
    if hit_is_text_input(hit) {
        return TemplatePrimaryActivationRoute::TextInputFocusOnly;
    }
    match hit.dispatch_kind.as_str() {
        "inspector" => TemplatePrimaryActivationRoute::Inspector,
        kind if asset_dispatch_source(kind).is_some() => TemplatePrimaryActivationRoute::Asset,
        "welcome" => TemplatePrimaryActivationRoute::Welcome,
        "showcase" => TemplatePrimaryActivationRoute::Showcase,
        "workbench_option" if hit.control_id.as_str() == WORKBENCH_COMMAND_PALETTE_CONTROL_ID => {
            TemplatePrimaryActivationRoute::CommandPaletteOption
        }
        "workbench_option" => TemplatePrimaryActivationRoute::WorkbenchOption,
        "workbench_menu_item" => TemplatePrimaryActivationRoute::WorkbenchMenuItem,
        "export_wizard_panel" => TemplatePrimaryActivationRoute::SurfaceAction,
        _ if !hit.binding_id.is_empty() => TemplatePrimaryActivationRoute::SurfaceBinding,
        _ => TemplatePrimaryActivationRoute::SurfaceAction,
    }
}
