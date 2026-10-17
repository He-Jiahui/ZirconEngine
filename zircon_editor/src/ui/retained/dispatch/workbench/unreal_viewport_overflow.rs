use std::collections::BTreeSet;

use zircon_runtime::ui::tree::UiRuntimeTreeLayoutExt;
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::UiNodeId,
    layout::{UiContainerKind, UiSize, UiSlot, UiSlotKind},
    tree::UiTree,
};

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const TOOLBAR: &str = "UnrealViewportToolbar";
pub(crate) const OVERFLOW_CONTROL: &str = "UnrealViewportToolbarOverflow";
const MORE: &str = "UnrealViewportToolbarMore";
const OPEN_ACTION: &str = "workbench.unreal.viewport.overflow.open";

#[derive(Default)]
pub(super) struct ViewportToolbarOverflowState {
    contract: Option<OverflowContract>,
    measured_key: Option<MeasureKey>,
    measured_group_sizes: Vec<UiSize>,
    inline_count: usize,
    open: bool,
}

struct OverflowContract {
    row: UiNodeId,
    popup: UiNodeId,
    more: UiNodeId,
    original_row_children: Vec<UiNodeId>,
    groups: Vec<OverflowGroup>,
}

struct OverflowGroup {
    control_id: String,
    node_id: UiNodeId,
    original_slot: UiSlot,
}

#[derive(PartialEq)]
struct MeasureKey {
    row_width: f32,
    viewport_width: f32,
    viewport_height: f32,
    text_revisions: Vec<(UiNodeId, u64, Option<toml::Value>)>,
}

fn source_error(message: impl Into<String>) -> BuiltinHostWindowTemplateBridgeError {
    BuiltinHostWindowTemplateBridgeError::UnrealViewportToolbarOverflow(message.into())
}

// The source row has one zero-width stretching spacer in addition to its groups.
// When needed, reserve the measured More control before choosing a source prefix.
fn inline_prefix(widths: &[f32], available: f32, more_width: f32, gap: f32) -> usize {
    let full = widths.iter().sum::<f32>() + gap * widths.len() as f32;
    if full <= available {
        return widths.len();
    }
    let mut used = more_width + gap;
    let mut count = 0;
    for &width in widths {
        let next = used + width + gap;
        if next > available {
            break;
        }
        used = next;
        count += 1;
    }
    count
}

// Moves only a contract-owned direct child between two sibling containers.
// Node identity, incarnation, paint order, metadata and event bindings stay intact.
fn move_group(
    tree: &mut UiTree,
    group: &OverflowGroup,
    row: UiNodeId,
    popup: UiNodeId,
    to_popup: bool,
    popup_order: usize,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    let parent = tree
        .node(group.node_id)
        .and_then(|node| node.parent)
        .ok_or_else(|| source_error("overflow group has no live parent"))?;
    if parent != row && parent != popup {
        return Err(source_error(
            "overflow group is outside its source row/popup",
        ));
    }
    let target = if to_popup { popup } else { row };
    if parent == target {
        return Ok(());
    }
    if tree.node(row).and_then(|node| node.parent) != tree.node(popup).and_then(|node| node.parent)
    {
        return Err(source_error(
            "overflow row and popup must have the same source parent",
        ));
    }
    tree.node_mut(parent)
        .ok_or_else(|| source_error("missing overflow parent"))?
        .children
        .retain(|child| *child != group.node_id);
    tree.node_mut(target)
        .ok_or_else(|| source_error("missing overflow target"))?
        .children
        .push(group.node_id);
    tree.node_mut(group.node_id)
        .ok_or_else(|| source_error("missing overflow group"))?
        .parent = Some(target);
    tree.retain_layout_slots(|slot| slot.child_id != group.node_id);
    let mut slot = group.original_slot.clone();
    slot.parent_id = target;
    if to_popup {
        slot.kind = UiSlotKind::Scrollable;
        slot.order = popup_order as i32;
    }
    tree.push_layout_slot(slot);
    tree.mark_layout_order_changed(parent);
    tree.mark_layout_order_changed(target);
    tree.mark_layout_dirty(group.node_id)?;
    tree.mark_layout_dirty(parent)?;
    tree.mark_layout_dirty(target)?;
    Ok(())
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    fn capture_unreal_overflow_contract(
        &self,
    ) -> Result<Option<OverflowContract>, BuiltinHostWindowTemplateBridgeError> {
        let groups = self.control_string_array(TOOLBAR, "toolbar_overflow_group_controls");
        if groups.is_empty() {
            return Ok(None);
        }
        let row_id = self
            .control_string(TOOLBAR, "toolbar_overflow_row_control")
            .ok_or_else(|| source_error("missing source overflow row control"))?;
        let row = self
            .control_node_id(&row_id)
            .ok_or_else(|| source_error("missing source overflow row"))?;
        let popup = self
            .control_node_id(OVERFLOW_CONTROL)
            .ok_or_else(|| source_error("missing source overflow popup"))?;
        let more = self
            .control_node_id(MORE)
            .ok_or_else(|| source_error("missing source More control"))?;
        let tree = &self.template_surface.surface.tree;
        let toolbar = self
            .control_node_id(TOOLBAR)
            .ok_or_else(|| source_error("missing source toolbar"))?;
        if tree.node(row).and_then(|node| node.parent) != Some(toolbar)
            || tree.node(popup).and_then(|node| node.parent) != Some(toolbar)
            || tree.node(more).and_then(|node| node.parent) != Some(row)
            || self
                .control_string(OVERFLOW_CONTROL, "toolbar_overflow_role")
                .as_deref()
                != Some("toolbar-overflow")
        {
            return Err(source_error(
                "More and popup do not belong to the authored toolbar",
            ));
        }
        let mut seen = BTreeSet::new();
        let mut source_groups = Vec::new();
        for control_id in groups {
            let node_id = self
                .control_node_id(&control_id)
                .ok_or_else(|| source_error(format!("missing source group {control_id}")))?;
            if !seen.insert(node_id) || tree.node(node_id).and_then(|node| node.parent) != Some(row)
            {
                return Err(source_error(
                    "source overflow group is duplicated or outside the row",
                ));
            }
            let slots = tree
                .layout_slots()
                .iter()
                .filter(|slot| slot.parent_id == row && slot.child_id == node_id)
                .collect::<Vec<_>>();
            if slots.len() != 1 {
                return Err(source_error(
                    "source overflow group needs exactly one original slot",
                ));
            }
            source_groups.push(OverflowGroup {
                control_id,
                node_id,
                original_slot: slots[0].clone(),
            });
        }
        Ok(Some(OverflowContract {
            row,
            popup,
            more,
            original_row_children: tree
                .node(row)
                .ok_or_else(|| source_error("missing source row"))?
                .children
                .clone(),
            groups: source_groups,
        }))
    }

    fn overflow_measure_key(
        &self,
        contract: &OverflowContract,
    ) -> Result<MeasureKey, BuiltinHostWindowTemplateBridgeError> {
        let row = self
            .template_surface
            .surface
            .tree
            .node(contract.row)
            .ok_or_else(|| source_error("missing source row"))?;
        let viewport = self
            .template_surface
            .control_frame("UnrealSceneViewViewport")
            .ok_or_else(|| source_error("missing authored viewport frame"))?;
        let mut text_revisions = Vec::new();
        for group in &contract.groups {
            for node_id in self.control_node_ids_with_descendants(&group.control_id) {
                let node = self
                    .template_surface
                    .surface
                    .tree
                    .node(node_id)
                    .ok_or_else(|| source_error("missing source group descendant"))?;
                // A native value caption is also a measurement input. Keep it
                // explicit even when a property's generic impact is paint-only.
                let caption = node
                    .template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.attributes.get("value_text"))
                    .cloned();
                text_revisions.push((node_id, node.layout_cache.text_layout_revision, caption));
            }
        }
        Ok(MeasureKey {
            row_width: row.layout_cache.frame.width,
            viewport_width: viewport.width,
            viewport_height: viewport.height,
            text_revisions,
        })
    }

    pub(super) fn refresh_unreal_viewport_toolbar_overflow(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if self.viewport_toolbar_overflow.contract.is_none() {
            self.viewport_toolbar_overflow.contract = self.capture_unreal_overflow_contract()?;
        }
        let Some(contract) = self.viewport_toolbar_overflow.contract.take() else {
            return Ok(());
        };
        let result = self.reflow_unreal_overflow(&contract);
        self.viewport_toolbar_overflow.contract = Some(contract);
        result
    }

    fn reflow_unreal_overflow(
        &mut self,
        contract: &OverflowContract,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let key = self.overflow_measure_key(contract)?;
        if self.viewport_toolbar_overflow.measured_key.as_ref() == Some(&key) {
            return Ok(());
        }
        // Measure canonical source groups with the popup closed. Hidden-parent zero
        // desired sizes must never become a subsequent overflow decision.
        for group in &contract.groups {
            move_group(
                &mut self.template_surface.surface.tree,
                group,
                contract.row,
                contract.popup,
                false,
                0,
            )?;
        }
        let row = self
            .template_surface
            .surface
            .tree
            .node_mut(contract.row)
            .ok_or_else(|| source_error("missing source row"))?;
        row.children.sort_by_key(|child| {
            contract
                .original_row_children
                .iter()
                .position(|original| original == child)
                .unwrap_or(usize::MAX)
        });
        self.set_visible(MORE, false)?;
        self.set_visible(OVERFLOW_CONTROL, false)?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;

        let tree = &self.template_surface.surface.tree;
        let row = tree
            .node(contract.row)
            .ok_or_else(|| source_error("missing source row"))?;
        let UiContainerKind::HorizontalBox(config) = row.container else {
            return Err(source_error("source toolbar row must be HorizontalBox"));
        };
        let available = row.layout_cache.frame.width - row.layout_padding.horizontal();
        let gap = config.gap;
        let more_width = tree
            .node(contract.more)
            .ok_or_else(|| source_error("missing source More control"))?
            .constraints
            .width
            .preferred;
        let mut measured_group_sizes = Vec::with_capacity(contract.groups.len());
        let sizes = contract
            .groups
            .iter()
            .map(|group| {
                let node = tree
                    .node(group.node_id)
                    .ok_or_else(|| source_error("missing source group"))?;
                measured_group_sizes.push(UiSize::new(
                    node.layout_cache.desired_size.width,
                    node.layout_cache.desired_size.height,
                ));
                Ok(UiSize::new(
                    node.layout_cache.desired_size.width + group.original_slot.padding.horizontal(),
                    node.layout_cache.desired_size.height + group.original_slot.padding.vertical(),
                ))
            })
            .collect::<Result<Vec<_>, BuiltinHostWindowTemplateBridgeError>>()?;
        if !available.is_finite()
            || available < 0.0
            || !gap.is_finite()
            || gap < 0.0
            || !more_width.is_finite()
            || more_width <= 0.0
            || sizes.iter().any(|size| {
                !size.width.is_finite()
                    || size.width <= 0.0
                    || !size.height.is_finite()
                    || size.height <= 0.0
            })
        {
            return Err(source_error(
                "source overflow measurement must be finite and positive",
            ));
        }
        let widths = sizes.iter().map(|size| size.width).collect::<Vec<_>>();
        let count = inline_prefix(&widths, available, more_width, gap);
        for (index, group) in contract.groups.iter().enumerate().skip(count) {
            move_group(
                &mut self.template_surface.surface.tree,
                group,
                contract.row,
                contract.popup,
                true,
                index - count,
            )?;
        }
        self.viewport_toolbar_overflow.inline_count = count;
        let overflowing = count < contract.groups.len();
        self.set_visible(MORE, overflowing)?;
        if !overflowing {
            self.viewport_toolbar_overflow.open = false;
        }
        if overflowing {
            let viewport = self
                .template_surface
                .control_frame("UnrealSceneViewViewport")
                .ok_or_else(|| source_error("missing source viewport frame"))?;
            let toolbar = self
                .template_surface
                .control_frame(TOOLBAR)
                .ok_or_else(|| source_error("missing source toolbar frame"))?;
            let popup = self
                .template_surface
                .surface
                .tree
                .node(contract.popup)
                .ok_or_else(|| source_error("missing source popup"))?;
            let UiContainerKind::ScrollableBox(popup_config) = popup.container else {
                return Err(source_error("source overflow popup must be ScrollableBox"));
            };
            let popup_padding = popup.layout_padding;
            let height_budget = (viewport.bottom() - toolbar.bottom() - popup.position.y).max(0.0);
            let suffix = &sizes[count..];
            let content_width = suffix.iter().map(|size| size.width).fold(0.0_f32, f32::max);
            let content_height = suffix.iter().map(|size| size.height).sum::<f32>()
                + popup_config.gap * suffix.len().saturating_sub(1) as f32;
            self.set_fixed_control_extent(
                OVERFLOW_CONTROL,
                UiSize::new(
                    (content_width + popup_padding.horizontal())
                        .min(available)
                        .max(0.0),
                    (content_height + popup_padding.vertical())
                        .min(height_budget)
                        .max(0.0),
                ),
            )?;
        }
        self.apply_unreal_overflow_open_state()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        self.viewport_toolbar_overflow.measured_group_sizes = measured_group_sizes;
        self.viewport_toolbar_overflow.measured_key = Some(self.overflow_measure_key(contract)?);
        Ok(())
    }

    fn apply_unreal_overflow_open_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let open = self.viewport_toolbar_overflow.open;
        self.set_visible(OVERFLOW_CONTROL, open)?;
        self.mutate_control_property(OVERFLOW_CONTROL, "popup_open", UiValue::Bool(open))?;
        self.mutate_control_property(OVERFLOW_CONTROL, "selected", UiValue::Bool(open))?;
        self.set_control_active(MORE, open)?;
        Ok(())
    }

    pub(super) fn apply_unreal_overflow_action(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if control_id != MORE || action_id != OPEN_ACTION {
            return Ok(false);
        }
        let Some(contract) = self.viewport_toolbar_overflow.contract.as_ref() else {
            return Ok(true);
        };
        if self.viewport_toolbar_overflow.inline_count >= contract.groups.len() {
            return Ok(true);
        }
        if self.viewport_toolbar_overflow.open {
            self.close_unreal_viewport_overflow()?;
        } else {
            self.viewport_toolbar_overflow.open = true;
            self.apply_unreal_overflow_open_state()?;
            self.template_surface
                .refresh_after_state_change(self.runtime.as_ref())?;
            if let Some(popup) = self.control_node_id(OVERFLOW_CONTROL) {
                self.template_surface.surface.focus_node(popup)?;
            }
            // Capture the source frame after focus has published its state.
            self.template_surface
                .refresh_after_state_change(self.runtime.as_ref())?;
        }
        Ok(true)
    }

    // These are the actual logical group measurements made in the original row,
    // before moving a suffix under a possibly hidden or stretched popup parent.
    // Metadata stays outside UiTree and changes no presentation cursor.
    pub(super) fn attach_unreal_toolbar_desired_sizes(
        &self,
        publication: &mut serde_json::Value,
    ) -> Option<()> {
        let state = &self.viewport_toolbar_overflow;
        let contract = state.contract.as_ref()?;
        let current_key = self.overflow_measure_key(contract).ok()?;
        if state.measured_key.as_ref() != Some(&current_key)
            || state.measured_group_sizes.len() != contract.groups.len()
        {
            return None;
        }
        let group_ids = publication.get("groupNodeIds")?.as_array()?.clone();
        if group_ids.len() != contract.groups.len() {
            return None;
        }
        let nodes = publication.get_mut("nodes")?.as_array_mut()?;
        for ((group, size), original_id) in contract
            .groups
            .iter()
            .zip(&state.measured_group_sizes)
            .zip(group_ids)
        {
            if ![size.width, size.height]
                .iter()
                .all(|value| value.is_finite() && *value >= 0.0)
            {
                return None;
            }
            let live = self.template_surface.surface.tree.node(group.node_id)?;
            if original_id.as_str() != Some(live.node_path.0.as_str()) {
                return None;
            }
            let mut matches = nodes
                .iter_mut()
                .filter(|node| node["nodeId"].as_str() == Some(live.node_path.0.as_str()));
            let node = matches.next()?;
            if matches.next().is_some()
                || node["controlId"].as_str() != Some(group.control_id.as_str())
            {
                return None;
            }
            node.as_object_mut()?.insert(
                "desiredSize".into(),
                serde_json::json!({
                    "width":size.width,"height":size.height,"coordinateSpace":"logical-pixels"
                }),
            );
        }
        Some(())
    }

    pub(super) fn close_unreal_viewport_overflow(
        &mut self,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.viewport_toolbar_overflow.open {
            return Ok(false);
        }
        let descendants = self.control_node_ids_with_descendants(OVERFLOW_CONTROL);
        let had_focus = self
            .template_surface
            .surface
            .focus
            .focused
            .is_some_and(|focused| descendants.contains(&focused));
        for node_id in descendants {
            self.mutate_node_bool(node_id, "popup_open", false)?;
            self.mutate_node_bool(node_id, "focused", false)?;
        }
        if had_focus {
            self.template_surface.surface.clear_focus();
        }
        self.viewport_toolbar_overflow.open = false;
        self.apply_unreal_overflow_open_state()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        if had_focus {
            if let Some(more) = self.control_node_id(MORE) {
                self.template_surface.surface.focus_node(more)?;
            }
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }
}

#[cfg(test)]
#[path = "../tests/unreal_viewport_overflow.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/viewport_toolbar_publish.rs"]
mod publication_tests;
