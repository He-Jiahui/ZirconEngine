pub(in crate::core::editing::state) fn parse_parent_field(
    value: &str,
) -> Result<Option<zircon_scene::NodeId>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<zircon_scene::NodeId>()
        .map(Some)
        .map_err(|error| format!("Parent field must be a valid node id: {error}"))
}
