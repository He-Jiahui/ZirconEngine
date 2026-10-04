use super::*;

#[test]
fn scene_toolbar_keeps_real_operations_and_module_overflow_reachable() {
    for width in [640.0, 1280.0, 1672.0] {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(width, 720.0)).unwrap();
        bridge
            .apply_responsive_toolbar_layout(UiSize::new(width, 720.0))
            .unwrap();
        bridge.recompute_layout(UiSize::new(width, 720.0)).unwrap();
        for control in [
            "WorkbenchToolbarSave",
            "WorkbenchToolSelect",
            "WorkbenchToolMove",
            "WorkbenchToolRotate",
            "WorkbenchToolScale",
            "WorkbenchRunPlay",
            "WorkbenchModuleMore",
        ] {
            assert!(
                bridge.control_frame(control).is_some(),
                "{control} at {width}"
            );
        }
        for control in [
            "WorkbenchToolbarMenu",
            "WorkbenchToolbarAssets",
            "WorkbenchToolbarOpen",
            "WorkbenchModuleCommands",
            "WorkbenchToolbarLayoutGroup",
            "WorkbenchModuleTabs",
        ] {
            assert!(
                bridge.control_frame(control).is_none(),
                "{control} at {width}"
            );
        }
        let save = bridge.control_frame("WorkbenchToolbarSave").unwrap();
        let tool = bridge.control_frame("WorkbenchToolSelect").unwrap();
        let play = bridge.control_frame("WorkbenchRunPlay").unwrap();
        assert!(save.x < tool.x && tool.x < play.x);
        assert!(bridge
            .dispatch_workbench_module_overflow_menu_item_state(
                "WorkbenchModuleOverflowMenu",
                "menu.item.effect",
            )
            .unwrap()
            .is_some());
    }
}

#[test]
fn module_toolbar_priority_uses_mounted_horizontal_scroll_row_at_all_widths() {
    for width in [640.0, 1280.0, 1672.0] {
        let size = UiSize::new(width, 720.0);
        let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(size).unwrap();
        bridge
            .apply_workbench_module_workspace("workbench.module.effect.select")
            .unwrap();
        bridge.apply_responsive_toolbar_layout(size).unwrap();
        bridge.recompute_layout(size).unwrap();
        let projected = resolve_toolbar_priority(&bridge.template_surface, width);
        bridge
            .refresh_workbench_module_overflow_menu_items()
            .unwrap();
        let popup_rows = bridge.control_string_array("WorkbenchModuleOverflowMenu", "menu_items");
        let core_actions = [
            "scene",
            "effect",
            "ability",
            "tags",
            "perception",
            "material",
            "behavior",
            "render",
            "assets",
            "v_f_x",
            "h_u_d",
        ];
        for action in core_actions {
            assert!(
                popup_rows
                    .iter()
                    .any(|row| row.contains(&format!("action=menu.item.{action},"))),
                "actual popup row {action} at {width}: {popup_rows:?}"
            );
        }
        assert_eq!(popup_rows.len(), if width >= 1280.0 { 11 } else { 13 });
        assert!(bridge.control_frame("WorkbenchModuleMore").is_some());
        assert!(bridge.control_frame("WorkbenchModuleSave").is_some());
        if width >= 1280.0 {
            assert!(projected.full_command_set, "wide module row at {width}");
            assert!(
                !projected.compact_module_tabs,
                "wide module tabs at {width}"
            );
            assert!(bridge.control_frame("WorkbenchModuleDiff").is_some());
            assert!(bridge.control_frame("WorkbenchModuleSimulate").is_some());
        } else {
            assert!(!projected.full_command_set);
            assert!(projected.compact_module_tabs);
            assert!(bridge
                .dispatch_workbench_module_overflow_menu_item_state(
                    "WorkbenchModuleOverflowMenu",
                    "menu.item.diff",
                )
                .unwrap()
                .is_some());
            assert!(bridge
                .dispatch_workbench_module_overflow_menu_item_state(
                    "WorkbenchModuleOverflowMenu",
                    "menu.item.sim",
                )
                .unwrap()
                .is_some());
        }
        // All eleven core modules remain real overflow dispatch sources while
        // the scene toolbar's authored module tab row is collapsed.
        for action in [
            "scene",
            "effect",
            "ability",
            "tags",
            "perception",
            "material",
            "behavior",
            "render",
            "assets",
            "v_f_x",
            "h_u_d",
        ] {
            assert!(
                bridge
                    .dispatch_workbench_module_overflow_menu_item_state(
                        "WorkbenchModuleOverflowMenu",
                        &format!("menu.item.{action}"),
                    )
                    .unwrap()
                    .is_some(),
                "{action} at {width}"
            );
        }
    }
}

#[test]
fn content_width_preserves_authored_priority_and_stretch_contract() {
    let authored = AxisConstraint {
        min: 72.0,
        max: 300.0,
        preferred: 300.0,
        priority: 60,
        weight: 2.0,
        stretch_mode: StretchMode::Stretch,
    };

    assert_eq!(
        content_axis(authored, 180.0),
        AxisConstraint {
            min: 72.0,
            max: 180.0,
            preferred: 180.0,
            priority: 60,
            weight: 2.0,
            stretch_mode: StretchMode::Stretch,
        }
    );
    assert_eq!(content_axis(authored, 12.0).preferred, authored.min);
    assert_eq!(content_axis(authored, 12.0).max, authored.min);
}
