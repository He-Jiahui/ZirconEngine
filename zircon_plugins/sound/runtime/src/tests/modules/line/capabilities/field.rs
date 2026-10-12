pub(super) fn module_capabilities_value(line: &str) -> Option<&str> {
    line.strip_prefix("capabilities = [")
        .and_then(|value| value.strip_suffix(']'))
}
