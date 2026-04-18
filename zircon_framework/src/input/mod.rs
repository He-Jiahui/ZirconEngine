use zircon_core::ChannelReceiver;

mod input_button;
mod input_event;
mod input_event_record;
mod input_snapshot;

pub use input_button::InputButton;
pub use input_event::InputEvent;
pub use input_event_record::InputEventRecord;
pub use input_snapshot::InputSnapshot;

pub trait InputManager: Send + Sync {
    fn submit_event(&self, event: InputEvent);
    fn snapshot(&self) -> InputSnapshot;
    fn drain_events(&self) -> Vec<InputEvent>;
    fn drain_event_records(&self) -> Vec<InputEventRecord>;

    fn subscribe_events(&self) -> Option<ChannelReceiver<InputEventRecord>> {
        None
    }
}
