#[path = "event/declaration.rs"]
mod declaration;
#[path = "event/projection.rs"]
mod projection;
#[path = "event/receiver.rs"]
mod receiver;

pub use declaration::{AssetEvent, AssetEventKind};
pub use receiver::AssetEventReceiver;
pub(crate) use receiver::{typed_event_receiver, AssetEventPoll};

#[cfg(test)]
#[path = "event/tests/cases.rs"]
mod tests;
