use crate::ui::retained_host::hierarchy_pointer::HierarchyPaintMetadata;

use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_geometry::{
    frame_from_template, is_visible_frame, translated,
};

pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::native_panes) fn hierarchy_viewport_frame(
    pane: &PaneData,
    body: &FrameRect,
) -> FrameRect {
    let nodes = &pane.hierarchy.nodes;
    nodes
        .metadata::<HierarchyPaintMetadata>()
        .into_iter()
        .flat_map(HierarchyPaintMetadata::viewport_node_rows)
        .filter_map(|&row| nodes.get(row))
        .map(|node| translated(&frame_from_template(&node.frame), body.x, body.y))
        .find(is_visible_frame)
        .unwrap_or_else(|| body.clone())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::ui::retained_host::hierarchy_pointer::hierarchy_paint_metadata;
    use crate::ui::retained_host::host_contract::data::TemplateNodeFrameData;
    use crate::ui::retained_host::primitives::ModelRc;

    fn node(
        control_id: &str,
        y: f32,
        height: f32,
    ) -> super::super::super::super::data::TemplatePaneNodeData {
        super::super::super::super::data::TemplatePaneNodeData {
            control_id: control_id.into(),
            frame: TemplateNodeFrameData {
                x: 4.0,
                y,
                width: 92.0,
                height,
            },
            ..super::super::super::super::data::TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn viewport_reads_live_candidate_geometry_after_metadata_preserving_patch() {
        let original = vec![
            node("HierarchyListPanel", 10.0, 0.0),
            node("HierarchyTreeSlotAnchor", 40.0, 20.0),
        ];
        let metadata =
            hierarchy_paint_metadata(original.iter().map(|node| node.control_id.as_str()));
        let nodes =
            ModelRc::with_metadata(original, metadata).with_row_patches(BTreeMap::from([(
                0,
                node("HierarchyListPanel", 16.0, 20.0),
            )]));
        let mut pane = PaneData::default();
        pane.hierarchy.nodes = nodes;

        assert_eq!(
            hierarchy_viewport_frame(
                &pane,
                &FrameRect {
                    x: 10.0,
                    y: 20.0,
                    width: 120.0,
                    height: 80.0,
                },
            ),
            FrameRect {
                x: 14.0,
                y: 36.0,
                width: 92.0,
                height: 20.0,
            }
        );
    }
}
