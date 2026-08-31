use serde::Serialize;

use super::UiProfileFrame;

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileTextRun {
    pub(in crate::ui::retained_host::host_contract) command_index: usize,
    pub(in crate::ui::retained_host::host_contract) frame: UiProfileFrame,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::ui::retained_host::host_contract) clip: Option<UiProfileFrame>,
    pub(in crate::ui::retained_host::host_contract) color: [u8; 4],
    pub(in crate::ui::retained_host::host_contract) font_size: f32,
    pub(in crate::ui::retained_host::host_contract) line_height: f32,
    pub(in crate::ui::retained_host::host_contract) text_length: usize,
}
