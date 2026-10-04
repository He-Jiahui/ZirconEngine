// 固定清单模块列表仅在 modules 顶层表内读取；后续过滤 runtime 类型再与运行时包清单比较。
mod line;
mod state;

use self::state::ModuleContributionParserState;
use super::StaticModule;

pub(super) fn modules_from_plugin_toml(manifest: &str) -> Vec<StaticModule> {
    let mut parser = ModuleContributionParserState::default();
    for line in manifest.lines().map(str::trim) {
        parser.parse_manifest_line(line);
    }
    parser.finish()
}
