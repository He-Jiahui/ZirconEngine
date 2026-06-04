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
