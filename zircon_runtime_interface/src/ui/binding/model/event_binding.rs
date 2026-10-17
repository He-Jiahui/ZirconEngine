//! 事件路由地址与可选动作载荷的公共组合契约。
//!
//! 模板、Editor 和 Runtime 传递拥有内容的绑定对象；本模块提供构造、文本投影和解析，
//! 处理器注册、查找与执行仍由 Runtime 事件管理器负责。解析成功的对象不借用输入文本。

use serde::{Deserialize, Serialize};

use super::{parser::BindingParser, UiBindingCall, UiBindingParseError, UiEventPath};

#[cfg(test)]
#[path = "tests/native_projection_perf.rs"]
mod native_projection_performance_tests;

#[cfg(test)]
#[path = "tests/capacity_perf.rs"]
mod capacity_performance_tests;

/// 将 UI 事件路由地址与可选动作合并为序列化契约；无动作仍可单独订阅或描述事件路径。
///
/// # 状态与生命周期
/// 路径始终存在；动作缺省时只保留事件地址。两个字段均拥有内容，可脱离解析输入保存。
/// 公开字段允许手工组装；构造方法不检查路径语法、动作符号或整组参数的资源预算。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiEventBinding {
    /// 拥有视图、控件标识与事件种类的路由地址。
    pub path: UiEventPath,
    /// 可选的动作符号与有序参数；缺省状态不表示执行了任何动作。
    pub action: Option<UiBindingCall>,
}

impl UiEventBinding {
    /// 将已经构造的路径与动作移入一个带动作的绑定。
    ///
    /// # 参数与返回
    /// `path`、`action` 的所有权转入结果；本方法只组合载荷，不注册或执行处理器。
    pub fn new(path: UiEventPath, action: UiBindingCall) -> Self {
        Self {
            path,
            action: Some(action),
        }
    }

    /// 移入路径并返回无动作的绑定，供只描述或订阅事件地址的调用方使用。
    pub fn without_action(path: UiEventPath) -> Self {
        Self { path, action: None }
    }

    /// 生成 Runtime 注册表使用的路由键及可选动作文本；格式与 parse_native_binding 配对。
    ///
    /// # 返回与约束
    /// 返回独立拥有的文本；容量只作预估，参数写入时仍可能扩容。
    /// 路径和动作符号按原文写入，不在这里转义或验证；需要重新解析时须符合对应语法。
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
    ///
    /// # 参数与返回
    /// `input` 仅在本次解析中借用；成功返回拥有路径、符号及参数的绑定。
    /// 语法、构造器或参数值校验失败以此错误类型返回；多字节空白的已知游标缺陷见解析器标记。
    ///
    /// # 约束
    /// 解析不查找处理器；参数逐值校验，未合并计算整个参数向量的预算。
    pub fn parse_native_binding(input: &str) -> Result<Self, UiBindingParseError> {
        BindingParser::new(input).parse_binding()
    }
}
