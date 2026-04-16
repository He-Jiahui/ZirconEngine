use serde::{Deserialize, Serialize};

use super::UiBindingValue;

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
        format!(
            "{}({})",
            self.symbol,
            self.arguments
                .iter()
                .map(UiBindingValue::native_repr)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
