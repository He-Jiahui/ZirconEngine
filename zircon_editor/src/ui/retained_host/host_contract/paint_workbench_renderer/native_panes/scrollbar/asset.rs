use crate::ui::workbench::asset_content_layout::{
    AssetContentRect, AssetContentScrollbarExtent, AssetContentScrollbarViewport,
};

use super::super::super::super::data::FrameRect;

pub(super) fn asset_tree_viewport_frame(body: &FrameRect) -> FrameRect {
    let viewport_y = crate::ui::retained_host::asset_pointer::asset_tree_viewport_y();
    FrameRect {
        x: body.x,
        y: body.y + viewport_y,
        width: body.width,
        height: (body.height - viewport_y).max(0.0),
    }
}

pub(super) fn asset_scrollbar_viewport(
    viewport: AssetContentScrollbarViewport,
    body: &FrameRect,
) -> FrameRect {
    match viewport {
        AssetContentScrollbarViewport::ActivityTree => asset_tree_viewport_frame(body),
        AssetContentScrollbarViewport::Local(viewport) => {
            translated_asset_content_rect(viewport, body)
        }
    }
}

pub(super) fn asset_scrollbar_content_extent(extent: AssetContentScrollbarExtent) -> f32 {
    match extent {
        AssetContentScrollbarExtent::Pixels(extent) => extent,
        AssetContentScrollbarExtent::TreeRows(row_count) => {
            crate::ui::retained_host::asset_pointer::asset_tree_content_height(row_count)
        }
        AssetContentScrollbarExtent::ReferenceRows(row_count) => {
            crate::ui::retained_host::asset_pointer::asset_reference_content_height(row_count)
        }
    }
}

fn translated_asset_content_rect(rect: AssetContentRect, body: &FrameRect) -> FrameRect {
    FrameRect {
        x: body.x + rect.x,
        y: body.y + rect.y,
        width: rect.width,
        height: rect.height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
    use crate::ui::retained_host::host_contract::data::{
        TemplateNodeFrameData, TemplatePaneNodeData,
    };
    use crate::ui::retained_host::primitives::ModelRc;
    use crate::ui::workbench::asset_content_layout::{
        asset_content_paint_metadata, AssetContentPaintMetadata, AssetContentPaintNodeInput,
        AssetContentScrollbarDescriptor, AssetContentScrollbarKind, AssetContentSurface,
        BROWSER_CONTENT_PREVIEW_CONTROL_ID, BROWSER_CONTENT_TABLE_CONTROL_ID,
        BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID,
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
        assert!(production.contains("AssetContentScrollbarViewport"));
        assert!(production.contains("AssetContentScrollbarExtent"));
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

        let viewport = descriptor_viewport(
            &nodes,
            AssetContentScrollbarKind::Content,
            &FrameRect {
                x: 5.0,
                y: 7.0,
                width: 150.0,
                height: 180.0,
            },
        );
        let extent = asset_scrollbar_content_extent(descriptor_extent(
            &nodes,
            AssetContentScrollbarKind::Content,
        ));

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

        let viewport = descriptor_viewport(
            &nodes,
            AssetContentScrollbarKind::Content,
            &FrameRect {
                x: 5.0,
                y: 7.0,
                width: 350.0,
                height: 220.0,
            },
        );
        let extent = asset_scrollbar_content_extent(descriptor_extent(
            &nodes,
            AssetContentScrollbarKind::Content,
        ));

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

        let viewport = descriptor_viewport(
            &nodes,
            AssetContentScrollbarKind::Tree,
            &FrameRect {
                x: 5.0,
                y: 7.0,
                width: 900.0,
                height: 620.0,
            },
        );

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

        let viewport = descriptor_viewport(
            &nodes,
            AssetContentScrollbarKind::UsedBy,
            &FrameRect {
                x: 5.0,
                y: 7.0,
                width: 900.0,
                height: 620.0,
            },
        );

        assert_eq!(viewport, frame(199.0, 79.0, 168.0, 118.0));
        assert_eq!(
            descriptor_extent(&nodes, AssetContentScrollbarKind::UsedBy),
            AssetContentScrollbarExtent::ReferenceRows(0)
        );
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

        let viewport = descriptor_viewport(
            &nodes,
            AssetContentScrollbarKind::References,
            &FrameRect {
                x: 5.0,
                y: 7.0,
                width: 900.0,
                height: 620.0,
            },
        );

        assert_eq!(viewport, frame(19.0, 59.0, 152.0, 118.0));
        assert_eq!(
            descriptor_extent(&nodes, AssetContentScrollbarKind::References),
            AssetContentScrollbarExtent::ReferenceRows(2)
        );
    }

    #[test]
    fn browser_reference_scrollbars_evaluate_both_columns_before_combining_results() {
        let source = include_str!("../scrollbar.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source before tests");

        assert!(production.contains("for descriptor in metadata.scrollbar_descriptors()"));
        assert!(production.contains("painted = current || painted"));
        assert!(
            !production.contains("painted = painted || current"),
            "short-circuiting would skip later descriptors after the first painted scrollbar"
        );
    }

    fn browser_nodes(nodes: Vec<ViewTemplateNodeData>) -> ModelRc<TemplatePaneNodeData> {
        asset_nodes(nodes, AssetContentSurface::Browser)
    }

    fn activity_nodes(nodes: Vec<ViewTemplateNodeData>) -> ModelRc<TemplatePaneNodeData> {
        asset_nodes(nodes, AssetContentSurface::Activity)
    }

    fn descriptor(
        nodes: &ModelRc<TemplatePaneNodeData>,
        kind: AssetContentScrollbarKind,
    ) -> AssetContentScrollbarDescriptor {
        *nodes
            .metadata::<AssetContentPaintMetadata>()
            .expect("asset content paint metadata")
            .scrollbar_descriptors()
            .iter()
            .find(|descriptor| descriptor.kind() == kind)
            .expect("typed scrollbar descriptor")
    }

    fn descriptor_viewport(
        nodes: &ModelRc<TemplatePaneNodeData>,
        kind: AssetContentScrollbarKind,
        body: &FrameRect,
    ) -> FrameRect {
        let metadata = nodes
            .metadata::<AssetContentPaintMetadata>()
            .expect("asset content paint metadata");
        asset_scrollbar_viewport(
            metadata
                .scrollbar_viewport(descriptor(nodes, kind))
                .expect("descriptor viewport"),
            body,
        )
    }

    fn descriptor_extent(
        nodes: &ModelRc<TemplatePaneNodeData>,
        kind: AssetContentScrollbarKind,
    ) -> AssetContentScrollbarExtent {
        let metadata = nodes
            .metadata::<AssetContentPaintMetadata>()
            .expect("asset content paint metadata");
        metadata.scrollbar_extent(descriptor(nodes, kind))
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
