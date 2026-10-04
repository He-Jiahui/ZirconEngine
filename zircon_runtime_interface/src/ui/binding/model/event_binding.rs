use serde::{Deserialize, Serialize};

use super::{parser::BindingParser, UiBindingCall, UiBindingParseError, UiEventPath};

#[cfg(test)]
#[path = "event_binding/tests/native_projection_performance_tests.rs"]
mod native_projection_performance_tests;

#[cfg(test)]
#[path = "event_binding/tests/capacity_performance_tests.rs"]
mod capacity_performance_tests;

/// 将 UI 事件路由地址与可选动作合并为序列化契约；无动作仍可单独订阅或描述事件路径。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiEventBinding {
    pub path: UiEventPath,
    pub action: Option<UiBindingCall>,
}

impl UiEventBinding {
    pub fn new(path: UiEventPath, action: UiBindingCall) -> Self {
        Self {
            path,
            action: Some(action),
        }
    }

    pub fn without_action(path: UiEventPath) -> Self {
        Self { path, action: None }
    }

    /// 生成 Runtime 注册表使用的路由键及可选动作文本；格式与 parse_native_binding 配对。
    pub fn native_binding(&self) -> String {
        let action_capacity = self
            .action
            .as_ref()
            .map_or(0, |action| action.symbol.len() + 4);
        let mut output = String::with_capacity(
            self.path.view_id.len()
                + self.path.control_id.len()
                + self.path.event_kind.native_name().len()
                + 2
                + action_capacity,
        );
        self.path.native_prefix_into(&mut output);
        if let Some(action) = &self.action {
            output.push('(');
            action.native_repr_into(&mut output);
            output.push(')');
        }
        output
    }

    /// 从原生 binding 文本恢复路由和动作，并拒绝不完整或带尾随输入的字符串。
    pub fn parse_native_binding(input: &str) -> Result<Self, UiBindingParseError> {
        BindingParser::new(input).parse_binding()
    }
}
