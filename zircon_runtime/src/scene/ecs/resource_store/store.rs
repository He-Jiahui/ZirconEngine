use std::any::{type_name, TypeId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;

use crate::scene::ecs::{ChangeTick, ComponentTicks};

use super::stored_resource::{StoredResource, TransferredResourceRow};

#[derive(Default)]
/// 按 Rust TypeId 拥有运行时资源值与变更刻度；World 的资源写入口负责登记类型并记录写入。
pub struct ResourceStore {
    resources: HashMap<TypeId, StoredResource>,
}

impl ResourceStore {
    pub fn insert<T: 'static + Send + Sync>(&mut self, resource: T) -> Option<T> {
        self.insert_at_tick(resource, ChangeTick::INITIAL)
    }

    pub fn insert_at_tick<T: 'static + Send + Sync>(
        &mut self,
        resource: T,
        tick: ChangeTick,
    ) -> Option<T> {
        let type_id = TypeId::of::<T>();
        match self.resources.entry(type_id) {
            Entry::Occupied(mut occupied) => {
                let stored = occupied.get_mut();
                stored.ticks.set_changed(tick);
                let previous = std::mem::replace(&mut stored.value, Box::new(resource));
                let Ok(boxed) = previous.downcast::<T>() else {
                    return None;
                };
                Some(*boxed)
            }
            Entry::Vacant(vacant) => {
                vacant.insert(StoredResource {
                    value: Box::new(resource),
                    type_name: type_name::<T>(),
                    ticks: ComponentTicks::new(tick),
                });
                None
            }
        }
    }

    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        let stored = self.resources.get(&TypeId::of::<T>())?;
        stored.value.downcast_ref::<T>()
    }

    /// 借出值本身，不标记变更；需要检测写入时应使用 World 的资源写入口，或通过带 ticks 的借用自行记录。
    pub fn get_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        let stored = self.resources.get_mut(&TypeId::of::<T>())?;
        stored.value.downcast_mut::<T>()
    }

    pub fn get_mut_with_ticks<T: 'static + Send + Sync>(
        &mut self,
    ) -> Option<(&mut T, &mut ComponentTicks)> {
        let stored = self.resources.get_mut(&TypeId::of::<T>())?;
        let StoredResource { value, ticks, .. } = stored;
        let value = value.downcast_mut::<T>()?;
        Some((value, ticks))
    }

    pub fn remove<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        let stored = self.resources.remove(&TypeId::of::<T>())?;
        let Ok(boxed) = stored.value.downcast::<T>() else {
            return None;
        };
        Some(*boxed)
    }

    pub fn contains<T: 'static + Send + Sync>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    pub fn ticks<T: 'static + Send + Sync>(&self) -> Option<ComponentTicks> {
        let stored = self.resources.get(&TypeId::of::<T>())?;
        Some(stored.ticks)
    }

    pub fn len(&self) -> usize {
        self.resources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    pub fn clear(&mut self) {
        self.resources.clear();
    }

    pub fn type_names(&self) -> Vec<&'static str> {
        let mut names = Vec::with_capacity(self.resources.len());
        for stored in self.resources.values() {
            names.push(stored.type_name);
        }
        names.sort_unstable();
        names
    }

    pub(in crate::scene) fn merge_overrides_from(&mut self, overrides: Self) -> Self {
        let mut replaced = HashMap::with_capacity(overrides.resources.len());
        for (type_id, resource) in overrides.resources {
            if let Some(previous) = self.resources.insert(type_id, resource) {
                replaced.insert(type_id, previous);
            }
        }
        Self {
            resources: replaced,
        }
    }

    /// Detaches only resources already selected into an isolated artifact.
    /// The caller chooses the subset before this method is reached.
    pub(crate) fn take_transferred_rows(&mut self) -> Vec<TransferredResourceRow> {
        let resources = std::mem::take(&mut self.resources);
        let mut rows = Vec::with_capacity(resources.len());
        for (type_id, resource) in resources {
            rows.push(TransferredResourceRow {
                type_id,
                value: resource.value,
                type_name: resource.type_name,
                source_ticks: resource.ticks,
            });
        }
        rows
    }

    /// Publishes preflight-owned resource rows with the target World's change
    /// tick. No reflection adapter runs in this live-store operation.
    pub(crate) fn insert_transferred_rows(
        &mut self,
        rows: Vec<TransferredResourceRow>,
        tick: ChangeTick,
    ) {
        for row in rows {
            self.resources.insert(
                row.type_id,
                StoredResource {
                    value: row.value,
                    type_name: row.type_name,
                    ticks: ComponentTicks::new(tick),
                },
            );
        }
    }
}

impl fmt::Debug for ResourceStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceStore")
            .field("type_names", &self.type_names())
            .finish()
    }
}

// World::clone 调用此实现隔离运行时资源；返回空存储，不复制资源值或变更刻度。
impl Clone for ResourceStore {
    fn clone(&self) -> Self {
        Self::default()
    }
}

// 此比较忽略资源值与刻度，因此 World 的派生比较也不能证明资源内容相同。
impl PartialEq for ResourceStore {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[cfg(test)]
#[path = "tests/store.rs"]
mod tests;
