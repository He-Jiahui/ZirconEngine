use std::collections::BTreeMap;

use super::*;
use crate::ui::retained_host::hierarchy_pointer::hierarchy_paint_metadata;
use crate::ui::retained_host::host_contract::data::{TemplateNodeFrameData, TemplatePaneNodeData};
use crate::ui::retained_host::primitives::ModelRc;

fn node(control_id: &str, y: f32, height: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        frame: TemplateNodeFrameData {
            x: 4.0,
            y,
            width: 92.0,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn viewport_reads_live_candidate_geometry_after_metadata_preserving_patch() {
    let original = vec![
        node("HierarchyListPanel", 10.0, 0.0),
        node("HierarchyTreeSlotAnchor", 40.0, 20.0),
    ];
    let metadata = hierarchy_paint_metadata(original.iter().map(|node| node.control_id.as_str()));
    let nodes = ModelRc::with_metadata(original, metadata).with_row_patches(BTreeMap::from([(
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
