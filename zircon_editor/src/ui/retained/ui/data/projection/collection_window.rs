pub(super) fn collect_visible_collection_items<I>(
    items: I,
    visible_start: i32,
    visible_count: i32,
    overscan: i32,
) -> Vec<I::Item>
where
    I: IntoIterator,
{
    if visible_count <= 0 {
        return Vec::new();
    }

    let visible_start = visible_start.max(0);
    let overscan = overscan.max(0);
    let start = visible_start.saturating_sub(overscan).max(0) as usize;
    let end = visible_start
        .saturating_add(visible_count)
        .saturating_add(overscan)
        .max(0) as usize;

    items
        .into_iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}
