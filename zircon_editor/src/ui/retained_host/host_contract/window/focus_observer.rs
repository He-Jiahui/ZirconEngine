use std::cell::RefCell;
use std::rc::Rc;

/// Repeated native-focus observer owned by the window event loop boundary.
#[derive(Default)]
pub(super) struct NativeWindowFocusObserver {
    callback: RefCell<Option<Rc<dyn Fn()>>>,
}

impl NativeWindowFocusObserver {
    pub(super) fn register(
        &self,
        callback: impl Fn() + 'static,
    ) -> Result<(), NativeWindowFocusObserverError> {
        let mut slot = self.callback.borrow_mut();
        if slot.is_some() {
            return Err(NativeWindowFocusObserverError::AlreadyRegistered);
        }
        *slot = Some(Rc::new(callback));
        Ok(())
    }

    pub(super) fn notify(&self) {
        if let Some(callback) = self.callback.borrow().as_ref().cloned() {
            callback();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeWindowFocusObserverError {
    AlreadyRegistered,
}

impl std::fmt::Display for NativeWindowFocusObserverError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a native-window focus observer is already registered")
    }
}

impl std::error::Error for NativeWindowFocusObserverError {}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::NativeWindowFocusObserver;

    #[test]
    fn observer_notifies_every_native_focus_event() {
        let observer = NativeWindowFocusObserver::default();
        let count = Rc::new(Cell::new(0));
        let callback_count = Rc::clone(&count);
        observer
            .register(move || callback_count.set(callback_count.get() + 1))
            .expect("register observer");

        observer.notify();
        observer.notify();

        assert_eq!(count.get(), 2);
    }
}
