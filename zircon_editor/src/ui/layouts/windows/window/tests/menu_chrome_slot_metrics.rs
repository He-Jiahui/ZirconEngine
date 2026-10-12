use super::*;
use crate::ui::workbench::menu_bar::WORKBENCH_MENU_SLOT_FONT_SIZE;

#[test]
fn menu_chrome_projects_scaled_text_and_slot_width_together() {
    let metrics = MenuLabelSlotMetrics {
        font_size: WORKBENCH_MENU_SLOT_FONT_SIZE * 1.5,
        logical_font_size: WORKBENCH_MENU_SLOT_FONT_SIZE,
        horizontal_inset: 9.0,
    };
    let menus = model_rc(vec![super::super::HostMenuChromeMenuData {
        label: "Selection".into(),
        popup_width_px: 224.0,
        popup_height_px: 72.0,
        popup_nodes: ModelRc::default(),
        items: ModelRc::default(),
    }]);
    let nodes = expand_menu_chrome_slot_nodes(
        vec![ViewTemplateNodeData {
            control_id: "MenuSlot0".into(),
            frame: ViewTemplateFrameData {
                x: 8.0,
                y: 2.0,
                width: 40.0,
                height: 22.0,
            },
            ..ViewTemplateNodeData::default()
        }],
        &menus,
        metrics,
    );
    let slot = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "MenuSlot0")
        .expect("projected Selection slot");
    assert_eq!(slot.font_size, metrics.logical_font_size);
    assert_eq!(
        slot.frame.width,
        menu_label_slot_width("Selection", metrics)
    );
    assert!(
        slot.frame.width
            > menu_label_slot_width(
                "Selection",
                MenuLabelSlotMetrics {
                    font_size: WORKBENCH_MENU_SLOT_FONT_SIZE,
                    logical_font_size: WORKBENCH_MENU_SLOT_FONT_SIZE,
                    horizontal_inset: 6.0,
                },
            )
    );
}

#[test]
fn menu_chrome_recomposes_slots_when_resolved_font_generation_changes() {
    let menus = model_rc(vec![super::super::HostMenuChromeMenuData {
        label: "Selection".into(),
        popup_width_px: 224.0,
        popup_height_px: 72.0,
        popup_nodes: ModelRc::default(),
        items: ModelRc::default(),
    }]);

    let first = menu_chrome_nodes_with_text_generation(&menus, 420.0, 29.0, [1; 3]);
    let unchanged = menu_chrome_nodes_with_text_generation(&menus, 420.0, 29.0, [1; 3]);
    let resolved = menu_chrome_nodes_with_text_generation(&menus, 420.0, 29.0, [2; 3]);

    assert!(first.shares_values_with(&unchanged));
    assert!(
        !first.shares_values_with(&resolved),
        "resolved font changes must remeasure the menu slots even when labels and geometry are unchanged"
    );
}
