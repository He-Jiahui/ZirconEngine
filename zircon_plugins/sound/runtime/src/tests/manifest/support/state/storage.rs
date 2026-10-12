// 仅保存当前静态表行及已完成结果；表边界负责提交，最终结果交给包清单对照用例。
use super::super::super::StaticDependency;

// Keeps dependency-row finalization tied to the static TOML table scanner.
#[derive(Default)]
pub(in super::super) struct DependencyParserState {
    pub(in super::super) dependencies: Vec<StaticDependency>,
    pub(in super::super) current_id: Option<String>,
    pub(in super::super) current_required: Option<bool>,
    pub(in super::super) current_capability: Option<String>,
    pub(in super::super) inside_dependency: bool,
}
