use super::menu_item_spec::MenuItemSpec;

pub(in crate::ui::slint_host::menu_pointer) fn menu_item_route_index(
    items: &[MenuItemSpec],
    path: &[usize],
) -> Option<usize> {
    let mut current = 0usize;
    find_route_index(items, path, &mut current)
}

pub(in crate::ui::slint_host::menu_pointer) fn parent_path(path: &[usize]) -> Vec<usize> {
    path.iter()
        .take(path.len().saturating_sub(1))
        .copied()
        .collect()
}

fn find_route_index(
    items: &[MenuItemSpec],
    target_path: &[usize],
    current: &mut usize,
) -> Option<usize> {
    for (index, item) in items.iter().enumerate() {
        let item_route_index = *current;
        *current += 1;
        if target_path.len() == 1 && target_path[0] == index {
            return Some(item_route_index);
        }
        if let Some((first, rest)) = target_path.split_first() {
            if *first == index {
                if let Some(found) = find_route_index(&item.children, rest, current) {
                    return Some(found);
                }
            } else {
                skip_subtree(&item.children, current);
            }
        } else {
            skip_subtree(&item.children, current);
        }
    }
    None
}

fn skip_subtree(items: &[MenuItemSpec], current: &mut usize) {
    for item in items {
        *current += 1;
        skip_subtree(&item.children, current);
    }
}
