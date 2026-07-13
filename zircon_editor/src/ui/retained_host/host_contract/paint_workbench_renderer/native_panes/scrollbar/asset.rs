use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{
    ACTIVITY_CONTENT_PANEL_CONTROL_ID, BROWSER_CONTENT_PREVIEW_CONTROL_ID,
    BROWSER_CONTENT_TABLE_CONTROL_ID, BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID,
    BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
};

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::{frame_from_template, translated};

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
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .filter(|node| matches_asset_tree_row(node.control_id.as_str(), row_control_id))
        .count()
}

pub(super) fn activity_asset_content_viewport_and_extent(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
) -> Option<(FrameRect, f32)> {
    let panel = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| {
            node.control_id.rsplit('/').next() == Some(ACTIVITY_CONTENT_PANEL_CONTROL_ID)
        })?;
    let viewport = translated(&frame_from_template(&panel.frame), body.x, body.y);
    let extent = if panel.value_number.is_finite() {
        panel.value_number.max(0.0)
    } else {
        0.0
    };
    Some((viewport, extent))
}

pub(super) fn browser_asset_content_viewport_and_extent(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
) -> Option<(FrameRect, f32)> {
    if let Some(grid) = find_node(nodes, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID) {
        let viewport = translated(&frame_from_template(&grid.frame), body.x, body.y);
        let extent = if grid.value_number.is_finite() {
            grid.value_number.max(0.0)
        } else {
            0.0
        };
        return Some((viewport, extent));
    }

    let table = find_node(nodes, BROWSER_CONTENT_TABLE_CONTROL_ID)?;
    let header = find_node(nodes, BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID)?;
    let table_frame = translated(&frame_from_template(&table.frame), body.x, body.y);
    let header_bottom = body.y + header.frame.y + header.frame.height;
    let rows_bottom = find_node(nodes, BROWSER_CONTENT_PREVIEW_CONTROL_ID)
        .map(|preview| body.y + preview.frame.y)
        .unwrap_or(table_frame.y + table_frame.height)
        .min(table_frame.y + table_frame.height);
    let viewport = FrameRect {
        x: table_frame.x,
        y: header_bottom,
        width: table_frame.width,
        height: (rows_bottom - header_bottom).max(0.0),
    };
    let extent = if table.value_number.is_finite() {
        table.value_number.max(0.0)
    } else {
        0.0
    };
    Some((viewport, extent))
}

fn find_node(
    nodes: &ModelRc<TemplatePaneNodeData>,
    control_id: &str,
) -> Option<TemplatePaneNodeData> {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.rsplit('/').next() == Some(control_id))
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
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::host_contract::data::TemplateNodeFrameData;

    #[test]
    fn browser_viewport_stops_at_preview_when_table_frame_overlaps_it() {
        let mut table = node(BROWSER_CONTENT_TABLE_CONTROL_ID, 10.0, 20.0, 120.0, 80.0);
        table.value_number = 280.0;
        let nodes = model_rc(vec![
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
        let nodes = model_rc(vec![grid]);

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
}
