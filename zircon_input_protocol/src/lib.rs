//! Shared input protocol types for manager traits, runtime input handling, and entry bridges.

mod input_button;
mod input_event;
mod input_event_record;
mod input_snapshot;

pub use input_button::InputButton;
pub use input_event::InputEvent;
pub use input_event_record::InputEventRecord;
pub use input_snapshot::InputSnapshot;
