use std::path::Path;

use crate::scene::World;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionMetadata, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 每次重载档案后重新解析选择器，将 World 快照写入命中槽位并原子保存完整档案。
    pub fn capture_world_selected_slot_to_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        archive.capture_world_selected_slot(selector, world, metadata)?;
        io::save_to_path_atomically(&archive, path)?;
        archive.manifest()
    }

    /// 每次重载后重新解析选择器，捕获命中槽位时保留原有元数据，再原子保存完整档案。
    pub fn capture_world_selected_slot_preserving_metadata_to_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        world: &World,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        archive.capture_world_selected_slot_preserving_metadata(selector, world)?;
        io::save_to_path_atomically(&archive, path)?;
        archive.manifest()
    }
}
