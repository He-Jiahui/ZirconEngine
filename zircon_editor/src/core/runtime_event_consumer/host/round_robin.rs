use super::{ActiveConsumerSnapshot, EditorRuntimeEventConsumerHost};

impl EditorRuntimeEventConsumerHost {
    /// 本轮因总预算结束时保存首个未访问者，下一次泵继续公平扫描。
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
    if snapshot_count == 0 || visited_consumer_count == 0 {
        return None;
    }
    Some(visited_consumer_count % snapshot_count)
}

#[cfg(test)]
#[path = "tests/round_robin.rs"]
mod tests;

#[cfg(test)]
#[path = "round_robin/tests/reused_cursor_tests.rs"]
mod reused_cursor_tests;
