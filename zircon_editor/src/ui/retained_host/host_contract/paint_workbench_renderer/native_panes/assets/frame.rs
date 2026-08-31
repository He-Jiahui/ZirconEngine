use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::AssetContentPaintMetadata;

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::{frame_from_template, translated};

pub(super) fn asset_tree_row_frame(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
    hovered_index: usize,
    scroll_px: f32,
) -> Option<FrameRect> {
    let row = nodes
        .metadata::<AssetContentPaintMetadata>()?
        .activity_tree_node_row(hovered_index)?;
    let node = nodes.get(row)?;
    let mut frame = translated(&frame_from_template(&node.frame), body.x, body.y);
    frame.y -= scroll_px;
    Some(frame)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::ui::retained_host::host_contract::data::TemplateNodeFrameData;
    use crate::ui::workbench::asset_content_layout::{
        asset_content_paint_metadata, AssetContentPaintNodeInput, AssetContentSurface,
    };

    fn tree_node(y: f32) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "AssetsActivityTreeRowPanel".into(),
            frame: TemplateNodeFrameData {
                x: 4.0,
                y,
                width: 92.0,
                height: 18.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn activity_tree_hover_reads_live_frame_after_metadata_preserving_row_patch() {
        let original = tree_node(10.0);
        let metadata = asset_content_paint_metadata(
            std::iter::once(AssetContentPaintNodeInput::new(
                original.control_id.as_str(),
                original.frame.x,
                original.frame.y,
                original.frame.width,
                original.frame.height,
                0.0,
            )),
            AssetContentSurface::Activity,
        );
        let nodes = ModelRc::with_metadata(vec![original], metadata)
            .with_row_patches(BTreeMap::from([(0, tree_node(36.0))]));

        assert_eq!(
            asset_tree_row_frame(
                &nodes,
                &FrameRect {
                    x: 10.0,
                    y: 20.0,
                    width: 120.0,
                    height: 80.0,
                },
                0,
                6.0,
            ),
            Some(FrameRect {
                x: 14.0,
                y: 50.0,
                width: 92.0,
                height: 18.0,
            })
        );
    }
}
