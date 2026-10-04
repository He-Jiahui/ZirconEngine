pub(super) fn widget_prop_state_target_path(action_id: &str) -> Option<&str> {
    let target_path = action_id.strip_suffix(".set")?;
    if target_path.starts_with("widget.prop.") || target_path.starts_with("widget.state.") {
        Some(target_path)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/ui_asset_editor_detail_routes.rs"]
mod tests;
