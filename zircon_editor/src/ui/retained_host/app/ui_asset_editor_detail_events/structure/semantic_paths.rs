pub(super) fn slot_semantic_action_path(action_id: &str) -> Option<&'static str> {
    match action_id {
        "slot.linear.width_weight.set" => Some("layout.width.weight"),
        "slot.linear.width_stretch.set" => Some("layout.width.stretch"),
        "slot.linear.height_weight.set" => Some("layout.height.weight"),
        "slot.linear.height_stretch.set" => Some("layout.height.stretch"),
        "slot.overlay.anchor_x.set" => Some("layout.anchor.x"),
        "slot.overlay.anchor_y.set" => Some("layout.anchor.y"),
        "slot.overlay.pivot_x.set" => Some("layout.pivot.x"),
        "slot.overlay.pivot_y.set" => Some("layout.pivot.y"),
        "slot.overlay.position_x.set" => Some("layout.position.x"),
        "slot.overlay.position_y.set" => Some("layout.position.y"),
        "slot.overlay.z_index.set" => Some("layout.z_index"),
        "slot.grid.row.set" => Some("row"),
        "slot.grid.column.set" => Some("column"),
        "slot.grid.row_span.set" => Some("row_span"),
        "slot.grid.column_span.set" => Some("column_span"),
        "slot.flow.break_before.set" => Some("break_before"),
        "slot.flow.alignment.set" => Some("alignment"),
        _ => None,
    }
}

pub(super) fn layout_semantic_action_path(action_id: &str) -> Option<&'static str> {
    match action_id {
        "layout.box.gap.set" => Some("container.gap"),
        "layout.scroll.axis.set" => Some("container.axis"),
        "layout.scroll.gap.set" => Some("container.gap"),
        "layout.scroll.scrollbar_visibility.set" => Some("container.scrollbar_visibility"),
        "layout.scroll.virtualization.item_extent.set" => {
            Some("container.virtualization.item_extent")
        }
        "layout.scroll.virtualization.overscan.set" => Some("container.virtualization.overscan"),
        "layout.scroll.clip.set" => Some("clip"),
        _ => None,
    }
}
