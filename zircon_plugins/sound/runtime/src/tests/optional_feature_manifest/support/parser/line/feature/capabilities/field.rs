pub(super) fn feature_capabilities_value(line: &str) -> Option<&str> {
    super::super::super::field::bracketed_value(line, "capabilities = [")
}
