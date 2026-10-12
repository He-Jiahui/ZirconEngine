pub(super) fn feature_id_value(line: &str) -> Option<&str> {
    super::super::super::super::field::quoted_value(line, "id = \"")
}
