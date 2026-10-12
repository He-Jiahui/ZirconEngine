use super::*;
use crate::scene::ecs::SystemParamError;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[test]
fn active_state_rejects_other_world_before_tick_callback_and_retirement() {
    let mut original_world = World::empty();
    let mut other_world = World::empty();
    let mut original =
        SystemState::<EventReaderParam<FrameEvent>>::new(&mut original_world).unwrap();
    let mut other = SystemState::<EventReaderParam<FrameEvent>>::new(&mut other_world).unwrap();
    let original_id = original_world.event_type_id::<FrameEvent>().unwrap();
    let other_id = other_world.event_type_id::<FrameEvent>().unwrap();
    let original_tick = original_world.read_change_tick();
    let other_tick = other_world.read_change_tick();
    let last_run = original.last_run();
    let mut callback_entered = false;

    assert!(original.is_bound_to(&original_world));
    assert!(!original.is_bound_to(&other_world));
    assert!(catch_unwind(AssertUnwindSafe(|| {
        original.run(&mut other_world, |_| callback_entered = true);
    }))
    .is_err());
    assert!(!callback_entered);
    assert!(catch_unwind(AssertUnwindSafe(|| original.retire(&mut other_world))).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| original.rebind(&mut other_world))).is_err());
    let mut never_initialized = World::empty();
    assert!(catch_unwind(AssertUnwindSafe(|| {
        original.rebind_to(&mut other_world, &mut never_initialized)
    }))
    .is_err());
    assert!(never_initialized.event_type_id::<FrameEvent>().is_none());
    assert_eq!(original_world.read_change_tick(), original_tick);
    assert_eq!(other_world.read_change_tick(), other_tick);
    assert_eq!(original.last_run(), last_run);
    assert!(!original.is_retired());
    assert_eq!(original_world.event_reader_count(original_id), Some(1));
    assert_eq!(other_world.event_reader_count(other_id), Some(1));

    assert!(original_world.send_event(FrameEvent(11)));
    original_world.update_events::<FrameEvent>();
    let values = original.run(&mut original_world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert_eq!(values, vec![11]);
    original.retire(&mut original_world);
    original.retire(&mut other_world);
    assert!(original.is_retired());
    assert_eq!(original_world.event_reader_count(original_id), Some(0));
    assert_eq!(other_world.event_reader_count(other_id), Some(1));
    other.retire(&mut other_world);
}

#[test]
fn explicit_transfer_retires_original_reader_and_preserves_packet_on_init_error() {
    type Params = (
        EventReaderParam<FrameEvent>,
        ResParam<MissingRetirementResource>,
    );
    let mut original_world = World::empty();
    original_world.insert_resource(MissingRetirementResource);
    let mut other_world = World::empty();
    let mut state = SystemState::<Params>::new(&mut original_world).unwrap();
    let original_id = original_world.event_type_id::<FrameEvent>().unwrap();
    state.run(&mut original_world, |(mut events, _)| {
        assert_eq!(events.iter().count(), 0);
    });
    let last_run = state.last_run();
    let access_before = format!("{:?}", state.access());

    let error = state
        .rebind_to(&mut original_world, &mut other_world)
        .unwrap_err();
    assert_eq!(
        error,
        SystemParamError::TupleElement {
            index: 1,
            parameter_type: std::any::type_name::<ResParam<MissingRetirementResource>>(),
            source: Box::new(SystemParamError::MissingResource {
                type_name: std::any::type_name::<MissingRetirementResource>(),
            }),
        }
    );
    let other_id = other_world.event_type_id::<FrameEvent>().unwrap();
    assert!(state.is_bound_to(&original_world));
    assert!(!state.is_bound_to(&other_world));
    assert!(!state.is_retired());
    assert_eq!(state.last_run(), last_run);
    assert_eq!(format!("{:?}", state.access()), access_before);
    assert_eq!(original_world.event_reader_count(original_id), Some(1));
    assert_eq!(other_world.event_reader_count(other_id), Some(0));
    assert!(original_world.send_event(FrameEvent(17)));
    original_world.update_events::<FrameEvent>();
    assert_eq!(
        state.run(&mut original_world, |(mut events, _)| {
            events.iter().map(|event| event.0).collect::<Vec<_>>()
        }),
        vec![17]
    );

    other_world.insert_resource(MissingRetirementResource);
    state
        .rebind_to(&mut original_world, &mut other_world)
        .unwrap();
    assert_eq!(original_world.event_reader_count(original_id), Some(0));
    assert_eq!(other_world.event_reader_count(other_id), Some(1));
    assert!(!state.is_bound_to(&original_world));
    assert!(state.is_bound_to(&other_world));
    assert!(other_world.send_event(FrameEvent(23)));
    other_world.update_events::<FrameEvent>();
    assert_eq!(
        state.run(&mut other_world, |(mut events, _)| {
            events.iter().map(|event| event.0).collect::<Vec<_>>()
        }),
        vec![23]
    );
    state.retire(&mut other_world);
    assert_eq!(other_world.event_reader_count(other_id), Some(0));
}

#[test]
fn active_same_world_rebuild_and_retired_transfer_keep_one_reader_lease() {
    let mut world = World::empty();
    let mut state = SystemState::<EventReaderParam<FrameEvent>>::new(&mut world).unwrap();
    let type_id = world.event_type_id::<FrameEvent>().unwrap();
    state.run(&mut world, |mut events| {
        assert_eq!(events.iter().count(), 0)
    });
    state.rebind(&mut world).unwrap();
    assert!(state.is_bound_to(&world));
    assert!(!state.is_retired());
    assert_eq!(world.event_reader_count(type_id), Some(1));
    state.retire(&mut world);
    assert_eq!(world.event_reader_count(type_id), Some(0));
    let mut other = World::empty();
    state.rebind(&mut other).unwrap();
    let other_id = other.event_type_id::<FrameEvent>().unwrap();
    assert!(state.is_bound_to(&other));
    assert_eq!(other.event_reader_count(other_id), Some(1));
    state.retire(&mut other);
    assert_eq!(other.event_reader_count(other_id), Some(0));
}
