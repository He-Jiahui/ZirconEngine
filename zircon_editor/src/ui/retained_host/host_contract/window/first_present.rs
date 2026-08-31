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
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::FirstPresentNotification;

    #[test]
    fn notification_invokes_the_registered_callback_once() {
        let notification = FirstPresentNotification::default();
        let count = Rc::new(Cell::new(0));
        let callback_count = Rc::clone(&count);
        notification
            .register(move || {
                callback_count.set(callback_count.get() + 1);
                Ok(())
            })
            .expect("register callback");

        notification.notify().expect("first notification");
        notification.notify().expect("second notification");

        assert_eq!(count.get(), 1);
    }
}
