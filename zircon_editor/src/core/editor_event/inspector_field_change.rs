use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::binding::UiBindingValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 一次检查器编辑中的字段值请求；字段标识与绑定值要由目标 subject 的绑定层解释和验证。
pub struct InspectorFieldChange {
    pub field_id: String,
    pub value: UiBindingValue,
}
