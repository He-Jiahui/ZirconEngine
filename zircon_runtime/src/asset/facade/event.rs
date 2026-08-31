mod declaration;
mod projection;
mod receiver;

pub use declaration::{AssetEvent, AssetEventKind};
pub use receiver::AssetEventReceiver;
pub(crate) use receiver::{typed_event_receiver, AssetEventPoll};

#[cfg(test)]
mod tests;
