pub(in super::super) fn string_array_values(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter_map(|entry| entry.strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_string)
        .collect()
}
