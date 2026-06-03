use std::path::PathBuf;

use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::retained_host::{
    paint_runtime_render_commands_for_test, to_host_contract_workbench_window_nodes,
    TemplatePaneMenuItemData, TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiFrame, layout::UiSize};

const WORKBENCH_REFERENCE_WIDTH: u32 = 1672;
const WORKBENCH_REFERENCE_HEIGHT: u32 = 941;
const WORKBENCH_PREVIEW_CAPTURE_ENV: &str = "ZIRCON_WRITE_WORKBENCH_PREVIEW";
const WORKBENCH_PREVIEW_CAPTURE_PATH_ENV: &str = "ZIRCON_WORKBENCH_PREVIEW_PATH";

#[test]
fn componentized_workbench_toolbar_run_menu_paints_native_preview_pixels() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");

    let closed = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );

    bridge
        .dispatch_control_state("WorkbenchRunMode", UiEventKind::Click)
        .expect("run mode menu should dispatch")
        .expect("run mode should expose a menu binding");
    let menu_frame = bridge
        .control_frame("WorkbenchRunModeMenu")
        .expect("opened run mode menu should have a native frame");
    let menu_node = workbench_window_node(&bridge, "WorkbenchRunModeMenu");
    assert_eq!(menu_node.role.as_str(), "Menu");
    assert!(menu_node.popup_open);
    assert_eq!(menu_node.structured_menu_items.row_count(), 4);
    assert_eq!(
        structured_menu_item(&menu_node, 0).label.as_str(),
        "Play In Editor"
    );
    assert_eq!(
        structured_menu_item(&menu_node, 3).label.as_str(),
        "Network Preview"
    );

    let opened = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    maybe_write_workbench_preview_png(&opened);

    let menu_body_frame = frame_below_top_toolbar(menu_frame);
    assert!(
        changed_pixel_count_in_frame(&closed, &opened, menu_frame) > 0,
        "opening the toolbar run menu should repaint pixels inside the menu frame"
    );
    assert!(
        first_non_black_pixel_in_frame(&opened, menu_frame).is_some(),
        "opened toolbar run menu should render visible native pixels"
    );
    assert!(
        changed_pixel_count_in_frame(&closed, &opened, menu_body_frame) > 1_000,
        "opened toolbar run menu should paint a visible menu body below the top toolbar"
    );
}

fn workbench_window_node(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> TemplatePaneNodeData {
    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} should project to native host nodes"))
}

fn structured_menu_item(node: &TemplatePaneNodeData, row: usize) -> TemplatePaneMenuItemData {
    node.structured_menu_items
        .row_data(row)
        .unwrap_or_else(|| panic!("structured menu item row {row} should exist"))
}

fn changed_pixel_count_in_frame(before: &[u8], after: &[u8], frame: UiFrame) -> usize {
    frame_points(frame)
        .filter(|(x, y)| pixel(before, *x, *y) != pixel(after, *x, *y))
        .count()
}

fn first_non_black_pixel_in_frame(bytes: &[u8], frame: UiFrame) -> Option<[u8; 4]> {
    frame_points(frame)
        .map(|(x, y)| pixel(bytes, x, y))
        .find(|pixel| *pixel != [0, 0, 0, 255])
}

fn frame_points(frame: UiFrame) -> impl Iterator<Item = (u32, u32)> {
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

fn frame_below_top_toolbar(frame: UiFrame) -> UiFrame {
    let body_y = frame.y.max(64.0);
    UiFrame::new(
        frame.x,
        body_y,
        frame.width,
        (frame.y + frame.height - body_y).max(0.0),
    )
}

fn pixel(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * WORKBENCH_REFERENCE_WIDTH + x) * 4) as usize;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

fn maybe_write_workbench_preview_png(bytes: &[u8]) {
    if std::env::var_os(WORKBENCH_PREVIEW_CAPTURE_ENV).is_none() {
        return;
    }

    let path = std::env::var_os(WORKBENCH_PREVIEW_CAPTURE_PATH_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("target")
                .join("editor-workbench-visual-check")
                .join("editor-workbench-native-toolbar-run-menu-open-1672x941.png")
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
