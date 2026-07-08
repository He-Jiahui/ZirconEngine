const AXIS_LABEL_ROLE: &str = "Label";
const AXIS_ICON_ROLE: &str = "Icon";
const AXIS_SVG_ICON_ROLE: &str = "SvgIcon";

pub(super) fn is_axis_label_role(role: &str) -> bool {
    role == AXIS_LABEL_ROLE || role == AXIS_ICON_ROLE || role == AXIS_SVG_ICON_ROLE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_label_roles_include_text_and_icon_shapes_only() {
        assert!(is_axis_label_role("Label"));
        assert!(is_axis_label_role("Icon"));
        assert!(is_axis_label_role("SvgIcon"));
        assert!(!is_axis_label_role("Button"));
    }
}
