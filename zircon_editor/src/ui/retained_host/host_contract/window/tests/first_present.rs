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
