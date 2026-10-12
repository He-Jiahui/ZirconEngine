// 状态只属于静态清单测试扫描器；切换表时提交当前能力，最终由 metadata 用例与运行时类型比较。
// Keeps capability-status row finalization tied to the static TOML table scanner.
#[derive(Default)]
pub(in super::super) struct CapabilityStatusParserState {
    pub(in super::super) statuses: Vec<zircon_runtime::plugin::CapabilityStatusManifest>,
    pub(in super::super) current_capability: Option<String>,
    pub(in super::super) current_status: Option<zircon_runtime::plugin::CapabilityStatus>,
    pub(in super::super) current_bevy_references: Vec<String>,
    pub(in super::super) inside_status: bool,
}
