mod model;
mod source_table;
mod state;

pub(in crate::ui::retained_host::host_contract) use model::{
    HostPaintAtlasImage, HostPaintImageUvRect, HostRecordedFrame, HostRecordedPaintCommand,
    HostRecordedPaintKind, HostRenderCommandSource,
};
pub(in crate::ui::retained_host::host_contract) use source_table::{
    HostRenderSourceKey, HostRenderSourceTable,
};
pub(in crate::ui::retained_host::host_contract) use state::HostPaintRecording;
