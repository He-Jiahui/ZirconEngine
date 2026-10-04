use std::path::Path;

use crate::scene::World;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionMetadata,
};

impl RuntimeSessionArchive {
    /// 从路径读取（缺失时视为空档案），将当前 World 快照写入指定槽位并原子保存；返回保存后的清单。
    pub fn capture_world_slot_to_path_atomically(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        archive.capture_world_slot(slot_id, world, metadata)?;
        io::save_to_path_atomically(&archive, path)?;
        archive.manifest()
    }
}
