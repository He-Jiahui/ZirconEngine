#[path = "recording/model.rs"]
mod model;
#[path = "recording/source_table.rs"]
mod source_table;
#[path = "recording/state.rs"]
mod state;

pub(in crate::ui::retained_host::host_contract) use model::{
    HostPaintAtlasImage, HostPaintImageUvRect, HostRecordedFrame, HostRecordedPaintCommand,
    HostRecordedPaintKind, HostRenderCommandSource,
};
pub(in crate::ui::retained_host::host_contract) use source_table::{
    HostRenderSourceKey, HostRenderSourceTable,
};
pub(in crate::ui::retained_host::host_contract) use state::HostPaintRecording;
