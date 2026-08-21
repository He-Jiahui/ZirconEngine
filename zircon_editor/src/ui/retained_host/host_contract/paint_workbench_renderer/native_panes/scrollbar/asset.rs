use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{
    ActivityAssetReferenceListKind, AssetContentPaintMetadata, AssetContentRect,
    AssetContentSurface, BrowserAssetReferenceListKind,
};

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};

pub(super) fn asset_tree_viewport_frame(body: &FrameRect) -> FrameRect {
    let viewport_y = crate::ui::retained_host::asset_pointer::asset_tree_viewport_y();
    FrameRect {
        x: body.x,
        y: body.y + viewport_y,
        width: body.width,
        height: (body.height - viewport_y).max(0.0),
    }
}

pub(super) fn asset_tree_row_count(
    nodes: &ModelRc<TemplatePaneNodeData>,
    row_control_id: &str,
) -> usize {
    nodes
        .iter()
        .filter(|node| matches_asset_tree_row(node.control_id.as_str(), row_control_id))
        .count()
}

pub(super) fn activity_asset_content_viewport_and_extent(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
) -> Option<(FrameRect, f32)> {
    asset_content_viewport_and_extent(nodes, body, AssetContentSurface::Activity)
}

pub(super) fn browser_asset_content_viewport_and_extent(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
) -> Option<(FrameRect, f32)> {
    asset_content_viewport_and_extent(nodes, body, AssetContentSurface::Browser)
}

pub(super) fn browser_asset_tree_viewport_frame(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
) -> Option<FrameRect> {
    let metadata = nodes.metadata::<AssetContentPaintMetadata>()?;
    if metadata.surface() != AssetContentSurface::Browser {
        return None;
    }
    metadata
        .browser_source_tree_viewport()
        .map(|viewport| translated_asset_content_rect(viewport, body))
}

pub(super) fn browser_asset_reference_viewport_and_row_count(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
    list_kind: BrowserAssetReferenceListKind,
) -> Option<(FrameRect, usize)> {
    let metadata = nodes.metadata::<AssetContentPaintMetadata>()?;
    if metadata.surface() != AssetContentSurface::Browser {
        return None;
    }
    let viewport =
        translated_asset_content_rect(metadata.browser_reference_viewport(list_kind)?, body);
    Some((viewport, metadata.browser_reference_row_count(list_kind)))
}

pub(super) fn activity_asset_reference_viewport_and_row_count(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
    list_kind: ActivityAssetReferenceListKind,
) -> Option<(FrameRect, usize)> {
    let metadata = nodes.metadata::<AssetContentPaintMetadata>()?;
    if metadata.surface() != AssetContentSurface::Activity {
        return None;
    }
    let viewport =
        translated_asset_content_rect(metadata.activity_reference_viewport(list_kind)?, body);
    Some((viewport, metadata.activity_reference_row_count(list_kind)))
}

fn asset_content_viewport_and_extent(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
    surface: AssetContentSurface,
) -> Option<(FrameRect, f32)> {
    let metadata = nodes.metadata::<AssetContentPaintMetadata>()?;
    if metadata.surface() != surface {
        return None;
    }
    let viewport = translated_asset_content_rect(metadata.viewport()?, body);
    Some((viewport, metadata.content_extent()))
}

fn translated_asset_content_rect(rect: AssetContentRect, body: &FrameRect) -> FrameRect {
    FrameRect {
        x: body.x + rect.x,
        y: body.y + rect.y,
        width: rect.width,
        height: rect.height,
    }
}

fn matches_asset_tree_row(control_id: &str, row_control_id: &str) -> bool {
    control_id
        .rsplit('/')
        .next()
        .is_some_and(|leaf| leaf == row_control_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
    use crate::ui::retained_host::host_contract::data::TemplateNodeFrameData;
    use crate::ui::workbench::asset_content_layout::{
        asset_content_paint_metadata, ActivityAssetReferenceListKind, AssetContentPaintNodeInput,
        BrowserAssetReferenceListKind, BROWSER_CONTENT_PREVIEW_CONTROL_ID,
        BROWSER_CONTENT_TABLE_CONTROL_ID, BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID,
    };

    fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn browser_content_scrollbar_geometry_uses_generation_metadata_without_model_scans() {
        let source = include_str!("asset.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source before tests");

        assert!(!production.contains("row_data("));
        assert!(!production.contains("for row in 0..nodes.row_count()"));
        assert!(production.contains("metadata::<AssetContentPaintMetadata>"));
    }

    #[test]
    fn browser_viewport_stops_at_preview_when_table_frame_overlaps_it() {
        let mut table = node(BROWSER_CONTENT_TABLE_CONTROL_ID, 10.0, 20.0, 120.0, 80.0);
        table.value_number = 280.0;
        let nodes = browser_nodes(vec![
            table,
            node(
                BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID,
                10.0,
                20.0,
                120.0,
                24.0,
            ),
            node(BROWSER_CONTENT_PREVIEW_CONTROL_ID, 10.0, 90.0, 120.0, 40.0),
        ]);

        let (viewport, extent) = browser_asset_content_viewport_and_extent(
            &nodes,
            &FrameRect {
                x: 5.0,
                y: 7.0,
                width: 150.0,
                height: 180.0,
            },
        )
        .expect("browser viewport");

        assert_eq!(
            viewport,
            FrameRect {
                x: 15.0,
                y: 51.0,
                width: 120.0,
                height: 46.0,
            }
        );
        assert_eq!(extent, 280.0);
    }

    #[test]
    fn browser_thumbnail_viewport_uses_grid_frame_and_full_content_extent() {
        let mut grid = node("AssetBrowserThumbGridPanel", 10.0, 20.0, 320.0, 180.0);
        grid.value_number = 620.0;
        let nodes = browser_nodes(vec![grid]);

        let (viewport, extent) = browser_asset_content_viewport_and_extent(
            &nodes,
            &FrameRect {
                x: 5.0,
                y: 7.0,
                width: 350.0,
                height: 220.0,
            },
        )
        .expect("browser thumbnail viewport");

        assert_eq!(
            viewport,
            FrameRect {
                x: 15.0,
                y: 27.0,
                width: 320.0,
                height: 180.0,
            }
        );
        assert_eq!(extent, 620.0);
    }

    #[test]
    fn browser_tree_scrollbar_viewport_uses_projected_sources_panel() {
        let nodes = browser_nodes(vec![node(
            "AssetBrowserSourcesScrollBody",
            14.0,
            52.0,
            152.0,
            430.0,
        )]);

        let viewport = browser_asset_tree_viewport_frame(
            &nodes,
            &FrameRect {
                x: 5.0,
                y: 7.0,
                width: 900.0,
                height: 620.0,
            },
        )
        .expect("browser sources viewport");

        assert_eq!(
            viewport,
            FrameRect {
                x: 19.0,
                y: 59.0,
                width: 152.0,
                height: 430.0,
            }
        );
    }

    #[test]
    fn browser_reference_scrollbar_viewport_uses_its_projected_list_body() {
        let nodes = browser_nodes(vec![node(
            "AssetBrowserReferenceRightScrollBody",
            194.0,
            72.0,
            168.0,
            118.0,
        )]);

        let viewport = browser_asset_reference_viewport_and_row_count(
            &nodes,
            &FrameRect {
                x: 5.0,
                y: 7.0,
                width: 900.0,
                height: 620.0,
            },
            BrowserAssetReferenceListKind::UsedBy,
        )
        .expect("browser used-by viewport");

        assert_eq!(viewport.0, frame(199.0, 79.0, 168.0, 118.0));
        assert_eq!(viewport.1, 0);
    }

    #[test]
    fn activity_reference_scrollbar_viewport_counts_only_its_own_dynamic_rows() {
        let nodes = activity_nodes(vec![
            node(
                "AssetsActivityReferenceLeftScrollBody",
                14.0,
                52.0,
                152.0,
                118.0,
            ),
            node(
                "AssetsActivityReferenceLeftRowPanel01",
                14.0,
                52.0,
                148.0,
                34.0,
            ),
            node(
                "AssetsActivityReferenceLeftRowPanel02",
                14.0,
                90.0,
                148.0,
                34.0,
            ),
        ]);

        let viewport = activity_asset_reference_viewport_and_row_count(
            &nodes,
            &FrameRect {
                x: 5.0,
                y: 7.0,
                width: 900.0,
                height: 620.0,
            },
            ActivityAssetReferenceListKind::References,
        )
        .expect("activity references viewport");

        assert_eq!(viewport.0, frame(19.0, 59.0, 152.0, 118.0));
        assert_eq!(viewport.1, 2);
    }

    #[test]
    fn browser_reference_scrollbars_evaluate_both_columns_before_combining_results() {
        let source = include_str!("../scrollbar.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source before tests");

        assert!(production.contains("let references = draw_browser_asset_reference_scrollbar"));
        assert!(production.contains("let used_by = draw_browser_asset_reference_scrollbar"));
        assert!(production.contains("references || used_by"));
        assert!(
            !production.contains(") || draw_browser_asset_reference_scrollbar"),
            "short-circuiting would skip the Used By scrollbar whenever References overflows"
        );
    }

    fn browser_nodes(nodes: Vec<ViewTemplateNodeData>) -> ModelRc<TemplatePaneNodeData> {
        asset_nodes(nodes, AssetContentSurface::Browser)
    }

    fn activity_nodes(nodes: Vec<ViewTemplateNodeData>) -> ModelRc<TemplatePaneNodeData> {
        asset_nodes(nodes, AssetContentSurface::Activity)
    }

    fn asset_nodes(
        nodes: Vec<ViewTemplateNodeData>,
        surface: AssetContentSurface,
    ) -> ModelRc<TemplatePaneNodeData> {
        view_asset_content_model(nodes, surface).map_preserving_metadata(|node| {
            TemplatePaneNodeData {
                control_id: node.control_id.clone(),
                value_number: node.value_number,
                frame: TemplateNodeFrameData {
                    x: node.frame.x,
                    y: node.frame.y,
                    width: node.frame.width,
                    height: node.frame.height,
                },
                ..TemplatePaneNodeData::default()
            }
        })
    }

    fn view_asset_content_model(
        nodes: Vec<ViewTemplateNodeData>,
        surface: AssetContentSurface,
    ) -> ModelRc<ViewTemplateNodeData> {
        let metadata = asset_content_paint_metadata(
            nodes.iter().map(|node| {
                AssetContentPaintNodeInput::new(
                    node.control_id.as_str(),
                    node.frame.x,
                    node.frame.y,
                    node.frame.width,
                    node.frame.height,
                    node.value_number,
                )
            }),
            surface,
        );
        ModelRc::with_metadata(nodes, metadata)
    }

    fn node(control_id: &str, x: f32, y: f32, width: f32, height: f32) -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            control_id: control_id.into(),
            frame: ViewTemplateFrameData {
                x,
                y,
                width,
                height,
            },
            ..ViewTemplateNodeData::default()
        }
    }
}
