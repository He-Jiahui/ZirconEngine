//! 将共享事件绑定的原生文本格式集中到无状态入口。
//! 格式化与解析均委托绑定契约；此处不持有路由表或解析会话。

use crate::ui::binding::{UiBindingParseError, UiEventBinding};

/// 供调用方显式选择原生绑定文本格式的无状态门面。
///
/// 返回的绑定或字符串拥有数据，不借用本类型；语法和错误由共享绑定实现决定。
#[derive(Clone, Debug, Default)]
pub struct UiBindingCodec;

impl UiBindingCodec {
    /// 借用 `binding` 生成拥有的原生路由及可选动作文本。
    ///
    /// 返回值沿用绑定实现的格式约定；不会查找或注册对应的事件处理器。
    pub fn format(binding: &UiEventBinding) -> String {
        binding.native_binding()
    }

    /// 解析借用的原生文本，返回拥有的绑定或共享解析器的错误。
    ///
    /// 输入仅需在调用期间有效；完整语法及尾随输入的准入沿用绑定解析实现。
    pub fn parse(input: &str) -> Result<UiEventBinding, UiBindingParseError> {
        UiEventBinding::parse_native_binding(input)
    }
}
