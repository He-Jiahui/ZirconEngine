use super::*;

use crate::ui::layouts::windows::workbench_host_window::HostMenuChromeItemData;
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::document_tabs::{
    document_tab_preferred_width_from_title_width, DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH,
    DOCUMENT_TAB_CLOSE_EXTENT, DOCUMENT_TAB_STRIP_X, DOCUMENT_TAB_TITLE_FONT_SIZE,
};
use crate::ui::workbench::menu_bar::{
    workbench_menu_slot_width_from_label_width, WORKBENCH_MENU_SLOT_FONT_SIZE,
};
use crate::ui::workbench::page_tabs::{
    main_page_tab_close_frame, main_page_tab_preferred_width_from_title_width,
    MAIN_PAGE_TAB_CLOSE_EXTENT, MAIN_PAGE_TAB_TITLE_FONT_SIZE,
};
use zircon_runtime_interface::ui::design_tokens::{
    EditorControlTokens, EditorDensityTokens, EditorTypographyTokens,
};
use zircon_runtime_interface::ui::layout::UiFrame;

mod authored_projection;
mod fallback_projection;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.01,
        "expected {expected:.3}, got {actual:.3}",
    );
}

fn menu_slot_width_from_runtime_text(label: &str) -> f32 {
    workbench_menu_slot_width_from_label_width(measure_runtime_text_width(
        label,
        WORKBENCH_MENU_SLOT_FONT_SIZE,
    ))
}

fn node(nodes: &ModelRc<ViewTemplateNodeData>, control_id: &str) -> ViewTemplateNodeData {
    maybe_node(nodes, control_id)
        .unwrap_or_else(|| panic!("missing projected popup node {control_id}"))
}

fn maybe_node(
    nodes: &ModelRc<ViewTemplateNodeData>,
    control_id: &str,
) -> Option<ViewTemplateNodeData> {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
}

fn test_tab(title: &str, active: bool, closeable: bool) -> TabData {
    test_tab_with_icon(title, "", active, closeable)
}

fn test_tab_with_icon(title: &str, icon_key: &str, active: bool, closeable: bool) -> TabData {
    TabData {
        id: title.into(),
        slot: "document".into(),
        title: title.into(),
        icon_key: icon_key.into(),
        active,
        closeable,
    }
}

fn test_menu_item(
    label: &str,
    shortcut: &str,
    action_id: &str,
    enabled: bool,
) -> HostMenuChromeItemData {
    HostMenuChromeItemData {
        label: label.into(),
        shortcut: shortcut.into(),
        action_id: action_id.into(),
        enabled,
        children: ModelRc::default(),
    }
}

fn test_menu(label: &str) -> HostMenuChromeMenuData {
    HostMenuChromeMenuData {
        label: label.into(),
        popup_width_px: 224.0,
        popup_height_px: 72.0,
        popup_nodes: ModelRc::default(),
        items: ModelRc::default(),
    }
}
