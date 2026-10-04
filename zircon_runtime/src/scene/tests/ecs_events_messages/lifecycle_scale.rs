use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::scene::ecs::{
    EventReaderParam, EventWriterParam, Message, MessageReaderParam, MessageRetention,
    MessageWriterParam, SystemStage, SystemState, EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES,
};
use crate::scene::World;

use super::{FrameEvent, IdleEvent};

#[derive(Default)]
struct MillionMessageCounters {
    live: AtomicUsize,
    peak_live: AtomicUsize,
    dropped: AtomicUsize,
}

struct MillionRetainedMessage {
    ordinal: u32,
    payload: Box<[u8; 16]>,
    counters: Arc<MillionMessageCounters>,
}

impl MillionRetainedMessage {
    fn new(ordinal: u32, counters: &Arc<MillionMessageCounters>) -> Self {
        let payload = Box::new([ordinal as u8; 16]);
        let live = counters.live.fetch_add(1, Ordering::SeqCst) + 1;
        counters.peak_live.fetch_max(live, Ordering::SeqCst);
        Self {
            ordinal,
            payload,
            counters: Arc::clone(counters),
        }
    }
}

impl Message for MillionRetainedMessage {
    fn retained_byte_size(&self) -> usize {
        self.payload.len()
    }
}

impl Drop for MillionRetainedMessage {
    fn drop(&mut self) {
        self.counters.live.fetch_sub(1, Ordering::SeqCst);
        self.counters.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn million_event_burst_retires_payload_and_capacity_before_idle_frames() {
    for writes in [0_u32, 1, 1_000_000] {
        let mut world = World::empty();
        let mut writer = SystemState::<EventWriterParam<FrameEvent>>::new(&mut world).unwrap();
        let mut reader = SystemState::<EventReaderParam<FrameEvent>>::new(&mut world).unwrap();
        world.register_event::<IdleEvent>();
        world.update_all_events();
        assert_eq!(world.event_store().last_update_channel_visits(), 0);
        assert_eq!(
            writer.run(&mut world, |mut events| {
                events.send_batch((0..writes).map(FrameEvent))
            }),
            writes as usize
        );
        assert!(reader.run(&mut world, |mut events| events.iter().next().is_none()));
        world.update_all_events();
        assert_eq!(
            world.event_store().last_update_channel_visits(),
            usize::from(writes > 0)
        );
        assert_eq!(world.events::<FrameEvent>().unwrap().len(), writes as usize);
        reader.run(&mut world, |mut events| {
            let delivered = events
                .iter()
                .enumerate()
                .map(|(ordinal, event)| assert_eq!(event.0, ordinal as u32))
                .count();
            assert_eq!(delivered, writes as usize);
            assert!(events.iter().next().is_none());
        });
        for _ in 0..=EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES {
            world.update_all_events();
        }
        let capacity = world.events::<FrameEvent>().unwrap().capacity_metrics();
        assert_eq!(capacity.queued_len(), 0);
        assert_eq!(capacity.current_capacity, 0);
        assert_eq!(capacity.next_capacity, 0);
        assert_eq!(capacity.shrink_count > 0, writes > 0);
        for idle_frame in 0..10_000 {
            world.run_internal_scene_systems_for_stage(SystemStage::First);
            assert_eq!(
                world.event_store().last_update_channel_visits(),
                0,
                "writes {writes}, idle frame {idle_frame}"
            );
        }
    }
}

#[test]
fn million_messages_enforce_entry_byte_age_and_slow_reader_boundaries() {
    const RETAINED: usize = 8;
    const PAYLOAD_BYTES: usize = 16;
    for writes in [0, 1, 1_000_000] {
        for retention in [
            MessageRetention::new(RETAINED, usize::MAX, 1),
            MessageRetention::new(RETAINED * 2, RETAINED * PAYLOAD_BYTES, 1),
        ] {
            assert_million_message_lifecycle(writes, retention, RETAINED, PAYLOAD_BYTES);
        }
    }
}

fn assert_million_message_lifecycle(
    writes: usize,
    retention: MessageRetention,
    retention_limit: usize,
    payload_bytes: usize,
) {
    let retained_count = writes.min(retention_limit);
    let mut world = World::empty();
    world.configure_message_retention::<MillionRetainedMessage>(retention);
    let mut writer =
        SystemState::<MessageWriterParam<MillionRetainedMessage>>::new(&mut world).unwrap();
    let mut active_reader =
        SystemState::<MessageReaderParam<MillionRetainedMessage>>::new(&mut world).unwrap();
    let mut dormant_reader =
        SystemState::<MessageReaderParam<MillionRetainedMessage>>::new(&mut world).unwrap();
    let counters = Arc::new(MillionMessageCounters::default());

    world.run_internal_scene_systems_for_stage(SystemStage::First);
    world.run_internal_scene_systems_for_stage(SystemStage::First);
    assert_eq!(world.last_message_advance_channel_visits(), 0);
    let ids = writer.run(&mut world, |mut messages| {
        messages.write_batch(
            (0..writes as u32).map(|ordinal| MillionRetainedMessage::new(ordinal, &counters)),
        )
    });
    assert_eq!(ids.len(), writes);
    assert!(ids
        .iter()
        .enumerate()
        .all(|(ordinal, id)| id.id() == ordinal));
    drop(ids);

    let metrics = world
        .message_retention_metrics::<MillionRetainedMessage>()
        .unwrap();
    assert_eq!(metrics.retained_entries, retained_count);
    assert_eq!(metrics.retained_bytes, retained_count * payload_bytes);
    assert_eq!(
        metrics.budget_dropped_entries,
        (writes - retained_count) as u64
    );
    assert_eq!(
        metrics.budget_dropped_bytes,
        ((writes - retained_count) * payload_bytes) as u64
    );
    assert_eq!(metrics.age_dropped_entries, 0);
    assert_eq!(counters.live.load(Ordering::SeqCst), retained_count);
    assert!(counters.peak_live.load(Ordering::SeqCst) <= retained_count + usize::from(writes > 0));
    assert_eq!(
        counters.dropped.load(Ordering::SeqCst),
        writes - retained_count
    );

    let retained = active_reader.run(&mut world, |mut messages| {
        assert_eq!(messages.dropped_count(), 0);
        let retained = messages
            .read()
            .map(|(id, message)| (id.id(), message.ordinal, message.payload[0]))
            .collect::<Vec<_>>();
        assert_eq!(messages.dropped_count(), (writes - retained_count) as u64);
        retained
    });
    assert_eq!(
        retained,
        (writes - retained_count..writes)
            .map(|ordinal| (ordinal, ordinal as u32, ordinal as u8))
            .collect::<Vec<_>>()
    );

    world.run_internal_scene_systems_for_stage(SystemStage::First);
    assert_eq!(world.last_message_advance_channel_visits(), 1);
    assert_eq!(
        world
            .message_retention_metrics::<MillionRetainedMessage>()
            .unwrap()
            .retained_entries,
        retained_count
    );
    world.run_internal_scene_systems_for_stage(SystemStage::First);
    assert_eq!(
        world.last_message_advance_channel_visits(),
        usize::from(writes > 0)
    );
    let retired = world
        .message_retention_metrics::<MillionRetainedMessage>()
        .unwrap();
    assert_eq!(retired.retained_entries, 0);
    assert_eq!(retired.retained_bytes, 0);
    assert_eq!(
        retired.budget_dropped_entries,
        (writes - retained_count) as u64
    );
    assert_eq!(retired.age_dropped_entries, retained_count as u64);
    assert_eq!(
        retired.age_dropped_bytes,
        (retained_count * payload_bytes) as u64
    );
    assert_eq!(counters.live.load(Ordering::SeqCst), 0);
    assert_eq!(counters.dropped.load(Ordering::SeqCst), writes);
    dormant_reader.run(&mut world, |mut messages| {
        assert!(messages.read().next().is_none());
        assert_eq!(messages.dropped_count(), writes as u64);
        assert!(messages.read().next().is_none());
        assert_eq!(messages.dropped_count(), writes as u64);
    });
    active_reader.run(&mut world, |mut messages| {
        assert!(messages.read().next().is_none());
        assert_eq!(messages.dropped_count(), (writes - retained_count) as u64);
    });

    for idle_frame in 0..10_000 {
        world.run_internal_scene_systems_for_stage(SystemStage::First);
        assert_eq!(
            world.last_message_advance_channel_visits(),
            0,
            "idle frame {idle_frame}"
        );
    }
}
