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

#[test]
fn g37_asset_browser_catalog_names_reach_painted_pixels() {
    use crate::ui::layouts::views::asset_browser_pane_data;
    use crate::ui::retained_host::{
        paint_host_frame_for_test, template_node_command_summary_for_test, FrameRect,
        HostDocumentDockSurfaceData, HostWindowPresentationData, PaneData,
    };
    use crate::ui::workbench::snapshot::{
        AssetFolderSnapshot, AssetItemSnapshot, AssetTypeProjectionSnapshot, AssetViewMode,
        AssetWorkspaceSnapshot,
    };
    use zircon_runtime_interface::resource::ResourceKind;
    use zircon_runtime_interface::ui::layout::UiSize;

    for (width, height) in [(900, 620), (1500, 900)] {
        for view_mode in [AssetViewMode::List, AssetViewMode::Thumbnail] {
            let snapshot = AssetWorkspaceSnapshot {
                project_name: "G37".into(),
                view_mode,
                selected_folder_id: Some("res://models".into()),
                folder_tree: vec![AssetFolderSnapshot {
                    folder_id: "res://models".into(),
                    display_name: "Models".into(),
                    recursive_asset_count: 1,
                    selected: true,
                    ..Default::default()
                }],
                visible_assets: vec![AssetItemSnapshot {
                    uuid: "g37-cube".into(),
                    locator: "res://models/cube.obj".into(),
                    display_name: "cube.obj".into(),
                    file_name: "cube.obj".into(),
                    extension: "obj".into(),
                    kind: ResourceKind::Model,
                    asset_type: AssetTypeProjectionSnapshot::from_resource_kind(
                        ResourceKind::Model,
                    ),
                    preview_artifact_path: String::new(),
                    dirty: false,
                    diagnostics: Vec::new(),
                    selected: false,
                    resource_state: None,
                    resource_revision: None,
                }]
                .into(),
                ..Default::default()
            };
            let project = |snapshot: &AssetWorkspaceSnapshot| {
                to_host_contract_asset_browser_pane(
                    asset_browser_pane_data(snapshot, UiSize::new(width as f32, height as f32)),
                    PaneContentSize::new(width as f32, height as f32),
                )
            };
            let paint = |pane| {
                let bounds = FrameRect {
                    width: width as f32,
                    height: height as f32,
                    ..FrameRect::default()
                };
                let mut presentation = HostWindowPresentationData::default();
                presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
                    region_frame: bounds.clone(),
                    content_frame: bounds,
                    pane: PaneData {
                        kind: "AssetBrowser".into(),
                        asset_browser: pane,
                        ..PaneData::default()
                    },
                    ..HostDocumentDockSurfaceData::default()
                };
                paint_host_frame_for_test(width, height, &presentation)
            };
            let pane = project(&snapshot);
            let nodes = pane.nodes.clone();
            assert!(nodes.row_count() > 0);
            let painted = paint(pane);
            if let Some(directory) = std::env::var_os("ZIRCON_G37_PAINT_EVIDENCE_DIR") {
                image::save_buffer(
                    std::path::PathBuf::from(directory)
                        .join(format!("{view_mode:?}-{width}x{height}.png")),
                    &painted,
                    width,
                    height,
                    image::ColorType::Rgba8,
                )
                .expect("write G37 paint evidence to the caller-owned directory");
            }
            for (label, control_id) in [
                (
                    "cube.obj",
                    match view_mode {
                        AssetViewMode::List => "WorkbenchAssetBrowserAssetRow01",
                        AssetViewMode::Thumbnail => "AssetBrowserThumbName01",
                    },
                ),
                ("Models", "AssetBrowserSourcesRowPanel"),
            ] {
                let name_rows = nodes
                    .iter()
                    .filter(|node| node.control_id == control_id)
                    .collect::<Vec<_>>();
                assert!(
                    !name_rows.is_empty(),
                    "{view_mode:?} {width}x{height} must project {label}"
                );
                for node in &name_rows {
                    assert!(node.text.contains(label));
                    assert!(template_node_command_summary_for_test(node).text_count > 0);
                }
                // Change the logical source too: native virtual slots rebind their
                // text from metadata, so clearing only a node misses that path.
                let mut hidden_names = snapshot.clone();
                if label == "Models" {
                    hidden_names.folder_tree[0].display_name.clear();
                } else {
                    let mut asset = snapshot.visible_assets.iter().next().unwrap().clone();
                    asset.display_name.clear();
                    asset.file_name.clear();
                    hidden_names.visible_assets = vec![asset].into();
                }
                let without_names = paint(project(&hidden_names));
                let name_frame = &name_rows[0].frame;
                let changed = painted
                    .chunks_exact(4)
                    .zip(without_names.chunks_exact(4))
                    .enumerate()
                    .filter(|(pixel, (left, right))| {
                        let x = (*pixel % width as usize) as f32;
                        let y = (*pixel / width as usize) as f32;
                        x >= name_frame.x
                            && x < name_frame.x + name_frame.width
                            && y >= name_frame.y
                            && y < name_frame.y + name_frame.height
                            && left != right
                    })
                    .count();
                eprintln!("G37 {view_mode:?} {width}x{height} {label}: {changed} changed pixels");
                assert!(
                    changed > 20,
                    "{view_mode:?} {width}x{height} {label} was occluded: {changed} changed pixels"
                );
            }
        }
    }
}
