pub(super) fn module_target_modes_value(line: &str) -> Option<&str> {
    line.strip_prefix("target_modes = [")
        .and_then(|value| value.strip_suffix(']'))
}
