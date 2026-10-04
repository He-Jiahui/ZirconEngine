// 仅有 capability 的行会进入比较结果；调用方的固定清单保证该字段存在且状态可解析。
use super::super::storage::CapabilityStatusParserState;
use super::{manifest, required};

impl CapabilityStatusParserState {
    pub(in super::super) fn push_current_status(&mut self) {
        let Some(capability) = self.current_capability.take() else {
            return;
        };
        self.statuses.push(manifest::capability_status_manifest(
            capability,
            required::take_required_capability_status(&mut self.current_status),
            self.current_bevy_references.drain(..),
        ));
    }
}
