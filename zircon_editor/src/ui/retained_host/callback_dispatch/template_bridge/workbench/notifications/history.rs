use super::entry_unread;

pub(super) struct RetainedNotificationHistory {
    pub entries: Vec<String>,
    pub unread_count: i64,
    pub overflow_count: i64,
}

impl RetainedNotificationHistory {
    pub(super) fn merge(
        incoming: impl IntoIterator<Item = String>,
        existing: impl IntoIterator<Item = String>,
        capacity: usize,
        previous_overflow_count: i64,
    ) -> Self {
        let mut entries = Vec::with_capacity(capacity);
        let mut unread_count = 0_i64;
        let mut candidate_count = 0_i64;

        for entry in incoming.into_iter().chain(existing) {
            candidate_count = candidate_count.saturating_add(1);
            if entries.len() >= capacity {
                continue;
            }
            if entry_unread(&entry) {
                unread_count = unread_count.saturating_add(1);
            }
            entries.push(entry);
        }

        let dropped_count = candidate_count.saturating_sub(entries.len() as i64);
        Self {
            entries,
            unread_count,
            overflow_count: previous_overflow_count.max(0).saturating_add(dropped_count),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burst_retention_is_bounded_counted_and_newest_first() {
        let incoming = (0..1_000)
            .rev()
            .map(|index| format!("notification-{index}|title=Entry {index}|unread=true"));

        let retained = RetainedNotificationHistory::merge(incoming, [], 64, 0);

        assert_eq!(retained.entries.len(), 64);
        assert!(retained.entries[0].starts_with("notification-999|"));
        assert!(retained.entries[63].starts_with("notification-936|"));
        assert_eq!(retained.unread_count, 64);
        assert_eq!(retained.overflow_count, 936);
    }

    #[test]
    fn later_batches_preserve_order_and_accumulate_explicit_overflow() {
        let existing = (0..64).map(|index| format!("old-{index}|unread=false"));
        let retained =
            RetainedNotificationHistory::merge(["new|unread=true".to_string()], existing, 64, 936);

        assert_eq!(retained.entries.len(), 64);
        assert_eq!(retained.entries[0], "new|unread=true");
        assert_eq!(retained.entries[63], "old-62|unread=false");
        assert_eq!(retained.unread_count, 1);
        assert_eq!(retained.overflow_count, 937);
    }
}
