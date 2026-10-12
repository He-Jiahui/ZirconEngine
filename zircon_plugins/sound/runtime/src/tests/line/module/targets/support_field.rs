pub(super) fn module_target_modes_value(line: &str) -> Option<&str> {
    super::super::super::field::bracketed_value(line, "target_modes = [")
}
