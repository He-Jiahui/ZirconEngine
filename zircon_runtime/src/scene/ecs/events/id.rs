/// 可通过 EventStore 分发的类型；发送时的同步观察者与帧事件队列共享同一注册类型。
pub trait Event: 'static + Send + Sync {}

impl<T> Event for T where T: 'static + Send + Sync {}

/// EventStore 内部的类型槽位编号；数值只在创建它的 Store 中有意义。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventTypeId(u32);

impl EventTypeId {
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}
