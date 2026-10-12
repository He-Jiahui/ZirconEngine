pub(super) fn maturity_value(line: &str) -> Option<&str> {
    line.strip_prefix("maturity = \"")
        .and_then(|value| value.strip_suffix('"'))
}
