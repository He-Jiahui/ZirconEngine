use super::super::super::query as session_query;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn slot_count(&self) -> usize {
        session_query::slot_count(self)
    }

    pub fn is_empty(&self) -> bool {
        session_query::is_empty(self)
    }

    pub fn contains_slot(&self, slot_id: &str) -> bool {
        session_query::contains_slot(self, slot_id)
    }

    pub fn slot(&self, slot_id: &str) -> Option<&RuntimeSessionSlot> {
        session_query::slot(self, slot_id)
    }

    /// 以规范 ID 顺序读取当前代际的槽位；适合清单和稳定报告，不暴露内部密集行顺序。
    pub fn slots(&self) -> impl Iterator<Item = &RuntimeSessionSlot> {
        session_query::slots(self)
    }

    pub fn slot_ids(&self) -> impl Iterator<Item = &str> {
        session_query::slot_ids(self)
    }
}
