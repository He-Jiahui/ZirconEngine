//! 从路径完整载入并验证会话存档，再投影出路径状态、清单、统计和槽位选择结果。
//! 每次查询都读取当时的文件；结果不保留文件句柄或后续操作的写入权限。

mod manifest;
mod selection;
mod statistics;
mod status;

pub(super) use manifest::{
    contains_slot_from_path, load_manifest_from_path, slot_ids_from_path, slot_summary_from_path,
    slots_matching_display_name_from_path, slots_with_tag_from_path,
};
pub(super) use selection::{
    latest_updated_slot_id_from_path, latest_updated_slot_id_with_tag_from_path,
    oldest_updated_slot_id_from_path, oldest_updated_slot_id_with_tag_from_path,
    select_slot_from_path,
};
pub(super) use statistics::statistics_from_path;
pub(super) use status::inspect_path;
