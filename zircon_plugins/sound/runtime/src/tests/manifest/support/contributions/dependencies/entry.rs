// 只扫描 sound/plugin.toml 中对应的数组表；输出供静态与运行时包清单对照，输入不应当是任意 TOML。
use super::{line, state};

pub(super) fn dependencies_from_plugin_toml(manifest: &str) -> Vec<super::super::StaticDependency> {
    let mut parser = state::DependencyParserState::default();

    for line in manifest.lines().map(str::trim) {
        line::parse_dependency_line(line, &mut parser);
    }
    parser.finish()
}
