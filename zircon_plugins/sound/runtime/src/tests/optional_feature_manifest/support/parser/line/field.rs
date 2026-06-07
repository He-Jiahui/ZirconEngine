pub(super) fn raw_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    line.strip_prefix(prefix)
}

pub(super) fn quoted_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    raw_value(line, prefix).and_then(|value| value.strip_suffix('"'))
}

pub(super) fn bracketed_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    raw_value(line, prefix).and_then(|value| value.strip_suffix(']'))
}
