use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{event_ui::UiNodeId, surface::UiRenderCommand};

use super::ViewTemplateTextBinding;

#[derive(Default)]
pub(super) struct ViewTemplateRenderCommandIndex {
    command_count: usize,
    render_command_ranges: BTreeMap<UiNodeId, (usize, usize)>,
}

impl ViewTemplateRenderCommandIndex {
    pub(super) fn build(commands: &[UiRenderCommand]) -> Self {
        let mut render_command_ranges = BTreeMap::new();
        let mut start = 0;
        while start < commands.len() {
            let node_id = commands[start].node_id;
            let mut end = start + 1;
            while end < commands.len() && commands[end].node_id == node_id {
                end += 1;
            }
            // Runtime render extraction keeps each node's commands contiguous. An empty index
            // converts any contract drift into the cache owner's normal topology fallback.
            if render_command_ranges
                .insert(node_id, (start, end))
                .is_some()
            {
                render_command_ranges.clear();
                break;
            }
            start = end;
        }
        Self {
            command_count: commands.len(),
            render_command_ranges,
        }
    }

    #[cfg(test)]
    pub(super) fn command_count(&self) -> usize {
        self.command_count
    }

    pub(super) fn matches_changed_bindings(
        &self,
        commands: &[UiRenderCommand],
        bindings: &BTreeMap<String, ViewTemplateTextBinding>,
        changed_control_ids: &BTreeSet<String>,
    ) -> bool {
        commands.len() == self.command_count
            && changed_control_ids
                .iter()
                .filter_map(|control_id| bindings.get(control_id))
                .all(|binding| {
                    self.indexed_render_commands(commands, binding.node_id)
                        .is_some()
                })
    }

    pub(super) fn indexed_render_commands<'a>(
        &self,
        commands: &'a [UiRenderCommand],
        node_id: UiNodeId,
    ) -> Option<&'a [UiRenderCommand]> {
        let (start, end) = self.render_command_ranges.get(&node_id).copied()?;
        let commands = commands.get(start..end)?;
        (!commands.is_empty() && commands.iter().all(|command| command.node_id == node_id))
            .then_some(commands)
    }
}
