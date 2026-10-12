// 依赖行同时保留所需插件与能力及主依赖标记，避免只核对插件名称。
use super::{identity, primary};

pub(super) fn parse_dependency_line(
    line: &str,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) {
    if identity::parse_dependency_identity_line(line, plugin_id, capability) {
        return;
    }

    if primary::parse_dependency_primary_line(line, primary) {
        return;
    }
}
