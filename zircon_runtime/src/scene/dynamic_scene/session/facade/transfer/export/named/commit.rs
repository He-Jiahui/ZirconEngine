use super::super::super::super::super::{
    slot_export, RuntimeSessionArchive, RuntimeSessionArchiveError,
};

impl RuntimeSessionArchive {
    /// 克隆指定槽形成独立归档，保留槽身份、场景和元数据；源归档不移除该槽，新归档重建代际及发布谱系。
    pub fn single_slot_archive(&self, slot_id: &str) -> Result<Self, RuntimeSessionArchiveError> {
        slot_export::single_slot_archive(self, slot_id)
    }
}
