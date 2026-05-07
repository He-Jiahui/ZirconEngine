use std::collections::BTreeSet;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiSurfaceDebugSnapshot, UiWidgetReflectorNode},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EditorUiDebugReflectorSummary {
    pub title: String,
    pub export_status: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EditorUiDebugReflectorNodeRow {
    pub node_id: UiNodeId,
    pub depth: usize,
    pub label: String,
    pub selected: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EditorUiDebugReflectorSection {
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EditorUiDebugReflectorModel {
    pub summary: EditorUiDebugReflectorSummary,
    pub nodes: Vec<EditorUiDebugReflectorNodeRow>,
    pub details: Vec<String>,
    pub sections: Vec<EditorUiDebugReflectorSection>,
    pub warnings: Vec<String>,
}

impl EditorUiDebugReflectorModel {
    pub(crate) fn no_active_surface() -> Self {
        Self {
            summary: EditorUiDebugReflectorSummary {
                title: "UI Debug Reflector: no active surface snapshot".to_string(),
                export_status: "Export unavailable until a shared UI surface frame is active"
                    .to_string(),
            },
            details: vec![
                "Waiting for Runtime Diagnostics to receive a UiSurfaceDebugSnapshot".to_string(),
            ],
            ..Self::default()
        }
    }

    pub(crate) fn from_snapshot(snapshot: &UiSurfaceDebugSnapshot) -> Self {
        let selected_node = snapshot.capture.selected_node;
        let known_nodes = snapshot
            .nodes
            .iter()
            .map(|node| node.node_id)
            .collect::<BTreeSet<_>>();
        let mut warnings = Vec::new();
        if let Some(selected_node) = selected_node {
            if !known_nodes.contains(&selected_node) {
                warnings.push(format!(
                    "Selected node {} is not present in snapshot tree",
                    selected_node.0
                ));
            }
        }

        let nodes = snapshot
            .nodes
            .iter()
            .map(|node| node_row(snapshot, node, selected_node))
            .collect::<Vec<_>>();
        let details = details(snapshot, selected_node);
        let sections = vec![
            render_section(snapshot),
            render_visualizer_section(snapshot),
            renderer_parity_section(snapshot),
            hit_section(snapshot),
            overdraw_section(snapshot),
            invalidation_section(snapshot),
            damage_section(snapshot),
        ];

        Self {
            summary: EditorUiDebugReflectorSummary {
                title: format!(
                    "UI Debug Reflector: {} nodes, {} commands, schema v{}",
                    snapshot.nodes.len(),
                    snapshot.render.command_count,
                    snapshot.capture.schema_version
                ),
                export_status: format!(
                    "JSON export ready: {} command records, {} overlay primitives, {} render visualizer overlays",
                    snapshot.render.command_records.len(),
                    snapshot.overlay_primitives.len(),
                    snapshot.render_batches.visualizer.overlays.len()
                ),
            },
            nodes,
            details,
            sections,
            warnings,
        }
    }
}

fn node_row(
    snapshot: &UiSurfaceDebugSnapshot,
    node: &UiWidgetReflectorNode,
    selected_node: Option<UiNodeId>,
) -> EditorUiDebugReflectorNodeRow {
    let depth = node.node_path.0.matches('/').count();
    let root_marker = if snapshot.roots.contains(&node.node_id) {
        " root"
    } else {
        ""
    };
    EditorUiDebugReflectorNodeRow {
        node_id: node.node_id,
        depth,
        selected: selected_node == Some(node.node_id),
        label: format!(
            "{}{} node={} z={} paint={} frame=({}, {}, {}, {}) clip=({}, {}, {}, {}) visibility={:?} enabled={} input={:?} clickable={} hoverable={} focusable={} render={} hit_entries={} hit_cells={}",
            node.node_path.0,
            root_marker,
            node.node_id.0,
            node.z_index,
            node.paint_order,
            node.frame.x,
            node.frame.y,
            node.frame.width,
            node.frame.height,
            node.clip_frame.x,
            node.clip_frame.y,
            node.clip_frame.width,
            node.clip_frame.height,
            node.visibility,
            node.enabled,
            node.input_policy,
            node.clickable,
            node.hoverable,
            node.focusable,
            node.render_command_count,
            node.hit_entry_count,
            node.hit_cell_count,
        ),
    }
}

fn details(snapshot: &UiSurfaceDebugSnapshot, selected_node: Option<UiNodeId>) -> Vec<String> {
    let mut details = Vec::new();
    details.push(format!("Tree: {}", snapshot.tree_id.0));
    details.push(format!(
        "Capture: frame={:?}, source={:?}, selected={:?}, pick={}",
        snapshot.capture.frame_index,
        snapshot.capture.source_asset,
        selected_node.map(|node| node.0),
        snapshot.capture.pick_query.is_some()
    ));
    details.push(format!(
        "Focus: focused={:?}, captured={:?}, pressed={:?}, hovered={:?}",
        snapshot.focus_state.focused.map(|node| node.0),
        snapshot.focus_state.captured.map(|node| node.0),
        snapshot.focus_state.pressed.map(|node| node.0),
        snapshot
            .focus_state
            .hovered
            .iter()
            .map(|node| node.0)
            .collect::<Vec<_>>()
    ));
    if let Some(selected_node) = selected_node {
        if let Some(node) = snapshot
            .nodes
            .iter()
            .find(|node| node.node_id == selected_node)
        {
            details.push(format!(
                "Selected: {} frame=({}, {}, {}, {}) clip=({}, {}, {}, {})",
                node.node_path.0,
                node.frame.x,
                node.frame.y,
                node.frame.width,
                node.frame.height,
                node.clip_frame.x,
                node.clip_frame.y,
                node.clip_frame.width,
                node.clip_frame.height,
            ));
            details.push(format!(
                "Selected diagnostics: render_commands={} hit_entries={} hit_cells={}",
                node.render_command_count, node.hit_entry_count, node.hit_cell_count
            ));
        }
    }
    if let Some(pick) = snapshot.pick_hit_test.as_ref() {
        details.push(format!(
            "Pick: point=({}, {}) target={:?} hit_path={:?} bubble={:?} rejected={}",
            pick.point.x,
            pick.point.y,
            pick.hit_path.target.map(|node| node.0),
            pick.hit_path
                .root_to_leaf
                .iter()
                .map(|node| node.0)
                .collect::<Vec<_>>(),
            pick.hit_path
                .bubble_route
                .iter()
                .map(|node| node.0)
                .collect::<Vec<_>>(),
            pick.rejected.len()
        ));
        for reject in &pick.rejected {
            details.push(format!(
                "Reject: node={} control={:?} reason={:?} message={}",
                reject.node_id.0, reject.control_id, reject.reason, reject.message
            ));
        }
    }
    details
}

fn render_visualizer_section(snapshot: &UiSurfaceDebugSnapshot) -> EditorUiDebugReflectorSection {
    let visualizer = &snapshot.render_batches.visualizer;
    let stats = visualizer.stats;
    let text = visualizer.text;
    EditorUiDebugReflectorSection {
        title: "Render Visualizer".to_string(),
        lines: vec![
            format!("paint elements: {}", stats.paint_element_count),
            format!("batch groups: {}", stats.batch_group_count),
            format!("draw calls: {}", stats.draw_call_count),
            format!(
                "overlays: {} records={} overdraw_regions={} resources={}",
                stats.overlay_count,
                visualizer.overlays.len(),
                stats.overdraw_region_count,
                stats.resource_binding_count
            ),
            format!(
                "text: elements={} auto={} native={} sdf={} lines={} glyphs={} decorations={} selection={} caret={} composition={}",
                text.text_element_count,
                text.auto_text_count,
                text.native_text_count,
                text.sdf_text_count,
                text.shaped_line_count,
                text.glyph_count,
                text.decoration_count,
                text.selection_count,
                text.caret_count,
                text.composition_count
            ),
            format!(
                "cache: reused={} rebuilt={}",
                stats.cached_paint_count, stats.rebuilt_paint_count
            ),
        ],
    }
}

fn renderer_parity_section(snapshot: &UiSurfaceDebugSnapshot) -> EditorUiDebugReflectorSection {
    let parity = &snapshot.render_batches.parity;
    let stats = parity.stats;
    EditorUiDebugReflectorSection {
        title: "Renderer Parity".to_string(),
        lines: vec![
            format!("tree: {}", parity.tree_id.0),
            format!(
                "paint={} batches={} clipped={} resources={} text={}",
                stats.paint_element_count,
                stats.batch_count,
                stats.clipped_paint_count,
                stats.resource_bound_paint_count,
                stats.text_paint_count
            ),
            format!("paint rows: {}", parity.paint_order.len()),
            format!("batch rows: {}", parity.batch_order.len()),
        ],
    }
}

fn render_section(snapshot: &UiSurfaceDebugSnapshot) -> EditorUiDebugReflectorSection {
    EditorUiDebugReflectorSection {
        title: "Render".to_string(),
        lines: vec![
            format!("commands: {}", snapshot.render.command_count),
            format!("material batches: {}", snapshot.render.material_batch_count),
            format!(
                "estimated draw calls: {}",
                snapshot.render.estimated_draw_calls
            ),
            format!("command records: {}", snapshot.render.command_records.len()),
            format!(
                "batch breaks: {}",
                snapshot
                    .render
                    .material_batches
                    .iter()
                    .map(|batch| format!("{} -> {}", batch.key, batch.break_reason))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
        ],
    }
}

fn hit_section(snapshot: &UiSurfaceDebugSnapshot) -> EditorUiDebugReflectorSection {
    EditorUiDebugReflectorSection {
        title: "Hit Test".to_string(),
        lines: vec![
            format!("entries: {}", snapshot.hit_test.entry_count),
            format!("cells: {}", snapshot.hit_test.cell_count),
            format!("occupied cells: {}", snapshot.hit_test.occupied_cell_count),
            format!("cell records: {}", snapshot.hit_test.cell_records.len()),
        ],
    }
}

fn overdraw_section(snapshot: &UiSurfaceDebugSnapshot) -> EditorUiDebugReflectorSection {
    EditorUiDebugReflectorSection {
        title: "Overdraw".to_string(),
        lines: vec![
            format!("covered cells: {}", snapshot.overdraw.covered_cells),
            format!("overdrawn cells: {}", snapshot.overdraw.overdrawn_cells),
            format!("max layers: {}", snapshot.overdraw.max_layers),
            format!("cell records: {}", snapshot.overdraw.cells.len()),
        ],
    }
}

fn invalidation_section(snapshot: &UiSurfaceDebugSnapshot) -> EditorUiDebugReflectorSection {
    EditorUiDebugReflectorSection {
        title: "Invalidation".to_string(),
        lines: vec![
            format!("dirty flags: {:?}", snapshot.invalidation.dirty_flags),
            format!("dirty nodes: {}", snapshot.invalidation.dirty_node_count),
            format!(
                "layout recomputed: {}",
                snapshot.invalidation.rebuild.layout_recomputed
            ),
            format!(
                "hit grid rebuilt: {}",
                snapshot.invalidation.rebuild.hit_grid_rebuilt
            ),
            format!(
                "render rebuilt: {}",
                snapshot.invalidation.rebuild.render_rebuilt
            ),
        ],
    }
}

fn damage_section(snapshot: &UiSurfaceDebugSnapshot) -> EditorUiDebugReflectorSection {
    EditorUiDebugReflectorSection {
        title: "Damage".to_string(),
        lines: vec![
            format!("region: {:?}", snapshot.damage.damage_region),
            format!("painted pixels: {:?}", snapshot.damage.painted_pixels),
            format!("full paints: {:?}", snapshot.damage.full_paint_count),
            format!("region paints: {:?}", snapshot.damage.region_paint_count),
        ],
    }
}
