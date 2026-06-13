use serde::{Deserialize, Serialize};

use super::InputButton;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputBinding {
    pub action: String,
    pub buttons: Vec<InputButton>,
}

impl InputBinding {
    pub fn button(action: impl Into<String>, button: InputButton) -> Self {
        Self::chord(action, [button])
    }

    pub fn chord(
        action: impl Into<String>,
        buttons: impl IntoIterator<Item = InputButton>,
    ) -> Self {
        let mut buttons = buttons.into_iter().collect::<Vec<_>>();
        buttons.sort();
        buttons.dedup();
        Self {
            action: action.into(),
            buttons,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buttons.is_empty()
    }
}
