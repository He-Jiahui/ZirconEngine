use std::collections::HashMap;

use super::menu_item_spec::MenuItemSpec;

#[cfg(test)]
#[path = "menu_item_tree/capacity_tests.rs"]
mod capacity_tests;

pub(in crate::ui::retained_host::menu_pointer) fn menu_item_route_indices(
    items: &[MenuItemSpec],
) -> HashMap<Vec<usize>, usize> {
    let (item_count, max_depth) = menu_item_tree_shape(items);
    let mut indices = HashMap::with_capacity(item_count);
    let mut path = Vec::with_capacity(max_depth);
    let mut current = 0;
    index_item_paths(items, &mut path, &mut current, &mut indices);
    indices
}

fn menu_item_tree_shape(items: &[MenuItemSpec]) -> (usize, usize) {
    fn accumulate(
        items: &[MenuItemSpec],
        depth: usize,
        item_count: &mut usize,
        max_depth: &mut usize,
    ) {
        for item in items {
            *item_count = item_count.saturating_add(1);
            *max_depth = (*max_depth).max(depth);
            accumulate(
                &item.children,
                depth.saturating_add(1),
                item_count,
                max_depth,
            );
        }
    }

    let mut item_count = 0usize;
    let mut max_depth = 0usize;
    accumulate(items, 1, &mut item_count, &mut max_depth);
    (item_count, max_depth)
}

pub(in crate::ui::retained_host::menu_pointer) fn parent_path(path: &[usize]) -> Vec<usize> {
    path.iter()
        .take(path.len().saturating_sub(1))
        .copied()
        .collect()
}

fn index_item_paths(
    items: &[MenuItemSpec],
    path: &mut Vec<usize>,
    current: &mut usize,
    indices: &mut HashMap<Vec<usize>, usize>,
) {
    for (index, item) in items.iter().enumerate() {
        path.push(index);
        indices.insert(path.clone(), *current);
        *current = current.saturating_add(1);
        index_item_paths(&item.children, path, current, indices);
        path.pop();
    }
}
