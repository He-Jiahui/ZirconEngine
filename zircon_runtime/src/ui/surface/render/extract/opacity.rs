use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{event_ui::UiNodeId, tree::UiTree};

use crate::ui::surface::UiSurface;

use super::super::resolve::resolve_opacity;

/// Resolve against the retained parent chain, not the (possibly partial) draw order.
#[derive(Default)]
pub(super) struct EffectivePaintOpacity {
    resolved: BTreeMap<UiNodeId, f32>,
    path: Vec<(UiNodeId, f32)>,
}

impl EffectivePaintOpacity {
    pub(super) fn resolve(&mut self, tree: &UiTree, node_id: UiNodeId) -> Option<f32> {
        self.path.clear();
        let mut opacity = 1.0;
        let mut current = Some(node_id);
        while let Some(id) = current {
            if let Some(cached) = self.resolved.get(&id) {
                opacity = *cached;
                break;
            }
            if self.path.len() == tree.nodes.len() {
                return None;
            }
            let node = tree.node(id)?;
            let local = resolve_opacity(node.template_metadata.as_ref());
            // A malformed declaration must not poison the opacity of the entire branch.
            self.path
                .push((id, if local.is_nan() { 1.0 } else { local }));
            current = node.parent;
        }
        while let Some((id, local)) = self.path.pop() {
            opacity *= local;
            self.resolved.insert(id, opacity);
        }
        Some(opacity)
    }
}

impl UiSurface {
    pub(in crate::ui::surface) fn expand_opacity_render_patch_nodes(
        &self,
        changed_node_ids: &mut BTreeSet<UiNodeId>,
    ) -> Result<(), ()> {
        let mut next_opacity = EffectivePaintOpacity::default();
        let mut pending = Vec::new();
        for id in changed_node_ids.iter().copied() {
            let node = self.tree.node(id).ok_or(())?;
            if node.children.is_empty() || !self.arranged_visibility.is_render_visible(id) {
                continue;
            }
            // Extraction emits the owner first, even for a group without a brush. Without
            // that prior value we cannot prove reuse; the caller falls back to full paint.
            let previous = self
                .current_render_commands_for_node(id)
                .and_then(|commands| commands.first())
                .filter(|command| command.node_id == id)
                .ok_or(())?;
            if previous.opacity != next_opacity.resolve(&self.tree, id).ok_or(())? {
                pending.extend(node.children.iter().map(|&child| (id, child)));
            }
        }
        while let Some((parent, id)) = pending.pop() {
            let node = self.tree.node(id).ok_or(())?;
            if node.parent != Some(parent) {
                return Err(());
            }
            // Originally dirty descendants already had their own opacity checked above.
            if self.arranged_visibility.is_render_visible(id) && changed_node_ids.insert(id) {
                pending.extend(node.children.iter().map(|&child| (id, child)));
            }
        }
        Ok(())
    }
}
