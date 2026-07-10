use serde::{Deserialize, Serialize};

use crate::core::editor_message::SelectionDomain;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusMessage {
    SelectionChanged {
        domain: SelectionDomain,
        revision: u64,
    },
    FocusObject {
        entity: u64,
    },
}
