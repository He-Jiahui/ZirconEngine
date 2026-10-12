// 扫描静态清单的能力状态表；调用者只传入仓内固定 TOML，离开表或输入结束时提交当前行。
use super::{line, state};

pub(super) fn capability_statuses_from_plugin_toml(
    manifest: &str,
) -> Vec<zircon_runtime::plugin::CapabilityStatusManifest> {
    let mut parser = state::CapabilityStatusParserState::default();
    for line in manifest.lines().map(str::trim) {
        line::parse_capability_status_line(line, &mut parser);
    }
    parser.finish()
}
