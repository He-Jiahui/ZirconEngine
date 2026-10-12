use serde::Serialize;

use super::frame::UiProfileFrame;

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileRoundedShape {
    pub(in crate::ui::retained_host::host_contract) command_index: usize,
    pub(in crate::ui::retained_host::host_contract) frame: UiProfileFrame,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::ui::retained_host::host_contract) clip: Option<UiProfileFrame>,
    pub(in crate::ui::retained_host::host_contract) corner_radius: f32,
    pub(in crate::ui::retained_host::host_contract) border_width: f32,
}
