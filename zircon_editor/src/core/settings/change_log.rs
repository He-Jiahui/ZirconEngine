use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::{SettingsKey, SettingsScope};

pub const DEFAULT_SETTINGS_CHANGE_LOG_MAX_ENTRIES: usize = 4_096;
pub const DEFAULT_SETTINGS_CHANGE_LOG_MAX_BYTES: usize = 256 * 1024;
pub const DEFAULT_SETTINGS_CHANGE_LOG_MAX_AGE: Duration = Duration::from_secs(300);

#[derive(Clone, Debug, PartialEq)]
pub struct SettingChange {
    pub key: SettingsKey,
    pub scope: SettingsScope,
    pub revision: u64,
    pub requires_restart: bool,
}

/// A consumer position in the bounded settings-change history.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SettingsChangeCursor {
    revision: u64,
}

impl SettingsChangeCursor {
    pub const fn origin() -> Self {
        Self { revision: 0 }
    }

    pub const fn revision(self) -> u64 {
        self.revision
    }

    pub(super) const fn at(revision: u64) -> Self {
        Self { revision }
    }
}

/// A bounded delta; consumers refresh their immutable settings snapshot when history was evicted.
#[derive(Clone, Debug, PartialEq)]
pub struct SettingsChangeDelta {
    pub changes: Vec<SettingChange>,
    pub cursor: SettingsChangeCursor,
    pub requires_snapshot: bool,
}

/// Retention limits for the transient settings change log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsChangeLogPolicy {
    max_entries: usize,
    max_bytes: usize,
    max_age: Duration,
}

impl SettingsChangeLogPolicy {
    pub const fn new(max_entries: usize, max_bytes: usize, max_age: Duration) -> Self {
        Self {
            max_entries,
            max_bytes,
            max_age,
        }
    }
}

impl Default for SettingsChangeLogPolicy {
    fn default() -> Self {
        Self::new(
            DEFAULT_SETTINGS_CHANGE_LOG_MAX_ENTRIES,
            DEFAULT_SETTINGS_CHANGE_LOG_MAX_BYTES,
            DEFAULT_SETTINGS_CHANGE_LOG_MAX_AGE,
        )
    }
}

#[derive(Clone)]
struct RetainedSettingChange {
    change: SettingChange,
    retained_at: Instant,
    bytes: usize,
}

/// The registry owns this log so all consumers share one bounded notification history.
#[derive(Clone, Default)]
pub(super) struct SettingsChangeLog {
    policy: SettingsChangeLogPolicy,
    entries: VecDeque<RetainedSettingChange>,
    retained_bytes: usize,
}

impl SettingsChangeLog {
    pub(super) fn with_policy(policy: SettingsChangeLogPolicy) -> Self {
        Self {
            policy,
            entries: VecDeque::new(),
            retained_bytes: 0,
        }
    }

    pub(super) fn record(&mut self, change: SettingChange) {
        let now = Instant::now();
        self.expire_before(now);
        let bytes = setting_change_bytes(&change);
        self.entries.push_back(RetainedSettingChange {
            change,
            retained_at: now,
            bytes,
        });
        self.retained_bytes = self.retained_bytes.saturating_add(bytes);
        self.enforce_budget();
    }

    pub(super) fn delta_since(
        &mut self,
        cursor: SettingsChangeCursor,
        latest_revision: u64,
    ) -> SettingsChangeDelta {
        self.expire_before(Instant::now());
        let requires_snapshot = if cursor.revision > latest_revision {
            true
        } else if let Some(entry) = self.entries.front() {
            cursor.revision.saturating_add(1) < entry.change.revision
        } else {
            cursor.revision < latest_revision
        };
        let first_change_index = if cursor.revision >= latest_revision {
            self.entries.len()
        } else {
            self.entries.front().map_or(0, |entry| {
                let next_revision = cursor.revision.saturating_add(1);
                let offset = next_revision.saturating_sub(entry.change.revision);
                usize::try_from(offset)
                    .unwrap_or(usize::MAX)
                    .min(self.entries.len())
            })
        };
        let changes = self
            .entries
            .range(first_change_index..)
            .map(|entry| entry.change.clone())
            .collect();
        SettingsChangeDelta {
            changes,
            cursor: SettingsChangeCursor::at(latest_revision),
            requires_snapshot,
        }
    }

    fn expire_before(&mut self, now: Instant) {
        while self.entries.front().is_some_and(|entry| {
            now.saturating_duration_since(entry.retained_at) >= self.policy.max_age
        }) {
            self.pop_front();
        }
    }

    fn enforce_budget(&mut self) {
        while self.entries.len() > self.policy.max_entries
            || self.retained_bytes > self.policy.max_bytes
        {
            self.pop_front();
        }
    }

    fn pop_front(&mut self) {
        if let Some(entry) = self.entries.pop_front() {
            self.retained_bytes = self.retained_bytes.saturating_sub(entry.bytes);
        }
    }
}

fn setting_change_bytes(change: &SettingChange) -> usize {
    std::mem::size_of::<SettingChange>().saturating_add(change.key.as_str().len())
}
