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
