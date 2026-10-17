//! UI 动作符号与有序参数的拥有型载荷。
//!
//! Editor 解码器借用参数，Runtime 调用方可复制或传递完整载荷；此处不解析动作语义。
//! 文本写入复用调用方缓冲区，Serde 的参数反序列化交给值类型自身的校验入口。

use serde::{Deserialize, Serialize};

use super::UiBindingValue;

/// UI 事件动作的跨宿主载荷：符号标识要调用的动作，arguments 按写入顺序作为调用参数传递。
///
/// # 所有权与约束
/// 字符串和参数向量随本对象拥有；构造及追加方法不验证符号语法或跨参数的合并预算。
/// 字段可直接访问，因此调用方提交载荷时仍须满足目标动作的参数协议。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBindingCall {
    /// 由消费方解释的动作符号；构造时按传入内容保存。
    pub symbol: String,
    /// 按动作协议排列的拥有值；顺序参与原生文本投影。
    pub arguments: Vec<UiBindingValue>,
}

impl UiBindingCall {
    /// 接收可转为拥有字符串的动作符号，返回尚无参数的调用载荷。
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            arguments: Vec::new(),
        }
    }

    /// 消费当前载荷与一个参数，按尾部顺序追加后返回载荷；本步骤不校验值预算。
    pub fn with_argument(mut self, argument: UiBindingValue) -> Self {
        self.arguments.push(argument);
        self
    }

    /// 按从零开始的索引借用参数，越界返回 `None`。
    ///
    /// 返回引用的有效期受当前载荷借用约束；需要跨载荷修改或释放保存时由消费方取得拥有值。
    pub fn argument(&self, index: usize) -> Option<&UiBindingValue> {
        self.arguments.get(index)
    }

    /// 测试用的拥有文本包装器，用于与历史分配实现比较输出。
    #[cfg(test)]
    pub(crate) fn native_repr(&self) -> String {
        let mut output = String::with_capacity(self.symbol.len() + 2);
        self.native_repr_into(&mut output);
        output
    }

    /// 把动作及其参数追加到既有缓冲区，不清空原内容，也不取得缓冲区所有权。
    ///
    /// 参数按向量顺序投影；本方法不验证动作是否存在，也不执行动作。
    pub(crate) fn native_repr_into(&self, output: &mut String) {
        output.push_str(&self.symbol);
        output.push('(');
        for (index, argument) in self.arguments.iter().enumerate() {
            if index != 0 {
                output.push(',');
            }
            argument.native_repr_into(output);
        }
        output.push(')');
    }
}
