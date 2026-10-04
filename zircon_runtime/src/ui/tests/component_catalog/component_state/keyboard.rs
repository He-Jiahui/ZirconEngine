//! 键盘动作测试按控件族分组，固定语义动作进入组件归约器后的焦点、选择和文本所有权边界。

use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentKeyboardAction, UiComponentState, UiValue,
};

mod action_selection;
mod menu_navigation;
mod numeric_controls;
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
