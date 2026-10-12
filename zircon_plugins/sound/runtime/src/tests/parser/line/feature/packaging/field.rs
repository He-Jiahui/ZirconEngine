pub(super) fn default_packaging_value(line: &str) -> Option<&str> {
    super::super::super::super::field::bracketed_value(line, "default_packaging = [")
}
