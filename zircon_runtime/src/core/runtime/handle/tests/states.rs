use std::panic::{self, AssertUnwindSafe};

use crate::core::runtime::state_machine::NextState;
use crate::core::CoreRuntime;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
enum StateFixture {
    #[default]
    Boot,
    Running,
}

#[test]
fn core_handle_state_accessors_recover_poisoned_state_registry_lock() {
    let runtime = CoreRuntime::new();
    let handle = runtime.handle();

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = handle.inner.states.lock().unwrap();
        panic!("poison core handle state registry");
    }));

    let init_event = handle.init_state::<StateFixture>();
    assert_eq!(init_event.entered, Some(StateFixture::Boot));
    assert_eq!(
        handle.state::<StateFixture>().unwrap().into_inner(),
        StateFixture::Boot
    );

    handle.set_next_state(StateFixture::Running);
    assert_eq!(
        handle.next_state::<StateFixture>(),
        NextState::Pending(StateFixture::Running)
    );

    let transition = handle.apply_state_transition::<StateFixture>().unwrap();
    assert_eq!(transition.exited, Some(StateFixture::Boot));
    assert_eq!(transition.entered, Some(StateFixture::Running));
    assert_eq!(
        handle.state::<StateFixture>().unwrap().into_inner(),
        StateFixture::Running
    );

    let latest = handle.latest_state_transition::<StateFixture>();
    assert_eq!(latest, Some(transition));

    handle.reset_next_state::<StateFixture>();
    assert_eq!(handle.next_state::<StateFixture>(), NextState::Unchanged);
}

#[test]
fn existing_state_init_reuses_the_registry_lock() {
    let source = include_str!("../states.rs");
    let start = source.find("pub fn init_state").expect("init state method");
    let end = source[start..]
        .find("pub fn insert_state")
        .map(|offset| start + offset)
        .expect("insert state method");
    let implementation = &source[start..end];

    assert!(implementation.contains("let mut states = self.lock_states();"));
    assert!(implementation.contains("states.state::<T>()"));
    assert!(!implementation.contains("self.state::<T>()"));
}
