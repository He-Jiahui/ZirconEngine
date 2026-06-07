use super::{line, state};

pub(super) fn dependencies_from_plugin_toml(manifest: &str) -> Vec<super::super::StaticDependency> {
    let mut parser = state::DependencyParserState::default();

    for line in manifest.lines().map(str::trim) {
        line::parse_dependency_line(line, &mut parser);
    }
    parser.finish()
}
