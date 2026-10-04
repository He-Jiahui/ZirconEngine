use super::super::super::{RuntimeSessionArchive, RuntimeSessionSlot};

// 可选式删除供裁剪和普通移除共用；不存在时返回空值，存在时返回完整槽位供调用者继续使用。
// 此低层入口不做档案整体预检；需要诊断无效档案的流程先走预览或载入验证。
pub(in crate::scene::dynamic_scene::session) fn remove_slot(
    archive: &mut RuntimeSessionArchive,
    slot_id: &str,
) -> Option<RuntimeSessionSlot> {
    archive.remove_indexed_slot(slot_id)
}
