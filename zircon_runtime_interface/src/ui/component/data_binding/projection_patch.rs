//! 宿主呈现投影的局部更新值，分别保存控件属性与保留状态的自有键值集合。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::component::UiValue;

/// Carries host projection updates while keeping attributes separate from retained state.
/// 适配器完成数据处理后交给宿主的局部呈现增量；此结构只描述投影更新，不执行源数据写入。
///
/// # 使用与生命周期
/// 适配器把此值放入结果交给宿主，供宿主按控件标识更新投影；是否及何时应用由消费端决定。
/// 两个集合分别拥有键和值；同名键可以在两个域中同时出现，不持有控件或源数据的借用。
/// 构建器只修改该值，不查找控件、不执行源数据写入，也不自动设置适配结果的刷新标志。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiComponentProjectionPatch {
    pub control_id: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, UiValue>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub state_values: BTreeMap<String, UiValue>,
}

impl UiComponentProjectionPatch {
    /// 接管控件标识并创建两个空集合；返回自有投影，不验证控件是否存在。
    pub fn new(control_id: impl Into<String>) -> Self {
        Self {
            control_id: control_id.into(),
            attributes: BTreeMap::new(),
            state_values: BTreeMap::new(),
        }
    }

    /// 接管属性键和值并返回投影；同一属性键替换旧值，保留状态集合保持原值。
    pub fn with_attribute(mut self, key: impl Into<String>, value: UiValue) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// 接管保留状态键和值并返回投影；同一状态键替换旧值，属性集合保持原值。
    pub fn with_state_value(mut self, key: impl Into<String>, value: UiValue) -> Self {
        self.state_values.insert(key.into(), value);
        self
    }
}
