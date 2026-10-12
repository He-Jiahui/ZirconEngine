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
#[path = "tests/focus_observer.rs"]
mod tests;
