use super::super::command::ChromeCommand;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::HostRenderSourceTable;

pub(in crate::ui::retained_host::host_contract) struct ChromeCommandExtraction {
    pub(in crate::ui::retained_host::host_contract) commands: Vec<ChromeCommand>,
    pub(in crate::ui::retained_host::host_contract) clipped_damage: Option<FrameRect>,
    pub(in crate::ui::retained_host::host_contract) render_sources: HostRenderSourceTable,
}
