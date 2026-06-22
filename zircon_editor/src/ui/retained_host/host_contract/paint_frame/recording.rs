mod model;
mod state;

pub(in crate::ui::retained_host::host_contract) use model::{
    HostPaintAtlasImage, HostPaintImageUvRect, HostRecordedPaintCommand, HostRecordedPaintKind,
};
pub(in crate::ui::retained_host::host_contract) use state::HostPaintRecording;
