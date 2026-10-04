mod center;
mod error;
mod model;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(crate) use center::AUTOMATIC_PROGRESS_SOURCE_ID;
pub use center::{
    ProgressNotificationCenter, ProgressNotificationSnapshot, MAX_PROGRESS_NOTIFICATIONS,
};
pub use error::ProgressNotificationError;
pub use model::ProgressNotification;
