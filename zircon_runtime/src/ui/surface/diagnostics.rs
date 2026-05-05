use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{
        UiHitGridDebugStats, UiMaterialBatchDebugStat, UiOverdrawDebugStats, UiRenderCommand,
        UiRenderCommandKind, UiRenderDebugStats, UiSurfaceDebugOptions, UiSurfaceDebugSnapshot,
        UiSurfaceFrame, UiWidgetReflectorNode,
    },
};

pub fn debug_surface_frame(surface_frame: &UiSurfaceFrame) -> UiSurfaceDebugSnapshot {
    debug_surface_frame_with_options(surface_frame, &UiSurfaceDebugOptions::default())
}

pub fn debug_surface_frame_with_options(
    surface_frame: &UiSurfaceFrame,
    options: &UiSurfaceDebugOptions,
) -> UiSurfaceDebugSnapshot {
    let render_counts = render_command_counts(surface_frame);
    let hit_entry_counts = hit_entry_counts(surface_frame);
    let hit_cell_counts = hit_cell_counts(surface_frame);
    let nodes = surface_frame
        .arranged_tree
        .nodes
        .iter()
        .map(|node| UiWidgetReflectorNode {
            node_id: node.node_id,
            node_path: node.node_path.clone(),
            parent: node.parent,
            children: node.children.clone(),
            frame: node.frame,
            clip_frame: node.clip_frame,
            z_index: node.z_index,
            paint_order: node.paint_order,
            visibility: node.visibility,
            input_policy: node.input_policy,
            enabled: node.enabled,
            clickable: node.clickable,
            hoverable: node.hoverable,
            focusable: node.focusable,
            control_id: node.control_id.clone(),
            render_command_count: render_counts.get(&node.node_id).copied().unwrap_or(0),
            hit_entry_count: hit_entry_counts.get(&node.node_id).copied().unwrap_or(0),
            hit_cell_count: hit_cell_counts.get(&node.node_id).copied().unwrap_or(0),
        })
        .collect();

    UiSurfaceDebugSnapshot {
        capture: Default::default(),
        tree_id: surface_frame.tree_id.clone(),
        roots: surface_frame.arranged_tree.roots.clone(),
        nodes,
        rebuild: surface_frame.last_rebuild,
        render: render_debug_stats(surface_frame),
        hit_test: hit_grid_debug_stats(surface_frame),
        overdraw: overdraw_debug_stats(surface_frame, options.overdraw_sample_cell_size),
        focus_state: surface_frame.focus_state.clone(),
        invalidation: Default::default(),
        damage: Default::default(),
        events: Vec::new(),
        overlay_primitives: Vec::new(),
    }
}

fn render_command_counts(surface_frame: &UiSurfaceFrame) -> BTreeMap<UiNodeId, usize> {
    let mut counts = BTreeMap::new();
    for command in &surface_frame.render_extract.list.commands {
        *counts.entry(command.node_id).or_insert(0) += 1;
    }
    counts
}

fn hit_entry_counts(surface_frame: &UiSurfaceFrame) -> BTreeMap<UiNodeId, usize> {
    let mut counts = BTreeMap::new();
    for entry in &surface_frame.hit_grid.entries {
        *counts.entry(entry.node_id).or_insert(0) += 1;
    }
    counts
}

fn hit_cell_counts(surface_frame: &UiSurfaceFrame) -> BTreeMap<UiNodeId, usize> {
    let mut counts = BTreeMap::new();
    for cell in &surface_frame.hit_grid.cells {
        for entry_index in &cell.entries {
            let Some(entry) = surface_frame.hit_grid.entries.get(*entry_index) else {
                continue;
            };
            *counts.entry(entry.node_id).or_insert(0) += 1;
        }
    }
    counts
}

fn render_debug_stats(surface_frame: &UiSurfaceFrame) -> UiRenderDebugStats {
    let mut stats = UiRenderDebugStats::default();
    let mut material_batches: BTreeMap<String, UiMaterialBatchDebugStat> = BTreeMap::new();

    for command in &surface_frame.render_extract.list.commands {
        stats.command_count += 1;
        match command.kind {
            UiRenderCommandKind::Group => stats.group_count += 1,
            UiRenderCommandKind::Quad => stats.quad_count += 1,
            UiRenderCommandKind::Text => stats.text_count += 1,
            UiRenderCommandKind::Image => stats.image_count += 1,
        }
        if command.clip_frame.is_some_and(|clip| clip != command.frame) {
            stats.clipped_command_count += 1;
        }
        if command.opacity >= 1.0 {
            stats.opaque_command_count += 1;
        } else {
            stats.translucent_command_count += 1;
        }

        if draws_geometry(command) {
            stats.estimated_draw_calls += 1;
        }
        if command.text.as_ref().is_some_and(|text| !text.is_empty()) {
            stats.estimated_draw_calls += 1;
        }

        let key = material_batch_key(command);
        let batch =
            material_batches
                .entry(key.clone())
                .or_insert_with(|| UiMaterialBatchDebugStat {
                    key,
                    break_reason: material_batch_break_reason(command),
                    command_kind: command.kind,
                    command_count: 0,
                    clipped_command_count: 0,
                    node_ids: Vec::new(),
                });
        batch.command_count += 1;
        if command.clip_frame.is_some_and(|clip| clip != command.frame) {
            batch.clipped_command_count += 1;
        }
        batch.node_ids.push(command.node_id);
    }

    stats.material_batches = material_batches.into_values().collect();
    stats.material_batch_count = stats.material_batches.len();
    stats
}

fn hit_grid_debug_stats(surface_frame: &UiSurfaceFrame) -> UiHitGridDebugStats {
    let occupied_cell_count = surface_frame
        .hit_grid
        .cells
        .iter()
        .filter(|cell| !cell.entries.is_empty())
        .count();
    let total_entries_in_cells = surface_frame
        .hit_grid
        .cells
        .iter()
        .map(|cell| cell.entries.len())
        .sum::<usize>();
    let max_entries_per_cell = surface_frame
        .hit_grid
        .cells
        .iter()
        .map(|cell| cell.entries.len())
        .max()
        .unwrap_or(0);
    UiHitGridDebugStats {
        entry_count: surface_frame.hit_grid.entries.len(),
        cell_count: surface_frame.hit_grid.cells.len(),
        occupied_cell_count,
        max_entries_per_cell,
        average_entries_per_occupied_cell: if occupied_cell_count == 0 {
            0.0
        } else {
            total_entries_in_cells as f32 / occupied_cell_count as f32
        },
        cell_records: Vec::new(),
    }
}

fn overdraw_debug_stats(
    surface_frame: &UiSurfaceFrame,
    sample_cell_size: f32,
) -> UiOverdrawDebugStats {
    let sample_cell_size = sample_cell_size.max(1.0);
    let layers: Vec<_> = surface_frame
        .render_extract
        .list
        .commands
        .iter()
        .filter(|command| {
            command.opacity > 0.0 && !matches!(command.kind, UiRenderCommandKind::Group)
        })
        .filter_map(command_visible_frame)
        .collect();
    let Some(bounds) = union_frames(&layers) else {
        return UiOverdrawDebugStats {
            sample_cell_size,
            ..UiOverdrawDebugStats::default()
        };
    };

    let columns = (bounds.width / sample_cell_size).ceil().max(1.0) as u32;
    let rows = (bounds.height / sample_cell_size).ceil().max(1.0) as u32;
    let mut layer_counts = vec![0usize; (columns * rows) as usize];

    for layer in layers {
        for cell_index in cells_for_frame(bounds, columns, rows, sample_cell_size, layer) {
            layer_counts[cell_index] += 1;
        }
    }

    let covered_cells = layer_counts.iter().filter(|count| **count > 0).count();
    let overdrawn_cells = layer_counts.iter().filter(|count| **count > 1).count();
    let max_layers = layer_counts.iter().copied().max().unwrap_or(0);
    let total_layer_samples = layer_counts.iter().sum();

    UiOverdrawDebugStats {
        sample_cell_size,
        bounds,
        columns,
        rows,
        covered_cells,
        overdrawn_cells,
        max_layers,
        total_layer_samples,
        cells: Vec::new(),
    }
}

fn draws_geometry(command: &UiRenderCommand) -> bool {
    command.opacity > 0.0
        && (matches!(
            command.kind,
            UiRenderCommandKind::Quad | UiRenderCommandKind::Image
        ) || command.style.background_color.is_some()
            || command.style.border_color.is_some()
            || command.style.border_width > 0.0
            || command.image.is_some())
}

fn command_visible_frame(command: &UiRenderCommand) -> Option<UiFrame> {
    let frame = command.frame;
    let clipped = command
        .clip_frame
        .and_then(|clip| frame.intersection(clip))
        .unwrap_or(frame);
    (clipped.width > 0.0 && clipped.height > 0.0).then_some(clipped)
}

fn material_batch_key(command: &UiRenderCommand) -> String {
    format!(
        "kind={:?};bg={};fg={};border={};bw={:.2};radius={:.2};font={};family={};text_mode={:?};image={};opacity={:.2}",
        command.kind,
        command.style.background_color.as_deref().unwrap_or("none"),
        command.style.foreground_color.as_deref().unwrap_or("none"),
        command.style.border_color.as_deref().unwrap_or("none"),
        command.style.border_width,
        command.style.corner_radius,
        command.style.font.as_deref().unwrap_or("none"),
        command.style.font_family.as_deref().unwrap_or("none"),
        command.style.text_render_mode,
        command
            .image
            .as_ref()
            .map(|image| format!("{image:?}"))
            .unwrap_or_else(|| "none".to_string()),
        command.opacity,
    )
}

fn material_batch_break_reason(command: &UiRenderCommand) -> String {
    let clip = if command.clip_frame.is_some_and(|clip| clip != command.frame) {
        "clipped"
    } else {
        "unclipped"
    };
    let opacity = if command.opacity >= 1.0 {
        "opaque"
    } else {
        "translucent"
    };
    let resource = if command.image.is_some() {
        "image"
    } else if command.text.as_ref().is_some_and(|text| !text.is_empty()) {
        "text"
    } else {
        "style"
    };
    format!("kind={:?};{clip};{opacity};{resource}", command.kind)
}

fn union_frames(frames: &[UiFrame]) -> Option<UiFrame> {
    let mut iter = frames.iter();
    let first = *iter.next()?;
    let (mut left, mut top, mut right, mut bottom) =
        (first.x, first.y, first.right(), first.bottom());
    for frame in iter {
        left = left.min(frame.x);
        top = top.min(frame.y);
        right = right.max(frame.right());
        bottom = bottom.max(frame.bottom());
    }
    Some(UiFrame::new(left, top, right - left, bottom - top))
}

fn cells_for_frame(
    bounds: UiFrame,
    columns: u32,
    rows: u32,
    cell_size: f32,
    frame: UiFrame,
) -> Vec<usize> {
    let left = ((frame.x - bounds.x) / cell_size).floor().max(0.0) as u32;
    let top = ((frame.y - bounds.y) / cell_size).floor().max(0.0) as u32;
    let right = ((frame.right() - bounds.x) / cell_size)
        .floor()
        .max(0.0)
        .min((columns - 1) as f32) as u32;
    let bottom = ((frame.bottom() - bounds.y) / cell_size)
        .floor()
        .max(0.0)
        .min((rows - 1) as f32) as u32;
    let mut indices = Vec::new();
    for row in top..=bottom {
        for column in left..=right {
            indices.push((row * columns + column) as usize);
        }
    }
    indices
}
