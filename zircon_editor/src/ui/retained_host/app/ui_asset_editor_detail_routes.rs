pub(super) fn widget_prop_state_target_path(action_id: &str) -> Option<&str> {
    let target_path = action_id.strip_suffix(".set")?;
    if target_path.starts_with("widget.prop.") || target_path.starts_with("widget.state.") {
        Some(target_path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::widget_prop_state_target_path;

    #[test]
    fn widget_prop_state_target_path_accepts_generic_prop_and_state_actions() {
        assert_eq!(
            widget_prop_state_target_path("widget.prop.variant.set"),
            Some("widget.prop.variant")
        );
        assert_eq!(
            widget_prop_state_target_path("widget.state.expanded.set"),
            Some("widget.state.expanded")
        );
    }

    #[test]
    fn widget_prop_state_target_path_rejects_non_patch_actions() {
        assert_eq!(widget_prop_state_target_path("widget.prop.variant"), None);
        assert_eq!(widget_prop_state_target_path("widget.text.set"), None);
        assert_eq!(widget_prop_state_target_path("slot.mount.set"), None);
    }
}
