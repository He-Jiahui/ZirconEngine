use super::*;

#[test]
fn componentized_workbench_window_template_bridge_projects_chrome_and_core_routes() {
    let _guard = env_lock().lock().unwrap();
    let (mut bridge, nodes) = componentized_workbench_projection_fixture();

    let toast = template_contract_node(&nodes, "WorkbenchToastRoot");
    assert!((toast.layout_offset_x + 15.0).abs() < 0.001);
    assert!((toast.layout_offset_y - 0.0).abs() < 0.001);
    assert!((toast.value_number - 18.0).abs() < 0.001);
    assert_eq!(
        toast.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(32, 159, 169)
    );
    assert_eq!(
        toast.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(35, 143, 152)
    );
    let tooltip = template_contract_node(&nodes, "WorkbenchTooltipRoot");
    assert_eq!(tooltip.role.as_str(), "Tooltip");
    assert_eq!(tooltip.component_role.as_str(), "tooltip");
    assert_eq!(tooltip.text.as_str(), "Tooltip");
    assert_eq!(tooltip.label_text.as_str(), "This is a tooltip");
    assert_eq!(tooltip.frame.width, 110.0);
    assert_eq!(tooltip.frame.height, 78.0);
    assert_eq!(tooltip.layout_icon_size, 18.0);
    assert_eq!(tooltip.layout_content_offset_y, 56.0);
    assert_eq!(tooltip.value_number, 8.0);
    assert_eq!(
        tooltip.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(23, 28, 32)
    );
    assert_eq!(
        tooltip.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(168, 179, 184)
    );
    assert_eq!(
        tooltip.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(37, 156, 167)
    );
    assert_eq!(
        style_color_u8(tooltip.button_style.element.background_color.as_ref()),
        Some([23, 28, 32, 255])
    );
    assert_eq!(
        style_color_u8(tooltip.button_style.element.border_color.as_ref()),
        Some([37, 45, 50, 255])
    );
    assert_eq!(
        style_color_u8(tooltip.button_style.element.foreground_color.as_ref()),
        Some([216, 227, 231, 255])
    );
    let status_grid = template_contract_node(&nodes, "WorkbenchStatusGrid");
    assert!((status_grid.layout_offset_y + 0.5).abs() < 0.001);
    assert_eq!(
        status_grid.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(125, 137, 144)
    );
    assert_eq!(
        status_grid.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(216, 227, 231)
    );
    let status_snap = template_contract_node(&nodes, "WorkbenchStatusSnap");
    assert!((status_snap.layout_offset_y + 0.5).abs() < 0.001);
    assert_eq!(
        status_snap.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(125, 137, 144)
    );
    assert_eq!(
        status_snap.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(216, 227, 231)
    );
    let status_snap_toggle = template_contract_node(&nodes, "WorkbenchStatusSnapToggle");
    let status_world = template_contract_node(&nodes, "WorkbenchStatusWorld");
    let status_target = template_contract_node(&nodes, "WorkbenchStatusTarget");
    let status_zoom = template_contract_node(&nodes, "WorkbenchStatusZoom");
    assert!((status_snap_toggle.layout_offset_y + 0.5).abs() < 0.001);
    assert!((status_world.layout_offset_y + 0.5).abs() < 0.001);
    assert!((status_target.layout_offset_y + 0.5).abs() < 0.001);
    assert!((status_zoom.layout_offset_y + 0.5).abs() < 0.001);
    assert_eq!(
        status_zoom.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(125, 137, 144)
    );
    assert_eq!(
        status_zoom.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(216, 227, 231)
    );
    let inspector_material_row = template_contract_node(&nodes, "WorkbenchMaterialRow");
    assert_eq!(inspector_material_row.text.as_str(), "Materials");
    assert_eq!(
        inspector_material_row.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(154, 165, 171)
    );
    assert_eq!(
        inspector_material_row.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(143, 154, 160)
    );
    assert_eq!(
        inspector_material_row.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(194, 204, 209)
    );
    assert_eq!(
        style_color_u8(
            inspector_material_row
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([23, 28, 32, 255])
    );
    assert_eq!(
        style_color_u8(
            inspector_material_row
                .button_style
                .element
                .border_color
                .as_ref()
        ),
        Some([23, 28, 32, 255])
    );
    assert_eq!(inspector_material_row.layout_icon_size, 15.0);
    let add_component = template_contract_node(&nodes, "WorkbenchAddComponent");
    assert_eq!(
        style_color_u8(add_component.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    let scale_link = template_contract_node(&nodes, "WorkbenchTransformScaleLink");
    assert_eq!(scale_link.layout_offset_x, -12.0);
    assert_eq!(scale_link.layout_icon_size, 17.0);
    assert!(matches!(
        bridge
            .binding_for_control(
                EditorWorkbenchTemplateControlIds::PRIMARY_BUTTON,
                UiEventKind::Click,
            )
            .expect("primary button binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.button.primary"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchTertiaryButton", UiEventKind::Click)
            .expect("tertiary button binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.button.tertiary"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchDropdownButton", UiEventKind::Click)
            .expect("button dropdown open binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.button_dropdown.open"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchDropdownButton", UiEventKind::Change)
            .expect("button dropdown select binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.button_dropdown.select"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchIconToggleSegmented", UiEventKind::Change)
            .expect("icon toggle segmented binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.icon_toggle_segment.select"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchModuleMaterial", UiEventKind::Click)
            .expect("material module tab binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.select"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchModuleCompile", UiEventKind::Click)
            .expect("compile module command binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.compile.invoke"
    ));
    let module_workspace = template_contract_node(&nodes, "WorkbenchMainBandModuleWorkspace");
    assert_eq!(
        module_workspace.control_id.as_str(),
        "WorkbenchMainBandModuleWorkspace"
    );
    for control_id in ["WorkbenchEffectTitle", "WorkbenchMaterialTitle"] {
        assert!(bridge
            .host_projection()
            .node_by_control_id(control_id)
            .unwrap_or_else(|| panic!("missing fixed core workspace title: {control_id}"))
            .routes
            .is_empty());
    }
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchAssetsImportButton")
        .expect("assets import button projection")
        .routes
        .iter()
        .any(|route| route.binding_id == "WorkbenchModule/AssetsImport"
            && route.event_kind == UiEventKind::Click));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchMaterialDomainDropdown")
        .expect("material domain dropdown projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/MaterialDomainEdit"
                && route.event_kind == UiEventKind::Change
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchBehaviorNodeRow03")
        .expect("behavior node row projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/BehaviorNodeCooldown"
                && route.event_kind == UiEventKind::Click
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchVfxSystemField")
        .expect("vfx system field projection")
        .routes
        .iter()
        .any(|route| route.binding_id == "WorkbenchModule/VfxSystemEdit"
            && route.event_kind == UiEventKind::Change));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchModuleAbility")
        .expect("ability module tab projection")
        .routes
        .iter()
        .any(|route| route.binding_id == "WorkbenchModule/Ability"
            && route.event_kind == UiEventKind::Click));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchAbilityNameField")
        .expect("ability name field projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/AbilityNameEdit"
                && route.event_kind == UiEventKind::Change
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchTagsAbilityActivateRow")
        .expect("tags ability row projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/TagsAbilityActivate"
                && route.event_kind == UiEventKind::Click
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchPerceptionConfigDropdown")
        .expect("perception config dropdown projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/PerceptionConfigEdit"
                && route.event_kind == UiEventKind::Change
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchRenderLightingPassRow")
        .expect("render lighting pass row projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/RenderLightingPass"
                && route.event_kind == UiEventKind::Click
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchHudScreenDropdown")
        .expect("hud screen dropdown projection")
        .routes
        .iter()
        .any(|route| route.binding_id == "WorkbenchModule/HudScreenEdit"
            && route.event_kind == UiEventKind::Change));

    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchToolMove", UiEventKind::Click)
            .expect("move tool binding")
            .payload(),
        EditorUiBindingPayload::ViewportCommand(ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Transform(TransformHandleKind::Move)
        ))
    ));

    bridge.recompute_layout(UiSize::new(1440.0, 941.0)).unwrap();
    let resized_viewport_frame = bridge
        .control_frame(EditorWorkbenchTemplateControlIds::VIEWPORT)
        .expect("resized viewport control frame");
    assert_eq!(bridge.frames().viewport, resized_viewport_frame);
    assert_eq!(
        bridge.layout_frames().document_region_frame,
        Some(resized_viewport_frame)
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id(EditorWorkbenchTemplateControlIds::VIEWPORT)
            .expect("resized viewport projection")
            .frame,
        bridge.frames().viewport
    );
}
