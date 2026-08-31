use super::{ActiveConsumerSnapshot, EditorRuntimeEventConsumerHost};

impl EditorRuntimeEventConsumerHost {
    pub(super) fn advance_round_robin_start(
        &self,
        snapshots: &[ActiveConsumerSnapshot],
        visited_consumer_count: usize,
    ) {
        let Some(next) = next_start_index(snapshots.len(), visited_consumer_count)
            .and_then(|index| snapshots.get(index))
            .map(|snapshot| snapshot.consumer_id.as_str())
        else {
            return;
        };
        let mut cursor = self
            .round_robin_cursor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        update_round_robin_cursor(&mut *cursor, next);
    }
}

fn update_round_robin_cursor(cursor: &mut Option<String>, next: &str) {
    match cursor {
        Some(current) if current.as_str() == next => {}
        Some(current) => {
            current.clear();
            current.push_str(next);
        }
        None => *cursor = Some(next.to_owned()),
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

#[cfg(test)]
#[path = "round_robin/reused_cursor_tests.rs"]
mod reused_cursor_tests;
