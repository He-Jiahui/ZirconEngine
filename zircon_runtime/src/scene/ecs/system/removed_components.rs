use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTickWindow, Component, RemovedComponentEvents, RemovedComponentReadIter,
    RemovedComponentReader, SystemParam, SystemParamAccess, SystemParamError,
};
use crate::scene::{EntityId, World};

/// 为每个系统保留独立移除事件游标；World 的保留窗口结束后可由 dropped_count 发现漏读。
pub struct RemovedComponentsParam<T>(PhantomData<fn() -> T>);

pub struct RemovedComponents<'world, T>
where
    T: Component,
{
    events: &'world RemovedComponentEvents,
    reader: &'world mut RemovedComponentReader<T>,
}

impl<'world, T> RemovedComponents<'world, T>
where
    T: Component,
{
    pub fn read(&mut self) -> RemovedComponentReadIter<'_, 'world, T> {
        self.reader.read(self.events)
    }

    pub fn len(&self) -> usize {
        self.reader.len(self.events)
    }

    pub fn is_empty(&self) -> bool {
        self.reader.is_empty(self.events)
    }

    pub fn clear(&mut self) {
        self.reader.clear(self.events);
    }

    pub fn dropped_count(&self) -> u64 {
        self.reader.dropped_count()
    }
}

impl<T> SystemParam for RemovedComponentsParam<T>
where
    T: Component,
{
    type State = RemovedComponentReader<T>;
    type Item<'world> = RemovedComponents<'world, T>;

    fn init_state(
        _world: &mut World,
        _access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        Ok(RemovedComponentReader::default())
    }

    // TODO: [CR-SCENE-ECS-0004] 核实与同一 tuple 中可变 World 参数共存时，从原始 World 指针建立共享引用的别名安全前提；SystemParam 契约尚未写明。
    unsafe fn get_param<'world>(
        world: *mut World,
        state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &*world;
        RemovedComponents {
            events: world.removed_component_events(),
            reader: state,
        }
    }
}
