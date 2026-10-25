use zircon_runtime_interface::ui::{
    layout::{Anchor, Pivot, Position, UiCanvasSlotPlacement, UiContainerKind, UiFrame, UiMargin},
    tree::UiTree,
};

pub(super) struct CanvasAnchorGeometry {
    parent_content: UiFrame,
    child: UiFrame,
    placement: UiCanvasSlotPlacement,
    has_slot_placement: bool,
}

/// Only an unambiguous native Canvas edge in the current root instance can
/// supply geometry. Component callsites and Free constraints need their own
/// ownership/size contracts; matching a repeated control label is insufficient.
pub(super) fn selected_canvas_geometry(
    tree: &UiTree,
    source_node_id: &str,
    source_parent_id: &str,
) -> Option<CanvasAnchorGeometry> {
    // Ancestor or sibling layout/style work can invalidate the selected edge
    // before its own measure cache is cleared. Consume no cached frame then.
    if tree.nodes.values().any(|node| {
        node.dirty.layout || node.dirty.style || node.dirty.text || node.dirty.visible_range
    }) {
        return None;
    }
    let mut matches = tree.nodes.values().filter_map(|child| {
        let metadata = child.template_metadata.as_ref()?;
        if metadata.source_node_id.as_deref() != Some(source_node_id)
            || metadata
                .instance_path
                .as_ref()
                .is_some_and(|path| !path.is_empty())
        {
            return None;
        }
        let parent = tree.node(child.parent?)?;
        let parent_metadata = parent.template_metadata.as_ref()?;
        if parent.container != UiContainerKind::Canvas
            || parent_metadata.source_node_id.as_deref() != Some(source_parent_id)
            || metadata.source_path != parent_metadata.source_path
            || metadata.instance_path != parent_metadata.instance_path
            || !child.layout_cache.measure_valid
            || !parent.layout_cache.measure_valid
        {
            return None;
        }
        let slot_index = tree.first_layout_slot_index_for_edge(parent.node_id, child.node_id)?;
        let slot = tree.layout_slot(slot_index)?;
        let placement = slot.canvas_placement.unwrap_or_else(|| {
            UiCanvasSlotPlacement::new(child.anchor, child.pivot, child.position)
        });
        let outer = parent.layout_cache.frame;
        let padding = parent.layout_padding;
        Some(CanvasAnchorGeometry {
            parent_content: UiFrame::new(
                outer.x + padding.left,
                outer.y + padding.top,
                (outer.width - padding.horizontal()).max(0.0),
                (outer.height - padding.vertical()).max(0.0),
            ),
            child: child.layout_cache.frame,
            placement,
            has_slot_placement: slot.canvas_placement.is_some(),
        })
    });
    let geometry = matches.next()?;
    matches.next().is_none().then_some(geometry)
}

impl CanvasAnchorGeometry {
    pub(super) fn original_placement(&self) -> UiCanvasSlotPlacement {
        self.placement
    }

    pub(super) fn source_placement_baseline(&self) -> UiCanvasSlotPlacement {
        if self.has_slot_placement {
            self.placement
        } else {
            // The first slot placement replaces node defaults as a whole. Its
            // omitted fields use slot defaults rather than inheriting the node.
            UiCanvasSlotPlacement::default()
        }
    }

    pub(super) fn can_rebase(&self) -> bool {
        let parent = self.parent_content;
        let child = self.child;
        let old = self.placement;
        let inputs = [
            parent.x,
            parent.y,
            parent.width,
            parent.height,
            child.x,
            child.y,
            child.width,
            child.height,
            old.pivot.x,
            old.pivot.y,
            old.position.x,
            old.position.y,
        ];
        inputs.iter().all(|value| value.is_finite())
            && parent.width > 0.0
            && parent.height > 0.0
            && child.width > 0.0
            && child.height > 0.0
            && !old.auto_size
    }

    pub(super) fn rebase(
        &self,
        minimum: [f32; 2],
        maximum: [f32; 2],
        shift: bool,
        control: bool,
    ) -> Option<UiCanvasSlotPlacement> {
        let parent = self.parent_content;
        let child = self.child;
        let old = self.placement;
        if !self.can_rebase() {
            return None;
        }
        let horizontal_stretch = minimum[0] != maximum[0];
        let vertical_stretch = minimum[1] != maximum[1];
        let old_maximum = old.resolved_anchor_max();
        let anchors_changed = old.anchor != Anchor::new(minimum[0], minimum[1])
            || old_maximum != Anchor::new(maximum[0], maximum[1]);
        let offset = if anchors_changed {
            // Rebase with the OLD pivot, before applying Shift. This ordering is
            // observable when Shift changes a point anchor's alignment.
            let left = child.x - parent.x - parent.width * minimum[0] - old.position.x
                + if horizontal_stretch {
                    0.0
                } else {
                    child.width * old.pivot.x
                };
            let top = child.y - parent.y - parent.height * minimum[1] - old.position.y
                + if vertical_stretch {
                    0.0
                } else {
                    child.height * old.pivot.y
                };
            let right = if horizontal_stretch {
                parent.x + parent.width * maximum[0] - child.x - child.width
            } else if old.anchor.x != old_maximum.x {
                child.width
            } else {
                // A point-to-point move retains authored sizing, including
                // zero's existing intrinsic-size fallback in the Canvas runtime.
                old.offset.right
            };
            let bottom = if vertical_stretch {
                parent.y + parent.height * maximum[1] - child.y - child.height
            } else if old.anchor.y != old_maximum.y {
                child.height
            } else {
                old.offset.bottom
            };
            UiMargin::new(left, top, right, bottom)
        } else {
            // UE skips rebasing when anchors did not move. Recomputing from the
            // arranged frame could freeze intrinsic size or introduce rounding.
            old.offset
        };
        let mut next = UiCanvasSlotPlacement::new(
            Anchor::new(minimum[0], minimum[1]),
            if shift {
                Pivot::new(
                    if horizontal_stretch { 0.0 } else { minimum[0] },
                    if vertical_stretch { 0.0 } else { minimum[1] },
                )
            } else {
                old.pivot
            },
            if control {
                Position::new(0.0, 0.0)
            } else {
                old.position
            },
        )
        .with_auto_size(old.auto_size)
        .with_offset(offset);
        // An absent maximum follows the minimum until a range is first authored.
        if old.anchor_max.is_some() || minimum != maximum {
            next.anchor_max = Some(Anchor::new(maximum[0], maximum[1]));
        }
        if control {
            next.offset.left = 0.0;
            next.offset.top = 0.0;
            if horizontal_stretch {
                next.offset.right = 0.0;
            }
            if vertical_stretch {
                next.offset.bottom = 0.0;
            }
        }
        [
            next.offset.left,
            next.offset.top,
            next.offset.right,
            next.offset.bottom,
        ]
        .iter()
        .all(|value| value.is_finite())
        .then_some(next)
    }
}
