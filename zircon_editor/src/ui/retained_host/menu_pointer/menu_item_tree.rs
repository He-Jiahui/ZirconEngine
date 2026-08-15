use std::collections::HashMap;

use super::menu_item_spec::MenuItemSpec;

pub(in crate::ui::retained_host::menu_pointer) fn menu_item_route_indices(
    items: &[MenuItemSpec],
) -> HashMap<Vec<usize>, usize> {
    let mut indices = HashMap::new();
    let mut path = Vec::new();
    let mut current = 0;
    index_item_paths(items, &mut path, &mut current, &mut indices);
    indices
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
