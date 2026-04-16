use serde::{Deserialize, Serialize};

use super::UiEventKind;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiEventPath {
    pub view_id: String,
    pub control_id: String,
    pub event_kind: UiEventKind,
}

impl UiEventPath {
    pub fn new(
        view_id: impl Into<String>,
        control_id: impl Into<String>,
        event_kind: UiEventKind,
    ) -> Self {
        Self {
            view_id: view_id.into(),
            control_id: control_id.into(),
            event_kind,
        }
    }

    pub fn native_prefix(&self) -> String {
        format!(
            "{}/{}:{}",
            self.view_id,
            self.control_id,
            self.event_kind.native_name()
        )
    }
}
