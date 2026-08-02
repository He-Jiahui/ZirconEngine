use crate::ui::layouts::views::asset_browser_pane_nodes;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, template_node_command_summary_for_test,
};
use crate::ui::workbench::snapshot::{AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::ui::layout::UiSize;

use super::template_node_conversion::to_host_contract_template_nodes;

const NARROW_ASSET_BROWSER_WIDTH: u32 = 420;
const NARROW_ASSET_BROWSER_HEIGHT: u32 = 360;
const NARROW_ASSET_BROWSER_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn narrow_asset_browser_projection_paints_direct_icon_actions() {
    let view_nodes = asset_browser_pane_nodes(
        &AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            ..AssetWorkspaceSnapshot::default()
        },
        UiSize::new(
            NARROW_ASSET_BROWSER_WIDTH as f32,
            NARROW_ASSET_BROWSER_HEIGHT as f32,
        ),
    );
    let host_nodes = to_host_contract_template_nodes(&view_nodes);
    let actions = host_nodes
        .iter()
        .filter(|node| {
            matches!(
                node.control_id.as_str(),
                "AssetBrowserViewModeListButton"
                    | "AssetBrowserViewModeThumbButton"
                    | "LocateSelectedAsset"
            )
        })
        .cloned()
        .collect::<Vec<_>>();

    assert_eq!(
        actions.len(),
        3,
        "all direct asset actions must be projected"
    );
    for (control_id, icon_name, selected) in [
        ("AssetBrowserViewModeListButton", "list-outline", false),
        ("AssetBrowserViewModeThumbButton", "grid-outline", true),
        (
            "LocateSelectedAsset",
            "editor_pages/asset_browser/navigation/search.svg",
            false,
        ),
    ] {
        assert!(
            actions.iter().any(|node| {
                node.control_id.as_str() == control_id
                    && node.role.as_str() == "IconButton"
                    && node.icon_name.as_str() == icon_name
                    && node.text.is_empty()
                    && node.frame.width == 30.0
                    && node.frame.height == 30.0
                    && node.selected == selected
            }),
            "{control_id} must preserve its icon-only production projection"
        );
    }
    let list = action_by_id(&actions, "AssetBrowserViewModeListButton");
    let thumbnail = action_by_id(&actions, "AssetBrowserViewModeThumbButton");
    let locate = action_by_id(&actions, "LocateSelectedAsset");
    assert!(
        list.frame.x + list.frame.width <= thumbnail.frame.x
            && thumbnail.frame.x + thumbnail.frame.width <= locate.frame.x,
        "direct asset actions must remain non-overlapping at narrow width"
    );
    for action in &actions {
        let command_summary = template_node_command_summary_for_test(action);
        assert_eq!(
            command_summary.text_count, 0,
            "{} must not emit a duplicate text label",
            action.control_id
        );
        assert_eq!(
            command_summary.image_frames.len(),
            1,
            "{} must render its resolved SVG icon asset",
            action.control_id
        );
    }

    let bytes = paint_template_nodes_for_test_with_background(
        NARROW_ASSET_BROWSER_WIDTH,
        NARROW_ASSET_BROWSER_HEIGHT,
        NARROW_ASSET_BROWSER_BACKGROUND,
        host_nodes,
    );
    for action in actions {
        assert!(
            has_non_background_pixel(
                &bytes,
                action.frame.x,
                action.frame.y,
                action.frame.width,
                action.frame.height
            ),
            "{} must reach the retained painter at its projected frame",
            action.control_id
        );
    }
}

fn action_by_id<'a>(
    actions: &'a [crate::ui::retained_host::TemplatePaneNodeData],
    control_id: &str,
) -> &'a crate::ui::retained_host::TemplatePaneNodeData {
    actions
        .iter()
        .find(|action| action.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("expected projected action {control_id}"))
}

fn has_non_background_pixel(bytes: &[u8], x: f32, y: f32, width: f32, height: f32) -> bool {
    let x_start = x.max(0.0).floor() as u32;
    let y_start = y.max(0.0).floor() as u32;
    let x_end = (x + width).min(NARROW_ASSET_BROWSER_WIDTH as f32).ceil() as u32;
    let y_end = (y + height).min(NARROW_ASSET_BROWSER_HEIGHT as f32).ceil() as u32;

    (y_start..y_end).any(|pixel_y| {
        (x_start..x_end).any(|pixel_x| {
            let index = ((pixel_y * NARROW_ASSET_BROWSER_WIDTH + pixel_x) * 4) as usize;
            [
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            ] != NARROW_ASSET_BROWSER_BACKGROUND
        })
    })
}
