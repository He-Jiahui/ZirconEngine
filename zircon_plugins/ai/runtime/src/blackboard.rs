#[path = "blackboard/layout.rs"]
mod layout;
#[path = "blackboard/observer.rs"]
mod observer;
#[path = "blackboard/store.rs"]
mod store;

pub use layout::{BlackboardLayout, BlackboardLayoutError, BlackboardSlot};
pub(crate) use observer::{BlackboardObserver, BlackboardObserverSet};
pub use store::{BlackboardRuntimeError, BlackboardStore, BlackboardWriteOutcome};
