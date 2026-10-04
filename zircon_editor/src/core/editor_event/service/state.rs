#[derive(Default)]
// 唯一序号与修订分配状态，由服务单独加锁；观察类记录消耗事件序号而不推进编辑修订，日志和监听器持有独立锁。
pub(super) struct EditorEventSequenceState {
    pub(super) next_event_id: u64,
    pub(super) next_sequence: u64,
    pub(super) revision: u64,
}
