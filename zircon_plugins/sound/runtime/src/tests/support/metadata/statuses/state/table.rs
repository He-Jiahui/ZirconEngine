// 每次进入或离开能力状态表都会完成上一行，避免跨表字段污染测试投影。
use super::storage::CapabilityStatusParserState;

impl CapabilityStatusParserState {
    pub(in super::super) fn begin_status_table(&mut self) {
        self.push_current_status();
        self.inside_status = true;
    }

    pub(in super::super) fn leave_status_table(&mut self) {
        self.push_current_status();
        self.inside_status = false;
    }

    pub(in super::super) fn is_inside_status(&self) -> bool {
        self.inside_status
    }
}
