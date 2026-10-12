// 输入结束时提交最后一行；这是 metadata 对照测试取得完整静态状态列表的边界。
use super::super::storage::CapabilityStatusParserState;

impl CapabilityStatusParserState {
    pub(in super::super::super) fn finish(
        mut self,
    ) -> Vec<zircon_runtime::plugin::CapabilityStatusManifest> {
        self.push_current_status();
        self.statuses
    }
}
