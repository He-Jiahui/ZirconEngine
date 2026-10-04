use std::cell::RefCell;

/// One-shot observer for a verified native-surface submission.
///
/// The event loop invokes this only from its successful presenter branch. Registration is kept at
/// the window boundary so startup protocol code cannot infer a present from window construction.
#[derive(Default)]
pub(super) struct FirstPresentNotification {
    callback: RefCell<Option<Box<dyn FnOnce() -> Result<(), String>>>>,
}

impl FirstPresentNotification {
    pub(super) fn register(
        &self,
        callback: impl FnOnce() -> Result<(), String> + 'static,
    ) -> Result<(), FirstPresentNotificationError> {
        let mut slot = self.callback.borrow_mut();
        if slot.is_some() {
            return Err(FirstPresentNotificationError::AlreadyRegistered);
        }
        *slot = Some(Box::new(callback));
        Ok(())
    }

    pub(super) fn notify(&self) -> Result<(), String> {
        let callback = self.callback.borrow_mut().take();
        match callback {
            Some(callback) => callback(),
            None => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FirstPresentNotificationError {
    AlreadyRegistered,
}

impl std::fmt::Display for FirstPresentNotificationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a first-present notification is already registered")
    }
}

impl std::error::Error for FirstPresentNotificationError {}

#[cfg(test)]
#[path = "tests/first_present.rs"]
mod tests;
