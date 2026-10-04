// 扫描固定清单中的可选功能区域；每次表切换提交前一行，结束时提交最后一个功能。
use super::super::types::StaticOptionalFeatureManifest;
use super::state::OptionalFeatureParserState;

pub(in super::super) fn optional_features_from_plugin_toml(
    manifest: &str,
) -> Vec<StaticOptionalFeatureManifest> {
    let mut parser = OptionalFeatureParserState::default();
    for line in manifest.lines().map(str::trim) {
        parser.parse_manifest_line(line);
    }
    parser.finish()
}
