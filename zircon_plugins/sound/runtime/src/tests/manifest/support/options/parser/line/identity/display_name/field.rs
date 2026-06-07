pub(super) fn option_display_name_value(line: &str) -> Option<&str> {
    line.strip_prefix("display_name = \"")
        .and_then(|value| value.strip_suffix('"'))
}
