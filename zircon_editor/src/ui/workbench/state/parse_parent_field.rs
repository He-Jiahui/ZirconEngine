use super::InspectorEditError;

/// 空白表示移到根节点；这里只解析身份，目标存在性及父子合法性由场景命令验证。
pub(super) fn parse_parent_field(
    value: &str,
) -> Result<Option<zircon_runtime::scene::NodeId>, InspectorEditError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<zircon_runtime::scene::NodeId>()
        .map(Some)
        .map_err(|_| InspectorEditError::InvalidParentField {
            value: value.to_string(),
        })
}
