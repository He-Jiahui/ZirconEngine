//! 记录组件写入，交由 World 的刷新入口收敛派生状态与渲染脏标记。
use std::any::TypeId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::scene::ecs::Component;
use crate::scene::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ComponentMutationRecord {
    entity: EntityId,
    component_type: TypeId,
    component_type_name: &'static str,
}

impl ComponentMutationRecord {
    pub(crate) const fn entity(self) -> EntityId {
        self.entity
    }

    pub(crate) const fn component_type(self) -> TypeId {
        self.component_type
    }

    pub(crate) const fn component_type_name(self) -> &'static str {
        self.component_type_name
    }
}

#[derive(Debug, Default)]
pub(crate) struct ComponentMutationSink {
    records: Mutex<Vec<ComponentMutationRecord>>,
    pending_count: AtomicU64,
}

impl Clone for ComponentMutationSink {
    fn clone(&self) -> Self {
        let records = self
            .records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        Self {
            pending_count: AtomicU64::new(records.len() as u64),
            records: Mutex::new(records),
        }
    }
}

impl ComponentMutationSink {
    pub(crate) fn recorder<T>(&self, entity: EntityId) -> ComponentMutationRecorder<'_>
    where
        T: Component,
    {
        ComponentMutationRecorder {
            sink: self,
            record: ComponentMutationRecord {
                entity,
                component_type: TypeId::of::<T>(),
                component_type_name: std::any::type_name::<T>(),
            },
        }
    }

    pub(crate) fn drain(&self) -> Vec<ComponentMutationRecord> {
        // World::flush_deferred_component_mutations 在派生系统的脏状态检查前排空，再把记录转成具体脏标记。
        let records = std::mem::take(
            &mut *self
                .records
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        self.pending_count.store(0, Ordering::Relaxed);
        records
    }

    pub(crate) fn pending_count(&self) -> u64 {
        self.pending_count.load(Ordering::Relaxed)
    }

    fn record(&self, record: ComponentMutationRecord) {
        self.records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(record);
        let _ = self
            .pending_count
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                Some(count.saturating_add(1))
            });
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ComponentMutationRecorder<'world> {
    sink: &'world ComponentMutationSink,
    record: ComponentMutationRecord,
}

impl ComponentMutationRecorder<'_> {
    pub(crate) fn record(self) {
        self.sink.record(self.record);
    }
}
