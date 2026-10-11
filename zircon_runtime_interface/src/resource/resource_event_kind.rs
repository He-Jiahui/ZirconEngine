use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 表示资源提交后的目录净变化，供订阅者选择刷新入口。
///
/// # 变体与消费
///
/// Added、Removed 分别表示记录出现与消失，地址变化使用 Renamed，其它记录变化使用 Updated，已标记的失败变化使用 ReloadFailed。仅替换载荷或驻留状态而记录未变时，不必产生目录事件。
///
/// # 时间边界
///
/// 此标签不是有序游标或完整状态；日志处理顺序与缺口，消费方再查询需要的当前记录。
///
/// # 参考
///
/// [`crate::resource::ResourceEvent`]、[`crate::resource::ResourceRecord`]。
pub enum ResourceEventKind {
    Added,
    Updated,
    Removed,
    Renamed,
    ReloadFailed,
}
