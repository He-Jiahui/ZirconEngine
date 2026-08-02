use serde::{Deserialize, Serialize};

use super::{parser::BindingParser, UiBindingCall, UiBindingParseError, UiEventPath};

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

    pub fn native_binding(&self) -> String {
        let prefix = self.path.native_prefix();
        match &self.action {
            Some(action) => format!("{prefix}({})", action.native_repr()),
            None => prefix,
        }
    }

    pub fn parse_native_binding(input: &str) -> Result<Self, UiBindingParseError> {
        BindingParser::new(input).parse_binding()
    }
}
