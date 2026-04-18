use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Mobility {
    Dynamic,
    Static,
}

impl Default for Mobility {
    fn default() -> Self {
        Self::Dynamic
    }
}
