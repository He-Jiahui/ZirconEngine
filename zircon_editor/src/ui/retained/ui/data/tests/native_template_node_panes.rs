use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};

#[test]
fn asset_browser_table_nodes_receive_narrow_context_variant() {
    let data = AssetBrowserPaneViewData {
        nodes: model_rc(vec![ViewTemplateNodeData {
            control_id: "WorkbenchAssetBrowserTableHeader".into(),
            role: "Table".into(),
            component_role: "table".into(),
            component_variant: "asset-table".into(),
            frame: ViewTemplateFrameData {
                width: 420.0,
                height: 28.0,
                ..ViewTemplateFrameData::default()
            },
            ..ViewTemplateNodeData::default()
        }]),
        ..AssetBrowserPaneViewData::default()
    };

    let pane = to_host_contract_asset_browser_pane(
        data,
        PaneContentSize {
            width: 420.0,
            height: 260.0,
        },
    );
    let node = pane.nodes.row_data(0).expect("asset browser table node");

    assert!(node
        .component_variant
        .as_str()
        .split_whitespace()
        .any(|token| token == "layoutNarrow"));
}

#[test]
fn assets_activity_preserves_the_runtime_render_source_frame() {
    let source_frame = Arc::new(UiSurfaceFrame::default());
    let pane = to_host_contract_assets_activity_pane(AssetsActivityPaneViewData {
        render_source_frame: Some(Arc::clone(&source_frame)),
        ..AssetsActivityPaneViewData::default()
    });

    assert!(pane
        .render_source_frame
        .as_ref()
        .is_some_and(|frame| Arc::ptr_eq(frame, &source_frame)));
}

#[test]
fn asset_browser_preserves_the_runtime_render_source_frame() {
    let source_frame = Arc::new(UiSurfaceFrame::default());
    let pane = to_host_contract_asset_browser_pane(
        AssetBrowserPaneViewData {
            render_source_frame: Some(Arc::clone(&source_frame)),
            ..AssetBrowserPaneViewData::default()
        },
        PaneContentSize::default(),
    );

    assert!(pane
        .render_source_frame
        .as_ref()
        .is_some_and(|frame| Arc::ptr_eq(frame, &source_frame)));
}
