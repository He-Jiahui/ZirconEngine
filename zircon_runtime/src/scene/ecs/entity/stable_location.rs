use std::fmt;

use crate::scene::EntityId;

use super::internal::InternalEntity;
use super::location::EntityLocation;

/// 一次解析得到的稳定场景 ID、内部代际句柄和表位置；查询可复用这个快照减少注册表探测。
/// 结构性修改后应重新获取，不能把其中的位置当作长期句柄。
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StableEntityLocation {
    pub(crate) stable_id: EntityId,
    pub(crate) internal: InternalEntity,
    pub(crate) location: EntityLocation,
}

impl fmt::Debug for StableEntityLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StableEntityLocation")
            .field("stable_id", &self.stable_id)
            .field("location", &self.location)
            .finish()
    }
}

impl StableEntityLocation {
    pub const fn stable_id(self) -> EntityId {
        self.stable_id
    }
}
