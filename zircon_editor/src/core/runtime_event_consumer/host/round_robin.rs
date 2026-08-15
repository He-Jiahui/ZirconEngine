use super::{ActiveConsumerSnapshot, EditorRuntimeEventConsumerHost};

impl EditorRuntimeEventConsumerHost {
    pub(super) fn advance_round_robin_start(
        &self,
        snapshots: &[ActiveConsumerSnapshot],
        visited_consumer_count: usize,
    ) {
        let Some(next) = next_start_index(snapshots.len(), visited_consumer_count)
            .and_then(|index| snapshots.get(index))
            .map(|snapshot| snapshot.consumer_id.clone())
        else {
            return;
        };
        *self
            .round_robin_cursor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(next);
    }
}

fn next_start_index(snapshot_count: usize, visited_consumer_count: usize) -> Option<usize> {
    (snapshot_count != 0 && visited_consumer_count != 0)
        .then_some(visited_consumer_count % snapshot_count)
}

#[cfg(test)]
mod tests {
    use super::next_start_index;

    #[test]
    fn global_budget_starts_next_pump_at_first_unvisited_consumer() {
        assert_eq!(next_start_index(4, 3), Some(3));
        assert_eq!(next_start_index(4, 4), Some(0));
    }
}
