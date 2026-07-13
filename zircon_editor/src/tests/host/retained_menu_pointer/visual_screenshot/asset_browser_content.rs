use super::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::retained_host::asset_pointer::{
    AssetContentListPointerBridge, AssetContentListPointerLayout, AssetListPointerState,
};
use crate::ui::retained_host::PaneSurfaceHostContext;
use crate::ui::workbench::asset_content_layout::{
    AssetContentLayoutMetrics, AssetContentSurfaceProfile, AssetThumbnailGridMetrics,
};
use zircon_runtime_interface::ui::layout::UiPoint;

const ASSET_BROWSER_LIST_SCROLLED_HOVER_ARTIFACT: &str =
    "editor-window-m3-asset-browser-list-scrolled-hover-900x620.png";
const ASSET_BROWSER_THUMBNAIL_SCROLLED_HOVER_ARTIFACT: &str =
    "editor-window-m3-asset-browser-thumbnail-scrolled-hover-900x620.png";
const CONTENT_SCROLL_DELTA: f32 = 112.0;

#[test]
fn asset_browser_list_scroll_repaints_rows_without_moving_header_or_preview() {
    std::env::set_var("SLINT_BACKEND", "software");
    let workspace = browser_list_workspace();
    let window = asset_browser_window_with_workspace(900, 620, workspace.clone());
    wire_browser_asset_content_callbacks(&window, &workspace);
    let table = absolute_document_node_frame(&window, "AssetBrowserAssetTablePanel");
    let header = absolute_document_node_frame(&window, "WorkbenchAssetBrowserTableHeader");
    let preview = absolute_document_node_frame(&window, "AssetBrowserContentPreviewCard");
    let before = window
        .window()
        .take_snapshot()
        .expect("unscrolled Asset Browser snapshot");

    scroll_browser_content(&window, &table, &header);
    let scrolled = window
        .window()
        .take_snapshot()
        .expect("scrolled Asset Browser snapshot");

    assert!(
        changed_snapshot_pixel_count_in_frame(
            before.as_bytes(),
            scrolled.as_bytes(),
            scrolled.width(),
            scrolled.height(),
            rows_viewport(&table, &header),
        ) > 200,
        "Browser content scroll should repaint the clipped table rows"
    );
    for (label, fixed) in [("header", header.clone()), ("preview", preview.clone())] {
        let changed = changed_snapshot_pixel_count_in_frame(
            before.as_bytes(),
            scrolled.as_bytes(),
            scrolled.width(),
            scrolled.height(),
            fixed.clone(),
        );
        assert_eq!(
            changed, 0,
            "Browser scroll must not repaint fixed {label} content: changed={changed}, fixed={fixed:?}, table={table:?}"
        );
    }

    let visible_rows = visible_browser_row_indices(&window, workspace.visible_assets.len());
    let hovered_index = *visible_rows
        .last()
        .expect("scrolled Browser table should keep a visible row");
    assert!(hovered_index > 0);
    let hovered_row = hover_browser_row(&window, hovered_index);
    let after_hover = window
        .window()
        .take_snapshot()
        .expect("hovered Asset Browser snapshot");
    assert!(
        changed_snapshot_pixel_count_in_frame(
            scrolled.as_bytes(),
            after_hover.as_bytes(),
            after_hover.width(),
            after_hover.height(),
            row_interior(hovered_row),
        ) > 40,
        "Browser pointer move should apply the standard hover surface to one row"
    );
    for index in visible_rows
        .into_iter()
        .filter(|index| *index != hovered_index)
    {
        assert_eq!(
            changed_snapshot_pixel_count_in_frame(
                scrolled.as_bytes(),
                after_hover.as_bytes(),
                after_hover.width(),
                after_hover.height(),
                row_interior(projected_browser_row_frame(&window, index)),
            ),
            0,
            "non-target Browser rows must remain visually idle"
        );
    }
    assert_eq!(
        changed_snapshot_pixel_count_in_frame(
            scrolled.as_bytes(),
            after_hover.as_bytes(),
            after_hover.width(),
            after_hover.height(),
            preview,
        ),
        0,
        "Browser row hover must stay outside the preview surface"
    );
}

#[test]
#[ignore = "writes the scrolled Asset Browser list screenshot artifact"]
fn capture_asset_browser_list_scrolled_hover_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");
    let workspace = browser_list_workspace();
    let window = asset_browser_window_with_workspace(900, 620, workspace.clone());
    wire_browser_asset_content_callbacks(&window, &workspace);
    let table = absolute_document_node_frame(&window, "AssetBrowserAssetTablePanel");
    let header = absolute_document_node_frame(&window, "WorkbenchAssetBrowserTableHeader");
    scroll_browser_content(&window, &table, &header);
    let hovered_index = *visible_browser_row_indices(&window, workspace.visible_assets.len())
        .last()
        .expect("scrolled Browser table should keep a visible row");
    hover_browser_row(&window, hovered_index);

    save_window_snapshot(&window, ASSET_BROWSER_LIST_SCROLLED_HOVER_ARTIFACT);
}

#[test]
fn asset_browser_thumbnail_scroll_repaints_grid_without_moving_fixed_controls() {
    std::env::set_var("SLINT_BACKEND", "software");
    let workspace = browser_thumbnail_workspace();
    let window = asset_browser_window_with_workspace(900, 620, workspace.clone());
    wire_browser_asset_content_callbacks(&window, &workspace);
    let grid = absolute_document_node_frame(&window, "AssetBrowserThumbGridPanel");
    let header = absolute_document_node_frame(&window, "AssetBrowserContentHeaderRow");
    let utility = absolute_document_node_frame(&window, "AssetBrowserUtilityTabsRow");
    let before = window
        .window()
        .take_snapshot()
        .expect("unscrolled Asset Browser thumbnail snapshot");

    scroll_browser_thumbnail_content(&window, &grid);
    let scrolled = window
        .window()
        .take_snapshot()
        .expect("scrolled Asset Browser thumbnail snapshot");

    assert!(
        changed_snapshot_pixel_count_in_frame(
            before.as_bytes(),
            scrolled.as_bytes(),
            scrolled.width(),
            scrolled.height(),
            grid.clone(),
        ) > 200,
        "Browser thumbnail scroll should repaint the clipped grid"
    );
    for (label, fixed) in [("content header", header), ("utility tabs", utility)] {
        assert_eq!(
            changed_snapshot_pixel_count_in_frame(
                before.as_bytes(),
                scrolled.as_bytes(),
                scrolled.width(),
                scrolled.height(),
                fixed,
            ),
            0,
            "Browser thumbnail scroll must keep {label} fixed"
        );
    }

    let visible = visible_browser_thumbnail_indices(&window, workspace.visible_assets.len());
    let hovered_index = *visible
        .last()
        .expect("scrolled Browser thumbnail grid should keep a visible card");
    assert!(hovered_index > 0);
    let hovered_card = hover_browser_thumbnail(&window, hovered_index);
    let after_hover = window
        .window()
        .take_snapshot()
        .expect("hovered Asset Browser thumbnail snapshot");
    assert!(
        changed_snapshot_pixel_count_in_frame(
            scrolled.as_bytes(),
            after_hover.as_bytes(),
            after_hover.width(),
            after_hover.height(),
            thumbnail_card_interior(hovered_card),
        ) > 40,
        "Browser pointer move should apply hover only to the target thumbnail"
    );
    for index in visible.into_iter().filter(|index| *index != hovered_index) {
        assert_eq!(
            changed_snapshot_pixel_count_in_frame(
                scrolled.as_bytes(),
                after_hover.as_bytes(),
                after_hover.width(),
                after_hover.height(),
                thumbnail_card_interior(projected_browser_thumbnail_frame(&window, index)),
            ),
            0,
            "non-target Browser thumbnails must remain visually idle"
        );
    }
}

#[test]
#[ignore = "writes the scrolled Asset Browser thumbnail screenshot artifact"]
fn capture_asset_browser_thumbnail_scrolled_hover_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");
    let workspace = browser_thumbnail_workspace();
    let window = asset_browser_window_with_workspace(900, 620, workspace.clone());
    wire_browser_asset_content_callbacks(&window, &workspace);
    let grid = absolute_document_node_frame(&window, "AssetBrowserThumbGridPanel");
    scroll_browser_thumbnail_content(&window, &grid);
    let hovered_index = *visible_browser_thumbnail_indices(&window, workspace.visible_assets.len())
        .last()
        .expect("scrolled Browser thumbnail grid should keep a visible card");
    hover_browser_thumbnail(&window, hovered_index);

    save_window_snapshot(&window, ASSET_BROWSER_THUMBNAIL_SCROLLED_HOVER_ARTIFACT);
}

fn browser_list_workspace() -> AssetWorkspaceSnapshot {
    let mut workspace = m3_asset_workspace();
    workspace.view_mode = AssetViewMode::List;
    workspace.visible_folders.clear();
    let base_assets = workspace.visible_assets.clone();
    for copy in 1..3 {
        for asset in &base_assets {
            let mut repeated = asset.clone();
            repeated.uuid = format!("{}-copy-{copy}", asset.uuid);
            repeated.locator = format!("{}?copy={copy}", asset.locator);
            repeated.display_name = format!("copy_{copy}_{}", asset.display_name);
            repeated.file_name = repeated.display_name.clone();
            repeated.selected = false;
            workspace.visible_assets.push(repeated);
        }
    }
    workspace
}

fn browser_thumbnail_workspace() -> AssetWorkspaceSnapshot {
    let mut workspace = browser_list_workspace();
    workspace.view_mode = AssetViewMode::Thumbnail;
    workspace
}

fn wire_browser_asset_content_callbacks(window: &UiHostWindow, workspace: &AssetWorkspaceSnapshot) {
    let surface = absolute_document_node_frame(
        window,
        match workspace.view_mode {
            AssetViewMode::List => "AssetBrowserAssetTablePanel",
            AssetViewMode::Thumbnail => "AssetBrowserThumbGridPanel",
        },
    );
    let mut pointer_bridge = AssetContentListPointerBridge::new();
    pointer_bridge.sync(
        AssetContentListPointerLayout::from_snapshot(
            workspace,
            UiSize::new(surface.width, surface.height),
            AssetContentSurfaceProfile::Browser,
        ),
        AssetListPointerState::default(),
    );
    let pointer_bridge = Rc::new(RefCell::new(pointer_bridge));

    let callback_window = window.clone_strong();
    let move_bridge = Rc::clone(&pointer_bridge);
    window
        .global::<PaneSurfaceHostContext>()
        .on_asset_content_pointer_moved(move |surface_mode, x, y, _width, _height| {
            assert_eq!(surface_mode.as_str(), "browser");
            let dispatch = move_bridge
                .borrow_mut()
                .handle_move(UiPoint::new(x, y))
                .expect("Browser content move should dispatch through the shared bridge");
            write_browser_asset_content_state(&callback_window, &dispatch.state);
        });

    let callback_window = window.clone_strong();
    window
        .global::<PaneSurfaceHostContext>()
        .on_asset_content_pointer_scrolled(move |surface_mode, x, y, delta, _width, _height| {
            assert_eq!(surface_mode.as_str(), "browser");
            let dispatch = pointer_bridge
                .borrow_mut()
                .handle_scroll(UiPoint::new(x, y), delta)
                .expect("Browser content scroll should dispatch through the shared bridge");
            write_browser_asset_content_state(&callback_window, &dispatch.state);
        });
}

fn scroll_browser_thumbnail_content(window: &UiHostWindow, grid: &FrameRect) {
    let result = window.dispatch_native_pointer_scroll_for_test(
        grid.x + grid.width * 0.5,
        grid.y + grid.height * 0.5,
        CONTENT_SCROLL_DELTA,
    );
    assert_eq!(result.damage_region(), Some(grid.clone()));
    assert!(
        window
            .get_pane_interaction_state()
            .browser_asset_content_scroll_px
            > 0.0
    );
}

fn visible_browser_thumbnail_indices(window: &UiHostWindow, count: usize) -> Vec<i32> {
    let grid = absolute_document_node_frame(window, "AssetBrowserThumbGridPanel");
    (0..count)
        .filter_map(|index| i32::try_from(index).ok())
        .filter(|index| {
            let card = projected_browser_thumbnail_frame(window, *index);
            card.y >= grid.y && card.y + card.height <= grid.y + grid.height
        })
        .collect()
}

fn hover_browser_thumbnail(window: &UiHostWindow, index: i32) -> FrameRect {
    let card = projected_browser_thumbnail_frame(window, index);
    window.dispatch_native_pointer_move_for_test(
        card.x + card.width * 0.5,
        card.y + card.height * 0.5,
    );
    assert_eq!(
        window
            .get_pane_interaction_state()
            .browser_asset_content_hovered_index,
        index
    );
    card
}

fn projected_browser_thumbnail_frame(window: &UiHostWindow, index: i32) -> FrameRect {
    let grid = absolute_document_node_frame(window, "AssetBrowserThumbGridPanel");
    let count = browser_thumbnail_count(window);
    let metrics = AssetThumbnailGridMetrics::new(grid.width, count);
    let item = metrics
        .item_frame(index as usize)
        .expect("thumbnail index should have a grid frame");
    let scroll_px = window
        .get_pane_interaction_state()
        .browser_asset_content_scroll_px;
    FrameRect {
        x: grid.x + item.x,
        y: grid.y + item.y - scroll_px,
        width: item.width,
        height: item.height,
    }
}

fn browser_thumbnail_count(window: &UiHostWindow) -> usize {
    let nodes = &window
        .get_host_presentation()
        .host_scene_data
        .document_dock
        .pane
        .asset_browser
        .nodes;
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .filter(|node| {
            node.control_id
                .as_str()
                .strip_prefix("AssetBrowserThumbCard")
                .is_some_and(|index| index.bytes().all(|byte| byte.is_ascii_digit()))
        })
        .count()
}

fn thumbnail_card_interior(card: FrameRect) -> FrameRect {
    FrameRect {
        x: card.x + 2.0,
        y: card.y + 2.0,
        width: (card.width - 4.0).max(0.0),
        height: (card.height - 4.0).max(0.0),
    }
}

fn write_browser_asset_content_state(window: &UiHostWindow, state: &AssetListPointerState) {
    let pane = window.global::<PaneSurfaceHostContext>();
    pane.set_browser_asset_content_scroll_px(state.scroll_offset);
    pane.set_browser_asset_content_hovered_index(
        state
            .hovered_row_index
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1),
    );
}

fn scroll_browser_content(window: &UiHostWindow, table: &FrameRect, header: &FrameRect) {
    let result = window.dispatch_native_pointer_scroll_for_test(
        table.x + 4.0,
        header.y + header.height + 4.0,
        CONTENT_SCROLL_DELTA,
    );
    assert_eq!(result.damage_region(), Some(table.clone()));
    assert!(
        window
            .get_pane_interaction_state()
            .browser_asset_content_scroll_px
            > 0.0
    );
}

fn visible_browser_row_indices(window: &UiHostWindow, count: usize) -> Vec<i32> {
    let table = absolute_document_node_frame(window, "AssetBrowserAssetTablePanel");
    let header = absolute_document_node_frame(window, "WorkbenchAssetBrowserTableHeader");
    let viewport = rows_viewport(&table, &header);
    (0..count)
        .filter_map(|index| i32::try_from(index).ok())
        .filter(|index| {
            let row = projected_browser_row_frame(window, *index);
            row.y >= viewport.y && row.y + row.height <= viewport.y + viewport.height
        })
        .collect()
}

fn hover_browser_row(window: &UiHostWindow, row_index: i32) -> FrameRect {
    let row = projected_browser_row_frame(window, row_index);
    window.dispatch_native_pointer_move_for_test(row.x + row.width * 0.5, row.y + row.height * 0.5);
    assert_eq!(
        window
            .get_pane_interaction_state()
            .browser_asset_content_hovered_index,
        row_index
    );
    row
}

fn projected_browser_row_frame(window: &UiHostWindow, row_index: i32) -> FrameRect {
    let table = absolute_document_node_frame(window, "AssetBrowserAssetTablePanel");
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Browser,
        AssetViewMode::List,
    );
    let scroll_px = window
        .get_pane_interaction_state()
        .browser_asset_content_scroll_px;
    FrameRect {
        x: table.x + metrics.row_x,
        y: table.y
            + metrics.first_row_y()
            + row_index as f32 * (metrics.item_height + metrics.row_gap)
            - scroll_px,
        width: metrics.row_width(table.width),
        height: metrics.item_height,
    }
}

fn rows_viewport(table: &FrameRect, header: &FrameRect) -> FrameRect {
    FrameRect {
        x: table.x,
        y: header.y + header.height,
        width: table.width,
        height: (table.y + table.height - header.y - header.height).max(0.0),
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

fn absolute_document_node_frame(window: &UiHostWindow, control_id: &str) -> FrameRect {
    let presentation = window.get_host_presentation();
    let dock = &presentation.host_scene_data.document_dock;
    let body_x = dock.region_frame.x + dock.content_frame.x;
    let body_y = dock.region_frame.y + dock.content_frame.y;
    let node = (0..dock.pane.asset_browser.nodes.row_count())
        .filter_map(|row| dock.pane.asset_browser.nodes.row_data(row))
        .find(|node| node.control_id == control_id)
        .unwrap_or_else(|| panic!("missing Asset Browser node `{control_id}`"));
    FrameRect {
        x: body_x + node.frame.x,
        y: body_y + node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}
