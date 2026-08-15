pub(super) use std::cell::RefCell;
pub(super) use std::path::PathBuf;
pub(super) use std::rc::Rc;

pub(super) use crate::ui::retained_host::callback_dispatch::{
    BuiltinHostWindowTemplateBridge, BuiltinWorkbenchWindowTemplateSurfaceBridge,
};
pub(super) use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;
pub(super) use crate::ui::retained_host::{
    HostWindowPresentationData, PaneSurfaceHostContext, TemplatePaneMenuItemData,
    TemplatePaneNodeData, TemplatePaneOptionData, UiHostWindow, WorkbenchContextMenuRequestData,
    paint_runtime_render_commands_for_test, to_host_contract_workbench_window_nodes,
};
pub(super) use zircon_runtime_interface::ui::{
    binding::UiEventKind, layout::UiFrame, layout::UiSize,
};

pub(super) const WORKBENCH_REFERENCE_IMAGE_CONTROL_ID: &str = "WorkbenchShellReferenceImage";
pub(super) const WORKBENCH_REFERENCE_WINDOW_CONTROL_ID: &str = "WorkbenchReferenceImage";
pub(super) const WORKBENCH_REFERENCE_WIDTH: u32 = 1672;
pub(super) const WORKBENCH_REFERENCE_HEIGHT: u32 = 941;
pub(super) const OUTSIDE_WORKBENCH_POPUP_X: f32 = 16.0;
pub(super) const OUTSIDE_WORKBENCH_POPUP_Y: f32 = 16.0;
pub(super) const WORKBENCH_PREVIEW_CAPTURE_ENV: &str = "ZIRCON_WRITE_WORKBENCH_PREVIEW";
pub(super) const WORKBENCH_PREVIEW_CAPTURE_PATH_ENV: &str = "ZIRCON_WORKBENCH_PREVIEW_PATH";
pub(super) const COMPONENT_LAB_INPUT_TEXT_COMMIT_ACTION_ID: &str =
    "component_lab.input_text.commit";
pub(super) const WORKBENCH_ABILITY_NAME_EDIT_ACTION_ID: &str = "workbench.module.ability.name.edit";
pub(super) const WORKBENCH_ABILITY_NAME_COMMIT_ACTION_ID: &str =
    "workbench.module.ability.name.commit";
pub(super) const WORKBENCH_MENU_NEW_ACTION_ID: &str = "menu.item.new";
pub(super) const WORKBENCH_MENU_MORE_TOOLS_ACTION_ID: &str = "menu.item.more_tools";

pub(super) fn host_with_componentized_workbench_nodes() -> UiHostWindow {
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    host_with_workbench_bridge(bridge)
}

pub(super) fn host_with_selected_workbench_module_nodes(module_control_id: &str) -> UiHostWindow {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state(module_control_id, UiEventKind::Click)
        .expect("module tab click should dispatch")
        .expect("module tab should have a selection binding");
    host_with_workbench_bridge(bridge)
}

pub(super) fn host_with_open_workbench_dropdown_nodes() -> UiHostWindow {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state("WorkbenchInputDropdown", UiEventKind::Click)
        .expect("dropdown click should dispatch")
        .expect("dropdown should have an open binding");
    host_with_workbench_bridge(bridge)
}

pub(super) fn host_with_workbench_bridge(
    bridge: BuiltinWorkbenchWindowTemplateSurfaceBridge,
) -> UiHostWindow {
    let mut presentation = HostWindowPresentationData::default();
    presentation.workbench_window_nodes =
        to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let ui = UiHostWindow::new().expect("host window should construct for workbench hover test");
    ui.set_host_presentation(presentation);
    ui
}

pub(super) fn workbench_node(
    presentation: &HostWindowPresentationData,
    control_id: &str,
) -> TemplatePaneNodeData {
    (0..presentation.workbench_window_nodes.row_count())
        .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} should project to native host nodes"))
}

pub(super) fn structured_option(node: &TemplatePaneNodeData, row: usize) -> TemplatePaneOptionData {
    node.structured_options
        .row_data(row)
        .unwrap_or_else(|| panic!("structured option row {row} should exist"))
}

pub(super) fn structured_menu_item(
    node: &TemplatePaneNodeData,
    row: usize,
) -> TemplatePaneMenuItemData {
    node.structured_menu_items
        .row_data(row)
        .unwrap_or_else(|| panic!("structured menu item row {row} should exist"))
}

pub(super) fn dropdown_option_row_point(node: &TemplatePaneNodeData, row: usize) -> (f32, f32) {
    let row_height = node.frame.height.max(24.0);
    (
        node.frame.x + 8.0,
        node.frame.y + node.frame.height + 4.0 + row as f32 * row_height + row_height * 0.5,
    )
}

pub(super) fn menu_item_row_point(node: &TemplatePaneNodeData, row: usize) -> (f32, f32) {
    let row_count = node.structured_menu_items.row_count().max(1);
    let row_height = (node.frame.height / row_count as f32).max(24.0);
    (
        node.frame.x + 8.0,
        node.frame.y + row as f32 * row_height + row_height * 0.5,
    )
}

pub(super) fn node_center(node: &TemplatePaneNodeData) -> (f32, f32) {
    (
        node.frame.x + node.frame.width * 0.5,
        node.frame.y + node.frame.height * 0.5,
    )
}

pub(super) fn node_right_center(node: &TemplatePaneNodeData) -> (f32, f32) {
    (
        node.frame.x + (node.frame.width - 16.0).max(1.0),
        node.frame.y + node.frame.height * 0.5,
    )
}

pub(super) fn changed_pixel_count_in_frame(before: &[u8], after: &[u8], frame: UiFrame) -> usize {
    frame_points(frame)
        .filter(|(x, y)| pixel(before, *x, *y) != pixel(after, *x, *y))
        .count()
}

pub(super) fn first_non_black_pixel_in_frame(bytes: &[u8], frame: UiFrame) -> Option<[u8; 4]> {
    frame_points(frame)
        .map(|(x, y)| pixel(bytes, x, y))
        .find(|pixel| *pixel != [0, 0, 0, 255])
}

pub(super) fn frame_points(frame: UiFrame) -> impl Iterator<Item = (u32, u32)> {
    let start_x = frame.x.floor().max(0.0) as u32;
    let start_y = frame.y.floor().max(0.0) as u32;
    let end_x = (frame.x + frame.width)
        .ceil()
        .min(WORKBENCH_REFERENCE_WIDTH as f32) as u32;
    let end_y = (frame.y + frame.height)
        .ceil()
        .min(WORKBENCH_REFERENCE_HEIGHT as f32) as u32;
    (start_y..end_y).flat_map(move |y| (start_x..end_x).map(move |x| (x, y)))
}

pub(super) fn pixel(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * WORKBENCH_REFERENCE_WIDTH + x) * 4) as usize;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

pub(super) fn contains_at_least_distinct_non_black_pixels(bytes: &[u8], minimum: usize) -> bool {
    let mut distinct = Vec::<[u8; 4]>::new();
    for chunk in bytes.chunks_exact(4) {
        let pixel = [chunk[0], chunk[1], chunk[2], chunk[3]];
        if pixel == [0, 0, 0, 255] || distinct.contains(&pixel) {
            continue;
        }
        distinct.push(pixel);
        if distinct.len() >= minimum {
            return true;
        }
    }
    false
}

pub(super) fn maybe_write_workbench_preview_png(bytes: &[u8]) {
    if std::env::var_os(WORKBENCH_PREVIEW_CAPTURE_ENV).is_none() {
        return;
    }

    let path = std::env::var_os(WORKBENCH_PREVIEW_CAPTURE_PATH_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("docs")
                .join("tests")
                .join("editor")
                .join("editor-workbench-native-1672x941.png")
        });
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("workbench preview output directory should exist");
    }

    image::save_buffer_with_format(
        &path,
        bytes,
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("workbench preview PNG should be written");
}
