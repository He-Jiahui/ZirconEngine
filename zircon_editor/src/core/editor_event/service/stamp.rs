use crate::core::editor_event::{EditorEventId, EditorEventSequence};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 一次派发在执行前分配的关联信息，成功与失败记录均使用同一组编号；修订范围不等价于事务提交成功。
pub(crate) struct EditorEventStamp {
    pub(crate) event_id: EditorEventId,
    pub(crate) sequence: EditorEventSequence,
    pub(crate) before_revision: u64,
    pub(crate) after_revision: u64,
}
