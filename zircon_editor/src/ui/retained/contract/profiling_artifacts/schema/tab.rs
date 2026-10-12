use serde::Serialize;

use super::frame::UiProfileFrame;

#[derive(Clone, Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileTabFrame {
    pub(in crate::ui::retained_host::host_contract) id: String,
    pub(in crate::ui::retained_host::host_contract) title: String,
    pub(in crate::ui::retained_host::host_contract) kind: String,
    pub(in crate::ui::retained_host::host_contract) surface: String,
    pub(in crate::ui::retained_host::host_contract) frame: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) close_frame: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) active: bool,
}
