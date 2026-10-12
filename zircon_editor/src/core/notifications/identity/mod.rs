#[path = "error.rs"]
mod error;
#[path = "id.rs"]
mod id;
#[path = "source.rs"]
mod source;

pub use error::NotificationIdentityError;
pub use id::{NotificationId, MAX_NOTIFICATION_ID_BYTES};
pub use source::{NotificationSource, NotificationSourceKind, MAX_NOTIFICATION_SOURCE_ID_BYTES};
