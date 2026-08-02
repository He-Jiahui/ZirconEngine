use super::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::retained_host::PaneSurfaceHostContext;
use crate::ui::retained_host::asset_pointer::{
    AssetContentListPointerBridge, AssetContentListPointerLayout, AssetListPointerState,
};
use crate::ui::workbench::asset_content_layout::{
    AssetContentLayoutMetrics, AssetContentSurfaceProfile,
};
use zircon_runtime_interface::ui::layout::UiPoint;

const ASSETS_DRAWER_SCROLLED_HOVER_ARTIFACT: &str =
    "editor-window-m3-assets-drawer-scrolled-hover-900x620.png";
const CONTENT_SCROLL_DELTA: f32 = 480.0;

const ASSETS_DRAWER_RESPONSIVE_ARTIFACTS: [(u32, u32, &str); 3] = [
    (640, 420, "editor-window-m3-assets-drawer-640x420.png"),
    (900, 620, "editor-window-m3-assets-drawer-900x620.png"),
    (1260, 780, "editor-window-m3-assets-drawer-1260x780.png"),
];

#[test]
#[ignore = "writes responsive Assets drawer screenshot artifacts"]
fn capture_assets_drawer_responsive_visual_artifacts() {
    std::env::set_var("SLINT_BACKEND", "software");

    for (width, height, file_name) in ASSETS_DRAWER_RESPONSIVE_ARTIFACTS {
        let window = assets_drawer_window(width, height);
        assert_populated_asset_content(&window);
        if width == 900 {
            assert_assets_drawer_adaptive_layout(&window, width);
        }
        save_window_snapshot(&window, file_name);
    }
}

#[test]
fn assets_drawer_content_scroll_repaints_inside_content_without_touching_utility() {
    std::env::set_var("SLINT_BACKEND", "software");
    let window = assets_drawer_window(900, 620);
    let content_frame = absolute_left_dock_node_frame(&window, "AssetsActivityContentPanel");
    let utility_frame = absolute_left_dock_node_frame(&window, "AssetsActivityUtilityPanel");
    let before = window
        .window()
        .take_snapshot()
        .expect("unscrolled Assets drawer snapshot");

    scroll_asset_content(&window);
    let scrolled = window
        .window()
        .take_snapshot()
        .expect("scrolled Assets drawer snapshot");

    assert!(
        changed_snapshot_pixel_count_in_frame(
            before.as_bytes(),
            scrolled.as_bytes(),
            scrolled.width(),
            scrolled.height(),
            content_frame,
        ) > 200,
        "native content scroll should visibly repaint the asset content viewport"
    );
    assert_eq!(
        changed_snapshot_pixel_count_in_frame(
            before.as_bytes(),
            scrolled.as_bytes(),
            scrolled.width(),
            scrolled.height(),
            utility_frame.clone(),
        ),
        0,
        "content paint projection must remain clipped above the Preview utility surface"
    );

    let visible_rows = visible_activity_asset_content_row_indices(&window);
    let hovered_index = *visible_rows
        .last()
        .expect("scrolled Assets drawer must retain a visible content row");
    assert!(
        hovered_index > 0,
        "native content scroll should reveal a later asset row for hover acceptance"
    );
    let hovered_row = hover_asset_content_row(&window, hovered_index);
    let after_hover = window
        .window()
        .take_snapshot()
        .expect("hovered Assets drawer snapshot");
    let hovered_row_interior = row_interior(hovered_row);
    assert!(
        changed_snapshot_pixel_count_in_frame(
            scrolled.as_bytes(),
            after_hover.as_bytes(),
            after_hover.width(),
            after_hover.height(),
            hovered_row_interior,
        ) > 40,
        "real pointer move should apply the standard hover surface to the final visible item row"
    );
    for other_index in visible_rows
        .into_iter()
        .filter(|index| *index != hovered_index)
    {
        assert_eq!(
            changed_snapshot_pixel_count_in_frame(
                scrolled.as_bytes(),
                after_hover.as_bytes(),
                after_hover.width(),
                after_hover.height(),
                row_interior(projected_asset_content_row_frame(&window, other_index)),
            ),
            0,
            "every non-target content row must retain its original surface"
        );
    }
    assert_eq!(
        changed_snapshot_pixel_count_in_frame(
            scrolled.as_bytes(),
            after_hover.as_bytes(),
            after_hover.width(),
            after_hover.height(),
            utility_frame,
        ),
        0,
        "content hover repaint must not touch the Preview utility surface"
    );
}

#[test]
#[ignore = "writes scrolled and hovered Assets drawer screenshot artifact"]
fn capture_assets_drawer_scrolled_hover_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");
    let window = assets_drawer_window(900, 620);
    apply_scrolled_hover_state(&window);

    save_window_snapshot(&window, ASSETS_DRAWER_SCROLLED_HOVER_ARTIFACT);
}

fn assert_populated_asset_content(window: &UiHostWindow) {
    let presentation = window.get_host_presentation();
    let nodes = &presentation
        .host_scene_data
        .left_dock
        .pane
        .assets_activity
        .nodes;
    let projected = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .collect::<Vec<_>>();
    let first_row = projected
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentItemRow00")
        .expect("responsive Assets drawer should project real asset rows");

    assert!(first_row.frame.width > 0.0 && first_row.frame.height > 0.0);
    assert!(first_row.selected);
    assert!(
        !projected
            .iter()
            .any(|node| node.control_id == "AssetsActivityContentEmptyText")
    );
}

pub(super) fn assets_drawer_window(width: u32, height: u32) -> UiHostWindow {
    let mut fixture = default_preview_fixture();
    let active = ViewInstanceId::new("editor.assets#1");
    if let Some(drawer) = fixture.layout.drawers.get_mut(&ActivityDrawerSlot::LeftTop) {
        if !drawer.tab_stack.tabs.contains(&active) {
            drawer.tab_stack.tabs.push(active.clone());
        }
        drawer.tab_stack.active_tab = Some(active.clone());
        drawer.active_view = Some(active);
        drawer.mode = ActivityDrawerMode::Pinned;
        drawer.visible = true;
    }

    let mut workspace = m3_asset_workspace();
    workspace.view_mode = AssetViewMode::List;
    workspace.visible_folders.clear();
    let mut chrome = fixture.build_chrome();
    chrome.asset_activity = workspace.clone();
    chrome.asset_browser = workspace.clone();
    let window = presented_window_from_chrome(
        chrome,
        &fixture.layout,
        &fixture.descriptors,
        width,
        height,
        &[],
        None,
    );
    wire_activity_asset_content_callbacks(&window, &workspace);
    window
}

fn apply_scrolled_hover_state(window: &UiHostWindow) {
    scroll_asset_content(window);
    let hovered_index = *visible_activity_asset_content_row_indices(window)
        .last()
        .expect("scrolled Assets drawer must retain a visible content row");
    hover_asset_content_row(window, hovered_index);
}

fn visible_activity_asset_content_row_indices(window: &UiHostWindow) -> Vec<i32> {
    let content = absolute_left_dock_node_frame(window, "AssetsActivityContentPanel");
    let content_bottom = content.y + content.height;
    (0..m3_asset_workspace().visible_assets.len())
        .filter_map(|index| i32::try_from(index).ok())
        .filter(|index| {
            let row = projected_asset_content_row_frame(window, *index);
            row.y >= content.y && row.y + row.height <= content_bottom
        })
        .collect()
}

fn wire_activity_asset_content_callbacks(
    window: &UiHostWindow,
    workspace: &AssetWorkspaceSnapshot,
) {
    let content = absolute_left_dock_node_frame(window, "AssetsActivityContentPanel");
    let mut pointer_bridge = AssetContentListPointerBridge::new();
    pointer_bridge.sync(
        AssetContentListPointerLayout::from_snapshot(
            workspace,
            UiSize::new(content.width, content.height),
            AssetContentSurfaceProfile::Activity,
        ),
        AssetListPointerState::default(),
    );
    let pointer_bridge = Rc::new(RefCell::new(pointer_bridge));

    let callback_window = window.clone_strong();
    let move_bridge = Rc::clone(&pointer_bridge);
    window
        .global::<PaneSurfaceHostContext>()
        .on_asset_content_pointer_moved(move |surface_mode, x, y, _width, _height| {
            assert_eq!(surface_mode.as_str(), "activity");
            let dispatch = move_bridge
                .borrow_mut()
                .handle_move(UiPoint::new(x, y))
                .expect("Activity content move should dispatch through the shared bridge");
            write_activity_asset_content_state(&callback_window, &dispatch.state);
        });

    let callback_window = window.clone_strong();
    window
        .global::<PaneSurfaceHostContext>()
        .on_asset_content_pointer_scrolled(move |surface_mode, x, y, delta, _width, _height| {
            assert_eq!(surface_mode.as_str(), "activity");
            let dispatch = pointer_bridge
                .borrow_mut()
                .handle_scroll(UiPoint::new(x, y), delta)
                .expect("Activity content scroll should dispatch through the shared bridge");
            write_activity_asset_content_state(&callback_window, &dispatch.state);
        });
}

fn write_activity_asset_content_state(window: &UiHostWindow, state: &AssetListPointerState) {
    let pane = window.global::<PaneSurfaceHostContext>();
    pane.set_activity_asset_content_scroll_px(state.scroll_offset);
    pane.set_activity_asset_content_hovered_index(
        state
            .hovered_row_index
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1),
    );
}

fn scroll_asset_content(window: &UiHostWindow) {
    let content = absolute_left_dock_node_frame(window, "AssetsActivityContentPanel");
    let result = window.dispatch_native_pointer_scroll_for_test(
        content.x + 4.0,
        content.y + 4.0,
        CONTENT_SCROLL_DELTA,
    );
    assert_eq!(
        result.damage_region(),
        Some(content),
        "native Activity content scroll must route through the content panel callback"
    );
    assert!(
        window
            .get_pane_interaction_state()
            .activity_asset_content_scroll_px
            > 0.0,
        "shared Activity content callback must write a positive scroll offset"
    );
}

fn hover_asset_content_row(window: &UiHostWindow, row_index: i32) -> FrameRect {
    let row = projected_asset_content_row_frame(window, row_index);
    let content = absolute_left_dock_node_frame(window, "AssetsActivityContentPanel");
    let result = window
        .dispatch_native_pointer_move_for_test(row.x + row.width * 0.5, row.y + row.height * 0.5);
    let interaction = window.get_pane_interaction_state();
    assert_eq!(
        interaction.activity_asset_content_hovered_index,
        row_index,
        "native Activity content move must write the shared hovered row index; template_control=`{}`, scroll={}, row=({}, {}, {}, {}), content=({}, {}, {}, {})",
        interaction.hovered_template_control_id,
        interaction.activity_asset_content_scroll_px,
        row.x,
        row.y,
        row.width,
        row.height,
        content.x,
        content.y,
        content.width,
        content.height,
    );
    let damage = result
        .damage_region()
        .expect("native Activity content hover should request regional repaint");
    assert!(
        frames_intersect(&damage, &row),
        "native Activity content hover damage must cover the pointer-targeted row"
    );
    row
}

fn frames_intersect(a: &FrameRect, b: &FrameRect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn projected_asset_content_row_frame(window: &UiHostWindow, row_index: i32) -> FrameRect {
    let content = absolute_left_dock_node_frame(window, "AssetsActivityContentPanel");
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        AssetViewMode::List,
    );
    let scroll_px = window
        .get_pane_interaction_state()
        .activity_asset_content_scroll_px;
    FrameRect {
        x: content.x + metrics.row_x,
        y: content.y
            + metrics.first_row_y()
            + row_index as f32 * (metrics.item_height + metrics.row_gap)
            - scroll_px,
        width: metrics.row_width(content.width),
        height: metrics.item_height,
    }
}

fn row_interior(row: FrameRect) -> FrameRect {
    FrameRect {
        x: row.x + 2.0,
        y: row.y + 2.0,
        width: (row.width - 16.0).max(0.0),
        height: (row.height - 4.0).max(0.0),
    }
}

fn absolute_left_dock_node_frame(window: &UiHostWindow, control_id: &str) -> FrameRect {
    let presentation = window.get_host_presentation();
    let dock = &presentation.host_scene_data.left_dock;
    let panel_x = if dock.rail_before_panel {
        dock.region_frame.x + dock.rail_width_px
    } else {
        dock.region_frame.x
    };
    let body_x = panel_x + dock.content_frame.x;
    let body_y = dock.region_frame.y + dock.content_frame.y;
    let node = (0..dock.pane.assets_activity.nodes.row_count())
        .filter_map(|row| dock.pane.assets_activity.nodes.row_data(row))
        .find(|node| node.control_id == control_id)
        .unwrap_or_else(|| panic!("missing Assets drawer node `{control_id}`"));
    FrameRect {
        x: body_x + node.frame.x,
        y: body_y + node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}
