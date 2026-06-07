pub(super) fn capability_status_bevy_references_value(line: &str) -> Option<&str> {
    line.strip_prefix("bevy_references = [")
        .and_then(|value| value.strip_suffix(']'))
}
