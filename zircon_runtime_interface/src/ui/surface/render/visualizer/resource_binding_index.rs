use std::collections::HashMap;

use super::{UiRenderResourceKey, UiRenderVisualizerResourceBinding};

#[derive(Default)]
pub(super) struct ResourceBindingIndex {
    bindings: Vec<UiRenderVisualizerResourceBinding>,
    binding_indices_by_id: HashMap<String, Vec<usize>>,
    membership: Vec<ResourceBindingMembership>,
}

impl ResourceBindingIndex {
    pub(super) fn add(
        &mut self,
        resource: &UiRenderResourceKey,
        paint_index: Option<usize>,
        batch_index: Option<usize>,
    ) {
        // id 只缩小候选集合；是否合并仍比较完整资源键，保留同 id 的不同变体。
        let existing_index =
            self.binding_indices_by_id
                .get(resource.id.as_str())
                .and_then(|candidate_indices| {
                    candidate_indices.iter().copied().find(|&candidate_index| {
                        self.bindings[candidate_index].resource == *resource
                    })
                });

        if let Some(binding_index) = existing_index {
            let binding = &mut self.bindings[binding_index];
            let membership = &mut self.membership[binding_index];
            if let Some(paint_index) = paint_index {
                push_ordered_unique_usize(
                    &mut binding.paint_indices,
                    paint_index,
                    &mut membership.paint_indices_ordered,
                );
            }
            if let Some(batch_index) = batch_index {
                push_ordered_unique_usize(
                    &mut binding.batch_indices,
                    batch_index,
                    &mut membership.batch_indices_ordered,
                );
            }
            return;
        }

        let binding_index = self.bindings.len();
        self.bindings.push(UiRenderVisualizerResourceBinding {
            resource: resource.clone(),
            paint_indices: paint_index.into_iter().collect(),
            batch_indices: batch_index.into_iter().collect(),
        });
        self.membership.push(ResourceBindingMembership::default());
        self.binding_indices_by_id
            .entry(resource.id.clone())
            .or_default()
            .push(binding_index);
    }

    pub(super) fn into_bindings(self) -> Vec<UiRenderVisualizerResourceBinding> {
        self.bindings
    }
}

struct ResourceBindingMembership {
    paint_indices_ordered: bool,
    batch_indices_ordered: bool,
}

impl Default for ResourceBindingMembership {
    fn default() -> Self {
        Self {
            paint_indices_ordered: true,
            batch_indices_ordered: true,
        }
    }
}

fn push_ordered_unique_usize(values: &mut Vec<usize>, value: usize, ordered: &mut bool) {
    let last = values.last().copied();
    if last == Some(value) {
        return;
    }
    if *ordered && last.map_or(true, |last| last < value) {
        values.push(value);
        return;
    }
    if values.contains(&value) {
        return;
    }
    *ordered = false;
    values.push(value);
}

#[cfg(test)]
#[path = "tests/resource_binding_index.rs"]
mod tests;
