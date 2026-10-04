use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;
use zircon_runtime_interface::ui::widget::UI_WIDGET_COMPONENT_ROLE_ATTRIBUTE;

/// 默认交互以显式语义角色为授权依据；组件显示名可被作者包装，不能据名称赋予排序、滚动等行为。
/// 匹配保持精确字符串约定，与展示层宽松的 painter 别名匹配分别承担不同职责。
pub(super) fn component_role(metadata: &UiTemplateNodeMetadata) -> Option<&str> {
    metadata
        .attributes
        .get(UI_WIDGET_COMPONENT_ROLE_ATTRIBUTE)
        .and_then(toml::Value::as_str)
}

pub(super) fn component_role_is(metadata: &UiTemplateNodeMetadata, role: &str) -> bool {
    component_role(metadata) == Some(role)
}

pub(super) fn component_role_is_one_of(metadata: &UiTemplateNodeMetadata, roles: &[&str]) -> bool {
    component_role(metadata).is_some_and(|role| roles.contains(&role))
}

#[cfg(test)]
#[path = "tests/semantics.rs"]
mod tests;
