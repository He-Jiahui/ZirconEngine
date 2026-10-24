use std::collections::HashSet;

use super::*;

#[test]
fn ability_editor_menu_actions_are_complete_and_unique() {
    assert_eq!(8, ABILITY_EDITOR_MENU_COMMANDS.len());
    assert_eq!(
        ABILITY_EDITOR_MENU_COMMANDS.len(),
        ABILITY_EDITOR_MENU_COMMANDS
            .iter()
            .map(|command| command.menu_action_id)
            .collect::<HashSet<_>>()
            .len()
    );
    assert!(ABILITY_EDITOR_MENU_COMMANDS
        .iter()
        .all(|command| command.menu_action_id.starts_with("menu.item.ability.")));
    assert!(ABILITY_EDITOR_MENU_COMMANDS.iter().all(|command| {
        command
            .extension_action_id
            .starts_with("workbench.extension.")
    }));
    assert!(ABILITY_EDITOR_MENU_COMMANDS
        .iter()
        .all(|command| command.extension_action_id.ends_with(".open")));
}

#[test]
fn ability_window_menu_opens_blend_space_through_its_live_animation_popup() {
    use crate::ui::retained_host::host_contract::componentized_workbench_regions::{
        assert_control_source_paint_and_hit, assert_module_tools_menu_paint_once, authored_panes,
        module_tools_menu_spec,
    };
    use zircon_runtime_interface::ui::v2::UiTemplateNodeInstancePathStep;
    use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

    const MENU: &str = "WorkbenchAbilityAnimationToolsMenu";
    const TRIGGER: &str = "WorkbenchAbilityAnimationTools";
    const ABILITY: &str = "WorkbenchModuleAbilityWorkspace";
    const BLEND: &str = "WorkbenchExtensionBlendSpaceWorkspace";
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1260.0, 780.0)).unwrap();
    assert!(bridge
        .dispatch_workbench_module_overflow_menu_item_state(
            "UnrealWindowMenuPopup",
            "menu.item.ability",
        )
        .unwrap()
        .is_none());
    bridge.set_preview_features(true).unwrap();
    assert!(bridge
        .dispatch_control_state("MenuSlot2", UiEventKind::Click)
        .unwrap()
        .is_some());
    assert!(bridge
        .dispatch_workbench_module_overflow_menu_item_state(
            "UnrealWindowMenuPopup",
            "menu.item.ability",
        )
        .unwrap()
        .is_some());

    let presentation = ability_presentation(&bridge);
    let root = presentation
        .workbench_window_nodes
        .iter()
        .find(|node| node.control_id.as_str() == ABILITY)
        .unwrap();
    let trigger = presentation
        .workbench_window_nodes
        .iter()
        .find(|node| node.control_id.as_str() == TRIGGER)
        .unwrap();
    assert!(root
        .source_path
        .as_str()
        .ends_with("/ui/editor/modules/core/gameplay/ability_workspace.zui"));
    assert!(trigger
        .source_path
        .as_str()
        .ends_with("/ui/editor/components/workbench/primitives/inputs/button.zui"));
    let root_path =
        serde_json::from_str::<Vec<UiTemplateNodeInstancePathStep>>(root.instance_path.as_str())
            .unwrap();
    let trigger_path =
        serde_json::from_str::<Vec<UiTemplateNodeInstancePathStep>>(trigger.instance_path.as_str())
            .unwrap();
    let spec = module_tools_menu_spec(MENU).unwrap();
    assert!(spec.trigger_source_is_owned(
        root.source_path.as_str(),
        Some(&root_path),
        trigger.source_path.as_str(),
        trigger.source_node_id.as_str(),
        Some(&trigger_path)
    ));
    let retired_primitive = "zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui";
    assert!(!spec.trigger_source_is_owned(
        root.source_path.as_str(),
        Some(&root_path),
        retired_primitive,
        trigger.source_node_id.as_str(),
        Some(&trigger_path),
    ));
    let mut unrelated_caller = trigger_path.clone();
    unrelated_caller.last_mut().unwrap().source_node_id = "unrelated_button".to_string();
    assert!(!spec.trigger_source_is_owned(
        root.source_path.as_str(),
        Some(&root_path),
        trigger.source_path.as_str(),
        trigger.source_node_id.as_str(),
        Some(&unrelated_caller)
    ));
    assert_control_source_paint_and_hit(&presentation, TRIGGER);

    assert!(bridge
        .dispatch_control_state(TRIGGER, UiEventKind::Click)
        .unwrap()
        .is_some());
    assert_module_tools_menu_paint_once(&ability_presentation(&bridge), MENU);
    assert!(bridge
        .dispatch_module_tools_menu_item_state(MENU, "menu.item.ability.blend_space")
        .unwrap()
        .is_some());
    assert!(!bridge.has_control(ABILITY));
    let presentation = ability_presentation(&bridge);
    assert!(authored_panes(&presentation).iter().any(|pane| pane
        .module_workspace
        .is_some_and(|root| root.control_id.as_str() == BLEND)));
    assert!(bridge
        .control_frame("WorkbenchExtensionBlendSpacePreviewViewportImage")
        .is_some());
}

fn ability_presentation(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
) -> crate::ui::retained_host::host_contract::data::HostWindowPresentationData {
    use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
    let mut presentation = HostWindowPresentationData {
        workbench_window_nodes: crate::ui::retained_host::to_host_contract_workbench_window_nodes(
            Some(bridge.host_projection()),
        ),
        ..Default::default()
    };
    let frame = |control| {
        let frame = bridge.control_frame(control).unwrap();
        FrameRect {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        }
    };
    presentation.host_layout.center_band_frame = frame("UnrealLevelEditorContent");
    presentation.host_layout.status_bar_frame = frame("WorkbenchWindowStatusBarRegion");
    presentation
}
