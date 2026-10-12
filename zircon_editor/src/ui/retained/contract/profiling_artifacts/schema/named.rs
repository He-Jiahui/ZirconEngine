use serde::Serialize;

use super::frame::UiProfileFrame;
use super::tab::UiProfileTabFrame;

#[derive(Clone, Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileNamedFrame {
    pub(in crate::ui::retained_host::host_contract) id: String,
    pub(in crate::ui::retained_host::host_contract) kind: String,
    pub(in crate::ui::retained_host::host_contract) surface: String,
    pub(in crate::ui::retained_host::host_contract) frame: UiProfileFrame,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::ui::retained_host::host_contract) clip: Option<UiProfileFrame>,
}

impl UiProfileNamedFrame {
    pub(in crate::ui::retained_host::host_contract) fn from_tab(tab: &UiProfileTabFrame) -> Self {
        Self {
            id: tab.id.clone(),
            kind: tab.kind.clone(),
            surface: tab.surface.clone(),
            frame: tab.frame.clone(),
            clip: None,
        }
    }
}
