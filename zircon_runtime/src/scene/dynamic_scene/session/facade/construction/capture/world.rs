use crate::scene::World;

use super::super::super::super::construction;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn from_world(
        slot_id: impl Into<String>,
        world: &World,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        construction::from_world(slot_id, world)
    }

    /// 捕获一个 World 为独立会话档案，并附带项目/显示信息；捕获失败不会返回部分档案。
    pub fn from_world_with_metadata(
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        construction::from_world_with_metadata(slot_id, world, metadata)
    }
}
