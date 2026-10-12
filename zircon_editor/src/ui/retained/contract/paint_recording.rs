#[path = "paint_recording/damage.rs"]
mod damage;
#[path = "paint_recording/record.rs"]
mod record;

pub(in crate::ui::retained_host::host_contract) use record::record_host_frame_commands;
