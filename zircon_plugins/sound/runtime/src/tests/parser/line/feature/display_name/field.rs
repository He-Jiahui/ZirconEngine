pub(super) fn feature_display_name_value(line: &str) -> Option<&str> {
    super::super::super::super::field::quoted_value(line, "display_name = \"")
}
