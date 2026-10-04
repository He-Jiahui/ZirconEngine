// 封存清单按 ID 排序可走二分；外部反序列化的清单可能未排序，查找失败后须线性兜底。
pub(super) fn sorted_index_by_key<T, F>(values: &[T], target: &str, key: F) -> Option<usize>
where
    F: Fn(&T) -> &str,
{
    values
        .binary_search_by(|value| key(value).cmp(target))
        .ok()
        .or_else(|| values.iter().position(|value| key(value) == target))
}

#[cfg(test)]
#[path = "tests/sorted_lookup.rs"]
mod tests;
