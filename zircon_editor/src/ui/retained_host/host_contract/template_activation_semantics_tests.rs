use super::asset::{asset_primary_activation, AssetPrimaryActivationKind};
use super::route::{primary_activation_route, TemplatePrimaryActivationRoute};
use crate::ui::retained_host::callback_dispatch::WORKBENCH_COMMAND_PALETTE_CONTROL_ID;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::template_component_family::TemplateComponentFamily;
use crate::ui::retained_host::primitives::SharedString;

#[test]
fn text_input_family_routes_to_focus_only_activation() {
    let mut hit = hit_with_kind("");
    hit.component_family = Some(TemplateComponentFamily::TextInput);
    hit.binding_id = "TextField.Binding".into();

    assert_eq!(
        primary_activation_route(&hit),
        TemplatePrimaryActivationRoute::TextInputFocusOnly
    );
}

#[test]
fn workbench_option_route_does_not_fall_back_to_binding() {
    let mut hit = hit_with_kind("workbench_option");
    hit.binding_id = "Dropdown.Binding".into();
    hit.value_text = "selected-option".into();

    assert_eq!(
        primary_activation_route(&hit),
        TemplatePrimaryActivationRoute::WorkbenchOption
    );
}

#[test]
fn command_palette_option_routes_to_commit_activation() {
    let mut hit = hit_with_kind("workbench_option");
    hit.control_id = WORKBENCH_COMMAND_PALETTE_CONTROL_ID.into();
    hit.value_text = "workbench.project.open".into();

    assert_eq!(
        primary_activation_route(&hit),
        TemplatePrimaryActivationRoute::CommandPaletteOption
    );
}

#[test]
fn workbench_menu_item_routes_as_surface_action() {
    let mut hit = hit_with_kind("workbench_menu_item");
    hit.action_id = "workbench.menu.open".into();
    hit.binding_id = "MenuBindingShouldNotWin".into();

    assert_eq!(
        primary_activation_route(&hit),
        TemplatePrimaryActivationRoute::WorkbenchMenuItem
    );
}

#[test]
fn export_wizard_panel_route_prefers_action_over_binding() {
    let mut hit = hit_with_kind("export_wizard_panel");
    hit.action_id = "workbench.build_export.execute.desktop_windows".into();
    hit.binding_id = "DesktopExportWizard/Start".into();

    assert_eq!(
        primary_activation_route(&hit),
        TemplatePrimaryActivationRoute::SurfaceAction
    );
}

#[test]
fn asset_dispatch_source_and_change_controls_are_classified() {
    let mut hit = hit_with_kind("asset:browser");
    hit.action_id = "workbench.asset.search.edit".into();

    let activation = asset_primary_activation(&hit).expect("asset dispatch should route");

    assert_eq!(activation.source.as_str(), "browser");
    assert_eq!(activation.control_id.as_str(), "SearchEdited");
    assert_eq!(activation.kind, AssetPrimaryActivationKind::Change);
}

#[test]
fn asset_dispatch_uses_control_id_when_action_is_empty() {
    let hit = hit_with_kind("asset");

    let activation = asset_primary_activation(&hit).expect("asset dispatch should route");

    assert_eq!(activation.source.as_str(), "activity");
    assert_eq!(activation.control_id.as_str(), "Control");
    assert_eq!(activation.kind, AssetPrimaryActivationKind::Click);
}

fn hit_with_kind(dispatch_kind: &str) -> TemplateNodePointerHit {
    TemplateNodePointerHit {
        control_id: "Control".into(),
        action_id: SharedString::new(),
        binding_id: SharedString::new(),
        dispatch_kind: dispatch_kind.into(),
        component_role: SharedString::new(),
        component_family: None,
        value_text: SharedString::new(),
        edit_action_id: SharedString::new(),
        commit_action_id: SharedString::new(),
        frame: FrameRect::default(),
    }
}
