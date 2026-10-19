//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
#[test]
fn review_f5_asset_meta_uses_typed_error() {
    let meta = include_str!("../../../../../asset/project/meta.rs");
    let meta_tests = include_str!("../../../../../asset/project/tests/meta.rs");
    assert!(meta.contains("#[path = \"tests/meta.rs\"]"));
    assert!(meta.contains("mod tests;"));
    assert!(meta_tests.contains("fn asset_meta_validation_reports_typed_future_version_error("));
    let asset_mod = include_str!("../../../../../asset/mod.rs");
    let importer_doc =
        include_str!("../../../../../../../docs/crates/zircon_runtime/asset/importer.md");

    for required in [
        "pub type AssetMetaResult<T> = std::result::Result<T, AssetMetaError>;",
        "pub enum AssetMetaError",
        "UnsupportedOldFormatVersion { found: u32, minimum: u32 }",
        "UnsupportedFutureFormatVersion { found: u32, supported: u32 }",
        "RetiredSourceHashField",
        "fn validate_current(&self) -> AssetMetaResult<()>",
    ] {
        assert!(
            meta.contains(required),
            "F5 asset meta typed error owner should contain `{required}`"
        );
    }
    for forbidden in ["Result<(), String>", "Err(format!("] {
        assert!(
            !meta.contains(forbidden),
            "asset meta migration should not keep String error branch `{forbidden}`"
        );
    }
    assert!(
        asset_mod
            .replace("\r\n", "\n")
            .contains("#[path = \"project/mod.rs\"]\npub mod project;"),
        "asset root must expose the actual public project facade"
    );
    let project = include_str!("../../../../../asset/project/mod.rs");
    assert!(project
        .replace("\r\n", "\n")
        .contains("#[path = \"meta.rs\"]\nmod meta;"));
    let meta_exports = project
        .split_once("pub use meta::{")
        .expect("the public project facade must reexport the private meta owner")
        .1
        .split_once("};")
        .expect("meta reexport list must be delimited")
        .0;
    assert!(
        meta_exports.contains("AssetMetaError") && meta_exports.contains("AssetMetaResult"),
        "asset::project facade should export AssetMetaError/AssetMetaResult"
    );
    let _: Option<crate::asset::project::AssetMetaError> = None;
    let _: crate::asset::project::AssetMetaResult<()> = Ok(());
    for doc_anchor in [
        "F5 asset meta typed errors",
        "runtime_15_asset_meta_typed_errors_static_passed_cargo_deferred",
        "review_f5_asset_meta_uses_typed_error",
        "UnsupportedOldFormatVersion",
        "UnsupportedFutureFormatVersion",
        "asset/project/meta.rs",
    ] {
        assert!(
            importer_doc.contains(doc_anchor),
            "F5 asset meta docs should record `{doc_anchor}`"
        );
    }
}
