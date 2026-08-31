use std::cell::RefCell;
use std::rc::Rc;

use super::asset::{asset_primary_activation, AssetPrimaryActivationKind};
use super::dispatch::dispatch_template_node_primary_press;
use super::route::{primary_activation_route, TemplatePrimaryActivationRoute};
use crate::ui::retained_host::callback_dispatch::WORKBENCH_COMMAND_PALETTE_CONTROL_ID;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::globals::{HostContractGlobal, HostContractState};
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::template_component_family::TemplateComponentFamily;
use crate::ui::retained_host::primitives::{PhysicalSize, SharedString};
use crate::ui::retained_host::PaneSurfaceHostContext;

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

#[test]
fn asset_dropdown_root_press_is_reserved_for_native_popup_toggle() {
    let mut hit = hit_with_kind("asset:browser");
    hit.control_id = "AssetBrowserKindFilterDropdown".into();
    hit.component_family = Some(TemplateComponentFamily::Dropdown);

    assert!(asset_primary_activation(&hit).is_none());
}

#[test]
fn asset_dropdown_option_routes_as_the_canonical_change() {
    let mut hit = hit_with_kind("asset:browser");
    hit.control_id = "AssetBrowserKindFilterDropdown".into();
    hit.action_id = "AssetSurface/SetKindFilter".into();
    hit.component_family = Some(TemplateComponentFamily::Dropdown);
    hit.value_text = "Texture".into();

    let activation = asset_primary_activation(&hit).expect("asset option should route");

    assert_eq!(activation.source.as_str(), "browser");
    assert_eq!(activation.control_id.as_str(), "SetKindFilter");
    assert_eq!(activation.kind, AssetPrimaryActivationKind::Change);
}

#[test]
fn table_row_primary_press_emits_the_current_typed_selection() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(state);
    let selection = Rc::new(RefCell::new(None));
    let observed_selection = Rc::clone(&selection);
    context.on_template_table_row_selected(
        move |pane_id, control_id, source_index, identity_kind, identity_text| {
            *observed_selection.borrow_mut() = Some((
                pane_id.to_string(),
                control_id.to_string(),
                source_index,
                identity_kind.to_string(),
                identity_text.to_string(),
            ));
        },
    );
    let mut hit = hit_with_kind("");
    hit.pane_id = "plugin.rows".into();
    hit.table_row_source_index = Some(9);
    hit.table_row_identity_kind = "integer".into();
    hit.table_row_identity_text = "73".into();

    dispatch_template_node_primary_press(&context, hit);

    assert_eq!(
        *selection.borrow(),
        Some((
            "plugin.rows".to_string(),
            "Control".to_string(),
            9,
            "integer".to_string(),
            "73".to_string(),
        ))
    );
}

#[test]
fn disabled_table_row_primary_press_does_not_emit_selection() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(state);
    let invocation_count = Rc::new(RefCell::new(0));
    let observed_count = Rc::clone(&invocation_count);
    context.on_template_table_row_selected(move |_, _, _, _, _| *observed_count.borrow_mut() += 1);
    let mut hit = hit_with_kind("");
    hit.pane_id = "plugin.rows".into();
    hit.table_row_source_index = Some(9);
    hit.table_row_identity_kind = "integer".into();
    hit.table_row_identity_text = "73".into();
    hit.disabled = true;

    dispatch_template_node_primary_press(&context, hit);

    assert_eq!(*invocation_count.borrow(), 0);
}

fn hit_with_kind(dispatch_kind: &str) -> TemplateNodePointerHit {
    TemplateNodePointerHit {
        pane_id: SharedString::new(),
        control_id: "Control".into(),
        action_id: SharedString::new(),
        binding_id: SharedString::new(),
        dispatch_kind: dispatch_kind.into(),
        component_role: SharedString::new(),
        component_family: None,
        value_text: SharedString::new(),
        edit_action_id: SharedString::new(),
        commit_action_id: SharedString::new(),
        disabled: false,
        frame: FrameRect::default(),
        table_row_source_index: None,
        table_row_identity_kind: SharedString::new(),
        table_row_identity_text: SharedString::new(),
    }
}
