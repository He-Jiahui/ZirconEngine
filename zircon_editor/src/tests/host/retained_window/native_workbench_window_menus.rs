use std::path::PathBuf;
use std::rc::Rc;

use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::retained_host::primitives::{ModelRc, PhysicalSize, VecModel};
use crate::ui::retained_host::{
    paint_runtime_render_commands_for_test, to_host_contract_workbench_window_nodes, FrameRect,
    HostChromeControlFrameData, HostDocumentDockSurfaceData, HostMenuChromeData,
    HostMenuChromeItemData, HostMenuChromeMenuData, HostMenuStateData, HostWindowLayoutData,
    PaneData, TemplatePaneMenuItemData, TemplatePaneNodeData, UiHostContext, UiHostWindow,
};
use zircon_runtime_interface::ui::design_tokens::EditorPaletteTokens;
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiFrame, layout::UiSize};

const WORKBENCH_REFERENCE_WIDTH: u32 = 1672;
const WORKBENCH_REFERENCE_HEIGHT: u32 = 941;
const WORKBENCH_PREVIEW_CAPTURE_ENV: &str = "ZIRCON_WRITE_WORKBENCH_PREVIEW";
const WORKBENCH_PREVIEW_CAPTURE_PATH_ENV: &str = "ZIRCON_WORKBENCH_PREVIEW_PATH";
const POPUP_BORDER: [u8; 4] = EditorPaletteTokens::WORKBENCH_BORDER;
const POPUP_FOCUS_RING: [u8; 4] = EditorPaletteTokens::WORKBENCH_FOCUS_RING;
const POPUP_PRIMARY_TEXT: [u8; 4] = EditorPaletteTokens::WORKBENCH_TEXT_PRIMARY;

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
    let menu_frame = rendered_control_frame(&bridge, "WorkbenchRunModeMenu");
    let menu_layout_height = bridge
        .surface()
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("WorkbenchRunModeMenu")
        })
        .map(|node| node.layout_cache.frame.height)
        .expect("run mode menu should have a layout frame");
    assert!((menu_layout_height - 130.0).abs() < f32::EPSILON);
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

#[test]
fn rust_owned_window_menu_popup_uses_muted_border_and_primary_item_text() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.menu_chrome = HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: model_rc(vec![control_frame("MenuSlot0", 8.0, 2.0, 56.0, 22.0)]),
        menus: model_rc(vec![HostMenuChromeMenuData {
            label: "File".into(),
            popup_width_px: 144.0,
            popup_height_px: 66.0,
            items: model_rc(vec![
                HostMenuChromeItemData {
                    label: "Open".into(),
                    action_id: "workbench.project.open".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                },
                HostMenuChromeItemData {
                    label: "Reset Layout".into(),
                    action_id: "workbench.layout.reset".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                },
            ]),
            ..HostMenuChromeMenuData::default()
        }]),
        ..HostMenuChromeData::default()
    };
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(0.0, 26.0, 360.0, 170.0),
        header_frame: host_frame(0.0, 0.0, 360.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 360.0, 137.0),
        pane: PaneData {
            kind: "Scene".into(),
            title: "Scene".into(),
            ..PaneData::default()
        },
        ..HostDocumentDockSurfaceData::default()
    };
    presentation.menu_state = HostMenuStateData {
        open_menu_index: 0,
        ..HostMenuStateData::default()
    };
    let menu_state = presentation.menu_state.clone();
    ui.set_host_presentation(presentation);
    ui.global::<UiHostContext>().set_menu_state(menu_state);

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("open menu snapshot should render");
    let bytes = snapshot.as_bytes();

    assert_eq!(
        snapshot_pixel(snapshot.width(), bytes, 20, 27),
        POPUP_BORDER,
        "window menu popup should use the neutral 1px border color"
    );
    assert_ne!(
        snapshot_pixel(snapshot.width(), bytes, 20, 27),
        POPUP_FOCUS_RING,
        "window menu popup border should not reuse the accent focus color"
    );
    assert!(
        contains_snapshot_pixel_near(
            snapshot.width(),
            bytes,
            22,
            37,
            64,
            16,
            POPUP_PRIMARY_TEXT,
            24
        ),
        "enabled window menu item labels should use primary text, not muted menu chrome text"
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

fn rendered_control_frame(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> UiFrame {
    let node_id = bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                .filter(|candidate| *candidate == control_id)
                .map(|_| node.node_id)
        })
        .unwrap_or_else(|| panic!("{control_id} should resolve to one runtime node"));
    bridge
        .surface()
        .render_extract
        .list
        .commands
        .iter()
        .filter(|command| command.node_id == node_id)
        .map(|command| command.frame)
        .max_by(|left, right| {
            let left_area = left.width.max(0.0) * left.height.max(0.0);
            let right_area = right.width.max(0.0) * right.height.max(0.0);
            left_area.total_cmp(&right_area)
        })
        .unwrap_or_else(|| panic!("{control_id} should emit popup render commands"))
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

fn host_window_layout_for_test(width: f32, height: f32) -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: host_frame(0.0, 58.0, width, height - 82.0),
        status_bar_frame: host_frame(0.0, height - 24.0, width, 24.0),
        left_region_frame: FrameRect::default(),
        document_region_frame: host_frame(60.0, 58.0, width - 80.0, height - 82.0),
        viewport_content_frame: host_frame(60.0, 118.0, width - 80.0, height - 142.0),
        ..HostWindowLayoutData::default()
    }
}

fn host_frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

fn control_frame(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> HostChromeControlFrameData {
    HostChromeControlFrameData {
        control_id: control_id.into(),
        frame: host_frame(x, y, width, height),
    }
}

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

fn snapshot_pixel(width: u32, bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

fn contains_snapshot_pixel_near(
    width: u32,
    bytes: &[u8],
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
    expected: [u8; 4],
    tolerance: u8,
) -> bool {
    let height = (bytes.len() / 4 / width as usize) as u32;
    let x1 = x.saturating_add(region_width).min(width);
    let y1 = y.saturating_add(region_height).min(height);
    (y..y1).any(|row| {
        (x..x1).any(|column| {
            color_near(
                snapshot_pixel(width, bytes, column, row),
                expected,
                tolerance,
            )
        })
    })
}

fn color_near(actual: [u8; 4], expected: [u8; 4], tolerance: u8) -> bool {
    actual
        .iter()
        .zip(expected.iter())
        .all(|(left, right)| left.abs_diff(*right) <= tolerance)
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
            PathBuf::from("docs")
                .join("tests")
                .join("editor")
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
