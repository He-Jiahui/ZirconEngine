use crate::core::framework::events::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone, Copy, Default)]
pub(super) struct Retained {
    pub events: usize,
    pub bytes: usize,
}
impl Retained {
    pub(super) fn add(self, events: usize, bytes: usize) -> Option<Self> {
        Some(Self {
            events: self.events.checked_add(events)?,
            bytes: self.bytes.checked_add(bytes)?,
        })
    }
    pub(super) fn subtract(self, events: usize, bytes: usize) -> Option<Self> {
        Some(Self {
            events: self.events.checked_sub(events)?,
            bytes: self.bytes.checked_sub(bytes)?,
        })
    }
    pub(super) fn fits(self, limits: EventRetentionLimits) -> bool {
        self.events <= limits.max_events.get() && self.bytes <= limits.max_bytes.get()
    }
}
pub(super) struct SubscriberAccount {
    pub topic: Arc<str>,
    pub retained: Retained,
    pub active: bool,
}
pub(super) struct TopicAccount {
    pub retained: Retained,
    pub active: usize,
}
pub(super) struct AdmissionLedger {
    pub limits: EventBusLimits,
    pub closed: bool,
    pub poisoned: bool,
    pub retained: Retained,
    pub peak: Retained,
    pub active_subscribers: usize,
    pub topics: HashMap<String, TopicAccount>,
    pub subscribers: HashMap<u64, SubscriberAccount>,
}
pub(super) struct SharedAdmission {
    inner: Mutex<AdmissionLedger>,
    commit: Mutex<()>,
}

impl SharedAdmission {
    pub(super) fn new(limits: EventBusLimits) -> Self {
        Self {
            commit: Mutex::new(()),
            inner: Mutex::new(AdmissionLedger {
                limits,
                closed: false,
                poisoned: false,
                retained: Retained::default(),
                peak: Retained::default(),
                active_subscribers: 0,
                topics: HashMap::new(),
                subscribers: HashMap::new(),
            }),
        }
    }
    pub(super) fn lock_commit(&self) -> MutexGuard<'_, ()> {
        match self.commit.lock() {
            Ok(commit) => commit,
            Err(poisoned) => {
                self.fail_closed();
                poisoned.into_inner()
            }
        }
    }
    pub(super) fn lock(&self) -> MutexGuard<'_, AdmissionLedger> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                guard.closed = true;
                guard.poisoned = true;
                guard
            }
        }
    }
    pub(super) fn fail_closed(&self) {
        let mut ledger = self.lock();
        ledger.closed = true;
        ledger.poisoned = true;
    }
    pub(super) fn snapshot(&self) -> EventBusRetentionSnapshot {
        self.lock().snapshot()
    }
    pub(super) fn register(&self, id: u64, topic: &str) -> Result<(), EngineEventSubscribeError> {
        let mut ledger = self.lock();
        if ledger.poisoned {
            return Err(EngineEventSubscribeError::Poisoned);
        }
        if ledger.closed {
            return Err(EngineEventSubscribeError::Closed);
        }
        if topic.is_empty() || topic.len() > ledger.limits.max_topic_name_bytes.get() {
            return Err(EngineEventSubscribeError::InvalidTopic);
        }
        if ledger.active_subscribers >= ledger.limits.max_subscribers.get()
            || (!ledger.topics.contains_key(topic)
                && ledger.topics.len() >= ledger.limits.max_topics.get())
        {
            return Err(EngineEventSubscribeError::RegistryFull);
        }
        ledger
            .topics
            .entry(topic.into())
            .or_insert(TopicAccount {
                retained: Retained::default(),
                active: 0,
            })
            .active += 1;
        ledger.subscribers.insert(
            id,
            SubscriberAccount {
                topic: topic.into(),
                retained: Retained::default(),
                active: true,
            },
        );
        ledger.active_subscribers += 1;
        Ok(())
    }
    pub(super) fn unregister(&self, id: u64) {
        let mut ledger = self.lock();
        let Some(account) = ledger.subscribers.get_mut(&id) else {
            return;
        };
        if !account.active {
            return;
        }
        account.active = false;
        let topic_name = account.topic.clone();
        let Some(active) = ledger.active_subscribers.checked_sub(1) else {
            ledger.closed = true;
            ledger.poisoned = true;
            return;
        };
        ledger.active_subscribers = active;
        let Some(topic) = ledger.topics.get_mut(topic_name.as_ref()) else {
            ledger.closed = true;
            ledger.poisoned = true;
            return;
        };
        let Some(active) = topic.active.checked_sub(1) else {
            ledger.closed = true;
            ledger.poisoned = true;
            return;
        };
        topic.active = active;
        ledger.prune(id, &topic_name);
    }
}

impl AdmissionLedger {
    pub(super) fn snapshot(&self) -> EventBusRetentionSnapshot {
        EventBusRetentionSnapshot {
            closed: self.closed,
            poisoned: self.poisoned,
            retained_events: self.retained.events,
            retained_bytes: self.retained.bytes,
            peak_retained_events: self.peak.events,
            peak_retained_bytes: self.peak.bytes,
        }
    }
    pub(super) fn charge(&mut self, id: u64, events: usize, bytes: usize) -> bool {
        let Some(account) = self.subscribers.get(&id) else {
            self.closed = true;
            self.poisoned = true;
            return false;
        };
        let topic_name = account.topic.clone();
        let next_account = account.retained.add(events, bytes);
        let next_topic = self
            .topics
            .get(topic_name.as_ref())
            .and_then(|a| a.retained.add(events, bytes));
        let next_global = self.retained.add(events, bytes);
        let (Some(account), Some(topic), Some(global)) = (next_account, next_topic, next_global)
        else {
            self.closed = true;
            self.poisoned = true;
            return false;
        };
        self.subscribers.get_mut(&id).unwrap().retained = account;
        self.topics.get_mut(topic_name.as_ref()).unwrap().retained = topic;
        self.retained = global;
        self.peak.events = self.peak.events.max(global.events);
        self.peak.bytes = self.peak.bytes.max(global.bytes);
        true
    }
    pub(super) fn release(&mut self, id: u64, events: usize, bytes: usize) -> bool {
        let Some(account) = self.subscribers.get(&id) else {
            self.closed = true;
            self.poisoned = true;
            return false;
        };
        let topic_name = account.topic.clone();
        let next_account = account.retained.subtract(events, bytes);
        let next_topic = self
            .topics
            .get(topic_name.as_ref())
            .and_then(|a| a.retained.subtract(events, bytes));
        let next_global = self.retained.subtract(events, bytes);
        let (Some(account), Some(topic), Some(global)) = (next_account, next_topic, next_global)
        else {
            self.closed = true;
            self.poisoned = true;
            return false;
        };
        self.subscribers.get_mut(&id).unwrap().retained = account;
        self.topics.get_mut(topic_name.as_ref()).unwrap().retained = topic;
        self.retained = global;
        self.prune(id, &topic_name);
        true
    }
    fn prune(&mut self, id: u64, topic_name: &str) {
        if self
            .subscribers
            .get(&id)
            .is_some_and(|a| !a.active && a.retained.events == 0)
        {
            self.subscribers.remove(&id);
        }
        if self
            .topics
            .get(topic_name)
            .is_some_and(|a| a.active == 0 && a.retained.events == 0)
        {
            self.topics.remove(topic_name);
        }
    }
}

pub(super) struct RetentionLease {
    admission: Arc<SharedAdmission>,
    id: u64,
    bytes: usize,
    charged: AtomicBool,
}
impl RetentionLease {
    pub(super) fn new(admission: Arc<SharedAdmission>, id: u64, bytes: usize) -> Self {
        Self {
            admission,
            id,
            bytes,
            charged: AtomicBool::new(false),
        }
    }
}
impl EngineEventRetentionLease for RetentionLease {
    fn activate_charge(&self) {
        self.charged.store(true, Ordering::Release);
    }
}
impl Drop for RetentionLease {
    fn drop(&mut self) {
        if self.charged.swap(false, Ordering::AcqRel) {
            self.admission.lock().release(self.id, 1, self.bytes);
        }
    }
}

#[cfg(test)]
#[path = "admission/tests/cases.rs"]
mod tests;
