use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;

use super::identity::{
    activity_content_identity, ActivityContentNodeIdentity, ActivityContentNodeRole,
};
use super::ActivityAssetContentProjector;

#[test]
fn activity_asset_content_identity_maps_rows_and_children_without_aliases() {
    assert_eq!(
        activity_content_identity("AssetsActivityContentPanel"),
        Some(ActivityContentNodeIdentity::ContentPanel)
    );
    assert_eq!(
        activity_content_identity("AssetsActivityContentFolderRow02"),
        Some(ActivityContentNodeIdentity::Folder {
            index: 2,
            role: ActivityContentNodeRole::Row,
        })
    );
    assert_eq!(
        activity_content_identity("AssetsActivityContentFolderName02"),
        Some(ActivityContentNodeIdentity::Folder {
            index: 2,
            role: ActivityContentNodeRole::Name,
        })
    );
    assert_eq!(
        activity_content_identity("mount/AssetsActivityContentItemMeta11"),
        Some(ActivityContentNodeIdentity::Item {
            index: 11,
            role: ActivityContentNodeRole::Meta,
        })
    );
    assert_eq!(
        activity_content_identity("AssetsActivityPreviewPanel"),
        None
    );
    assert_eq!(
        activity_content_identity("AssetsActivityContentItemRow"),
        None
    );
}

#[test]
fn activity_asset_content_projector_scrolls_clips_and_hovers_shared_row_index() {
    let nodes = model_rc(vec![
        node("AssetsActivityContentPanel", 10.0, 20.0, 100.0, 40.0),
        node("AssetsActivityContentFolderRow00", 12.0, 20.0, 96.0, 12.0),
        node("AssetsActivityContentItemRow00", 12.0, 60.0, 96.0, 12.0),
        node("AssetsActivityContentItemName00", 20.0, 62.0, 60.0, 8.0),
        node("AssetsActivityPreviewPanel", 10.0, 70.0, 100.0, 20.0),
    ]);
    let interaction = HostPaneInteractionStateData {
        activity_asset_content_scroll_px: 30.0,
        activity_asset_content_hovered_index: 1,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        ActivityAssetContentProjector::new(&nodes, &frame(5.0, 7.0, 120.0, 100.0), &interaction)
            .expect("content panel projector");
    let pane_clip = frame(0.0, 0.0, 200.0, 200.0);

    assert!(projector
        .transform(nodes.row_data(1).expect("folder row"), pane_clip.clone())
        .is_none());
    let (item, item_clip) = projector
        .transform(nodes.row_data(2).expect("item row"), pane_clip.clone())
        .expect("scrolled item row should enter content viewport");
    assert_eq!(item.frame.y, 30.0);
    assert!(item.hovered);
    assert_eq!(item_clip, frame(15.0, 27.0, 100.0, 40.0));

    let (name, name_clip) = projector
        .transform(nodes.row_data(3).expect("item name"), pane_clip.clone())
        .expect("item child should share projection and clip");
    assert_eq!(name.frame.y, 32.0);
    assert!(!name.hovered);
    assert_eq!(name_clip, item_clip);

    let (preview, preview_clip) = projector
        .transform(nodes.row_data(4).expect("preview"), pane_clip.clone())
        .expect("unrelated utility nodes pass through");
    assert_eq!(preview.frame.y, 70.0);
    assert_eq!(preview_clip, pane_clip);
}

#[test]
fn activity_asset_content_projector_falls_back_without_panel_and_ignores_stale_hover() {
    let no_panel = model_rc(vec![node(
        "AssetsActivityContentItemRow00",
        0.0,
        0.0,
        20.0,
        10.0,
    )]);
    assert!(ActivityAssetContentProjector::new(
        &no_panel,
        &frame(0.0, 0.0, 40.0, 40.0),
        &HostPaneInteractionStateData::default(),
    )
    .is_none());

    let nodes = model_rc(vec![
        node("AssetsActivityContentPanel", 0.0, 0.0, 40.0, 40.0),
        node("AssetsActivityContentItemRow00", 0.0, 0.0, 40.0, 10.0),
    ]);
    let interaction = HostPaneInteractionStateData {
        activity_asset_content_hovered_index: 99,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        ActivityAssetContentProjector::new(&nodes, &frame(0.0, 0.0, 40.0, 40.0), &interaction)
            .expect("projector");
    let (row, _) = projector
        .transform(
            nodes.row_data(1).expect("item row"),
            frame(0.0, 0.0, 40.0, 40.0),
        )
        .expect("visible row");
    assert!(!row.hovered);
}

#[test]
fn activity_asset_content_projector_keeps_empty_state_visible_with_stale_scroll() {
    let nodes = model_rc(vec![
        node("AssetsActivityContentPanel", 0.0, 0.0, 80.0, 40.0),
        node("AssetsActivityContentEmptyText", 4.0, 4.0, 72.0, 12.0),
    ]);
    let interaction = HostPaneInteractionStateData {
        activity_asset_content_scroll_px: 120.0,
        activity_asset_content_hovered_index: 3,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        ActivityAssetContentProjector::new(&nodes, &frame(10.0, 20.0, 80.0, 40.0), &interaction)
            .expect("empty content projector");

    let (empty, clip) = projector
        .transform(
            nodes.row_data(1).expect("empty node"),
            frame(0.0, 0.0, 120.0, 100.0),
        )
        .expect("empty state must remain visible");

    assert_eq!(empty.frame.y, 4.0);
    assert!(!empty.hovered);
    assert_eq!(clip, frame(10.0, 20.0, 80.0, 40.0));
}

fn node(control_id: &str, x: f32, y: f32, width: f32, height: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
