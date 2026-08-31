use crate::scene::ecs::events::metrics::{
    EventCapacityMetrics, EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES,
};

const EVENT_CAPACITY_LOW_WATER_DIVISOR: usize = 4;

#[derive(Clone, Debug)]
pub struct Events<T> {
    current: Vec<T>,
    next: Vec<T>,
    generation: u64,
    high_water_len: usize,
    low_water_frames: u32,
    capacity_shrink_count: u64,
}

impl<T> PartialEq for Events<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.current == other.current && self.next == other.next
    }
}

impl<T> Eq for Events<T> where T: Eq {}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Self {
            current: Vec::new(),
            next: Vec::new(),
            generation: 0,
            high_water_len: 0,
            low_water_frames: 0,
            capacity_shrink_count: 0,
        }
    }
}

impl<T> Events<T> {
    pub fn send(&mut self, event: T) {
        self.next.push(event);
        self.record_next_queue_len();
    }

    pub fn send_batch<I>(&mut self, events: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        let before = self.next.len();
        self.next.extend(events);
        let written = self.next.len() - before;
        if written > 0 {
            self.record_next_queue_len();
        }
        written
    }

    pub fn update(&mut self) {
        self.current.clear();
        std::mem::swap(&mut self.current, &mut self.next);
        self.generation = self.generation.saturating_add(1);
        self.update_capacity_policy();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.current.iter()
    }

    pub fn iter_from(&self, start: usize) -> std::slice::Iter<'_, T> {
        self.current[start.min(self.current.len())..].iter()
    }

    pub fn drain(&mut self) -> Vec<T> {
        std::mem::take(&mut self.current)
    }

    pub fn clear(&mut self) {
        self.current.clear();
        self.next.clear();
        self.generation = self.generation.saturating_add(1);
        self.high_water_len = 0;
        self.low_water_frames = 0;
    }

    pub fn len(&self) -> usize {
        self.current.len()
    }

    pub fn is_empty(&self) -> bool {
        self.current.is_empty()
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub fn capacity_metrics(&self) -> EventCapacityMetrics {
        EventCapacityMetrics {
            current_len: self.current.len(),
            next_len: self.next.len(),
            current_capacity: self.current.capacity(),
            next_capacity: self.next.capacity(),
            high_water_len: self.high_water_len,
            low_water_frames: self.low_water_frames,
            shrink_count: self.capacity_shrink_count,
        }
    }

    pub(crate) fn requires_maintenance(&self) -> bool {
        if !self.current.is_empty() || !self.next.is_empty() {
            return true;
        }
        let retained_capacity = self.current.capacity().max(self.next.capacity());
        retained_capacity > 0 && self.low_water_frames < EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES
    }

    fn record_next_queue_len(&mut self) {
        self.high_water_len = self.high_water_len.max(self.next.len());
        self.low_water_frames = 0;
    }

    fn update_capacity_policy(&mut self) {
        let active_len = self.current.len().max(self.next.len());
        self.high_water_len = self.high_water_len.max(active_len);
        self.reserve_next_for_high_water();

        let retained_capacity = self.current.capacity().max(self.next.capacity());
        if retained_capacity == 0 {
            self.high_water_len = 0;
            self.low_water_frames = 0;
            return;
        }

        let low_water_threshold = (retained_capacity / EVENT_CAPACITY_LOW_WATER_DIVISOR).max(1);
        if active_len > low_water_threshold {
            self.low_water_frames = 0;
            return;
        }

        self.low_water_frames = self.low_water_frames.saturating_add(1);
        if self.low_water_frames < EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES {
            return;
        }

        if self.shrink_buffers_to(active_len) {
            self.capacity_shrink_count = self.capacity_shrink_count.saturating_add(1);
        }
        self.high_water_len = active_len;
        self.low_water_frames = 0;
    }

    fn reserve_next_for_high_water(&mut self) {
        if self.high_water_len == 0 || self.next.capacity() >= self.high_water_len {
            return;
        }
        self.next
            .reserve_exact(self.high_water_len - self.next.capacity());
    }

    fn shrink_buffers_to(&mut self, target_capacity: usize) -> bool {
        let before = self.current.capacity().max(self.next.capacity());
        let current_target_capacity = target_capacity.max(self.current.len());
        let next_target_capacity = target_capacity.max(self.next.len());
        Self::shrink_vec_to(&mut self.current, current_target_capacity);
        Self::shrink_vec_to(&mut self.next, next_target_capacity);
        let after = self.current.capacity().max(self.next.capacity());
        after < before
    }

    fn shrink_vec_to(vec: &mut Vec<T>, target_capacity: usize) {
        if vec.capacity() <= target_capacity {
            return;
        }
        let mut replacement = Vec::with_capacity(target_capacity.max(vec.len()));
        replacement.extend(vec.drain(..));
        *vec = replacement;
    }
}

#[cfg(test)]
mod tests {
    use super::Events;

    #[test]
    fn runtime60_batch_specialized_batch_extend_preserves_order_and_count() {
        let mut events = Events::default();

        assert_eq!(events.send_batch(10_u32..14), 4);
        events.update();

        assert_eq!(events.iter().copied().collect::<Vec<_>>(), [10, 11, 12, 13]);
    }

    #[test]
    fn runtime60_batch_specialized_batch_extend_accepts_non_exact_iterators() {
        let mut events = Events::default();

        assert_eq!(
            events.send_batch((0_u32..8).filter(|value| value % 2 == 0)),
            4
        );
        events.update();

        assert_eq!(events.iter().copied().collect::<Vec<_>>(), [0, 2, 4, 6]);
    }

    #[test]
    fn runtime60_batch_empty_batch_does_not_raise_the_high_water_mark() {
        let mut events = Events::<u32>::default();

        assert_eq!(events.send_batch(std::iter::empty()), 0);

        let metrics = events.capacity_metrics();
        assert_eq!(metrics.next_len, 0);
        assert_eq!(metrics.high_water_len, 0);
    }

    #[test]
    fn event_queue_equality_ignores_reader_generation_metadata() {
        let mut first = Events::<u32>::default();
        let mut second = Events::<u32>::default();

        first.update();
        first.update();

        assert_eq!(first, second);

        first.send(5);
        second.send(5);

        assert_eq!(first, second);
    }
}
