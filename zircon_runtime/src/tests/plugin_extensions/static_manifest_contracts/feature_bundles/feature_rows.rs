//! 把可选特性与外部特性字段送入同一遍历器，保持各自诊断上下文；不在转发层追加声明准入。
use std::path::Path;

use super::feature_bundle_rows::for_each_feature_bundle;

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn for_each_optional_feature(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    for_each_feature_bundle(
        table,
        relative_path,
        "optional_features",
        "optional feature",
        visit,
    );
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn for_each_feature_extension(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    for_each_feature_bundle(
        table,
        relative_path,
        "feature_extensions",
        "feature extension",
        visit,
    );
}
