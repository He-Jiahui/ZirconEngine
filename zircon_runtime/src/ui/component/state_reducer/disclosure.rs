//! 折叠类事件需要让快速反馈标志与可投影的 expanded 值一致；写入时清理旧引用来源，避免新布尔值携带旧拖放身份。

use zircon_runtime_interface::ui::component::{UiComponentEventError, UiComponentState, UiValue};

pub(super) fn toggle_expanded(
    state: &mut UiComponentState,
    expanded: bool,
) -> Result<(), UiComponentEventError> {
    state.flags.expanded = expanded;
    super::clear_reference_source(state, "expanded");
    let value = UiValue::Bool(expanded);
    if let Some(existing) = state.values.get_mut("expanded") {
        *existing = value;
    } else {
        state.values.insert("expanded".to_owned(), value);
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/disclosure.rs"]
mod tests;
