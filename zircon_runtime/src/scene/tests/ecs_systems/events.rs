use super::*;

#[test]
fn event_reader_and_writer_use_current_and_next_queues() {
    let mut world = World::empty();
    type Writer = EventWriterParam<HitEvent>;
    type Reader = EventReaderParam<HitEvent>;

    let mut writer = SystemState::<Writer>::new(&mut world).unwrap();
    writer.run(&mut world, |mut events| events.send(HitEvent(3)));

    let mut reader = SystemState::<Reader>::new(&mut world).unwrap();
    let before_update = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert!(before_update.is_empty());

    world.update_events::<HitEvent>();
    let after_update = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert_eq!(after_update, vec![3]);
}

#[test]
fn event_writer_batch_preserves_next_queue_order() {
    let mut world = World::empty();
    type Writer = EventWriterParam<HitEvent>;
    type Reader = EventReaderParam<HitEvent>;

    let mut writer = SystemState::<Writer>::new(&mut world).unwrap();
    let first_count = writer.run(&mut world, |mut events| {
        events.send_batch([HitEvent(1), HitEvent(2), HitEvent(3)])
    });
    let second_count = writer.run(&mut world, |mut events| {
        events.send_batch([HitEvent(4), HitEvent(5)])
    });
    assert_eq!((first_count, second_count), (3, 2));

    let mut reader = SystemState::<Reader>::new(&mut world).unwrap();
    let before_update = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert!(before_update.is_empty());

    world.update_events::<HitEvent>();
    let observed = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert_eq!(observed, vec![1, 2, 3, 4, 5]);
}

#[test]
fn event_reader_param_keeps_cursor_between_system_runs() {
    let mut world = World::empty();
    type Writer = EventWriterParam<HitEvent>;
    type Reader = EventReaderParam<HitEvent>;

    let mut writer = SystemState::<Writer>::new(&mut world).unwrap();
    let mut reader = SystemState::<Reader>::new(&mut world).unwrap();

    writer.run(&mut world, |mut events| {
        events.send(HitEvent(1));
        events.send(HitEvent(2));
    });
    world.update_events::<HitEvent>();

    let first = reader.run(&mut world, |mut events| {
        assert_eq!(events.unread_count(), 2);
        assert_eq!(events.len(), 2);
        assert!(!events.is_empty());
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    let repeated = reader.run(&mut world, |mut events| {
        assert_eq!(events.unread_count(), 0);
        assert_eq!(events.len(), 0);
        assert!(events.is_empty());
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });

    writer.run(&mut world, |mut events| events.send(HitEvent(3)));
    world.update_events::<HitEvent>();
    let next_frame = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });

    writer.run(&mut world, |mut events| events.send(HitEvent(4)));
    world.update_events::<HitEvent>();
    let cleared = reader.run(&mut world, |mut events| {
        assert_eq!(events.unread_count(), 1);
        events.clear();
        assert!(events.is_empty());
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });

    assert_eq!(first, vec![1, 2]);
    assert!(repeated.is_empty());
    assert_eq!(next_frame, vec![3]);
    assert!(cleared.is_empty());
}

#[test]
fn event_reader_param_observes_events_after_global_clear() {
    let mut world = World::empty();
    type Writer = EventWriterParam<HitEvent>;
    type Reader = EventReaderParam<HitEvent>;

    let mut writer = SystemState::<Writer>::new(&mut world).unwrap();
    let mut reader = SystemState::<Reader>::new(&mut world).unwrap();

    writer.run(&mut world, |mut events| {
        events.send(HitEvent(1));
        events.send(HitEvent(2));
    });
    world.update_events::<HitEvent>();

    let first = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert_eq!(first, vec![1, 2]);

    world.clear_events::<HitEvent>();
    writer.run(&mut world, |mut events| events.send(HitEvent(3)));
    world.update_events::<HitEvent>();

    let after_clear = reader.run(&mut world, |mut events| {
        assert_eq!(events.unread_count(), 1);
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert_eq!(after_clear, vec![3]);
}
