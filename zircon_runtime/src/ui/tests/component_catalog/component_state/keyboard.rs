//! 键盘动作测试按控件族分组，固定语义动作进入组件归约器后的焦点、选择和文本所有权边界。

use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentKeyboardAction, UiComponentState, UiValue,
};

#[path = "../../component_state/keyboard/action_selection.rs"]
mod action_selection;
#[path = "../../component_state/keyboard/menu_navigation.rs"]
mod menu_navigation;
#[path = "../../component_state/keyboard/numeric_controls.rs"]
mod numeric_controls;
#[path = "../../component_state/keyboard/text_inputs.rs"]
mod text_inputs;

fn menu_option(id: &str, label: &str) -> UiValue {
    UiValue::Map(
        [
            ("id".to_string(), UiValue::String(id.to_string())),
            ("label".to_string(), UiValue::String(label.to_string())),
        ]
        .into_iter()
        .collect(),
    )
}
