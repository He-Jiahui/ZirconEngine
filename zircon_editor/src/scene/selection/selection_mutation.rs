use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionMutation {
    #[default]
    Replace,
    Extend,
    Toggle,
}

impl SelectionMutation {
    pub fn from_modifier_flags(shift: bool, control: bool) -> Self {
        if control {
            Self::Toggle
        } else if shift {
            Self::Extend
        } else {
            Self::Replace
        }
    }
}
