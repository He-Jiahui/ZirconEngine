#[path = "codec.rs"]
mod codec;
#[path = "command.rs"]
mod command;

pub use command::{inspector_field_control_id, DraftCommand};
