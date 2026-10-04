use serde::{Deserialize, Serialize};

use super::UiBindingValue;

/// UI 事件动作的跨宿主载荷：符号标识要调用的动作，arguments 按写入顺序作为调用参数传递。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBindingCall {
    pub symbol: String,
    pub arguments: Vec<UiBindingValue>,
}

impl UiBindingCall {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            arguments: Vec::new(),
        }
    }

    pub fn with_argument(mut self, argument: UiBindingValue) -> Self {
        self.arguments.push(argument);
        self
    }

    pub fn argument(&self, index: usize) -> Option<&UiBindingValue> {
        self.arguments.get(index)
    }

    pub(crate) fn native_repr(&self) -> String {
        let mut output = String::with_capacity(self.symbol.len() + 2);
        self.native_repr_into(&mut output);
        output
    }

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
