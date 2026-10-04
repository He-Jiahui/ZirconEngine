use super::{EventCursor, EventReadIter};
use crate::scene::ecs::events::Events;

#[test]
fn runtime60_batch_empty_event_read_iterator_stays_exhausted() {
    let mut read = EventReadIter::<u32>::empty();

    assert_eq!(read.next(), None);
    assert_eq!(read.next(), None);
}

#[test]
fn runtime60_batch_partial_event_read_commits_each_yield_exactly_once() {
    let mut events = Events::default();
    events.send_batch([1_u32, 2, 3]);
    events.update();
    let mut cursor = EventCursor::default();

    let mut read = cursor.read(Some(&events));
    assert_eq!(read.next(), Some(&1));
    drop(read);

    assert_eq!(cursor.unread_count(Some(&events)), 2);
}

#[test]
fn runtime60_batch_event_read_iterator_stays_exhausted_after_tail() {
    let mut events = Events::default();
    events.send(7_u32);
    events.update();
    let mut cursor = EventCursor::default();

    let mut read = cursor.read(Some(&events));
    assert_eq!(read.next(), Some(&7));
    assert_eq!(read.next(), None);
    assert_eq!(read.next(), None);
    drop(read);

    assert_eq!(cursor.unread_count(Some(&events)), 0);
}

#[test]
fn bounded_read_only_commits_events_consumed_by_the_iterator() {
    let mut events = Events::default();
    events.send_batch([1_u32, 2, 3, 4]);
    events.update();
    let mut cursor = EventCursor::default();

    let first_page = cursor
        .read(Some(&events))
        .take(2)
        .copied()
        .collect::<Vec<_>>();

    assert_eq!(first_page, [1, 2]);
    assert_eq!(cursor.unread_count(Some(&events)), 2);
    assert_eq!(
        cursor.read(Some(&events)).copied().collect::<Vec<_>>(),
        [3, 4]
    );
    assert_eq!(cursor.unread_count(Some(&events)), 0);
}
