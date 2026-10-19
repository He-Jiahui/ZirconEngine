//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
#[test]
fn review_f5_navigation_asset_uses_typed_error() {
    let navigation = include_str!("../../../../../core/framework/navigation/asset/mod.rs");
    let navigation_mod = include_str!("../../../../../core/framework/navigation/mod.rs");
    let navigation_tests = include_str!("../../../../../asset/tests/assets/navigation.rs");
    let navigation_doc =
        include_str!("../../../../../../../docs/crates/zircon_runtime/asset/assets/navigation.md");

    for required in [
        "pub type NavigationAssetResult<T> = std::result::Result<T, NavigationAssetError>;",
        "pub enum NavigationAssetError",
        "Serialize(#[source] zircon_runtime_interface::serialization::PositionalEncodeError)",
        "Deserialize(#[source] zircon_runtime_interface::serialization::PositionalDecodeError)",
        "pub fn to_bytes(&self) -> NavigationAssetResult<Vec<u8>>",
        "pub fn from_bytes(bytes: &[u8]) -> NavigationAssetResult<Self>",
    ] {
        assert!(
            navigation.contains(required),
            "F5 navigation asset typed error owner should contain `{required}`"
        );
    }
    for forbidden in [
        "Result<Vec<u8>, String>",
        "Result<Self, String>",
        "error.to_string()",
    ] {
        assert!(
            !navigation.contains(forbidden),
            "navigation asset binary helpers should not keep String error branch `{forbidden}`"
        );
    }
    for required in [
        "NavigationAssetError",
        "NavigationAssetResult",
        "navmesh_binary_roundtrip_reports_typed_errors",
        "NavigationAssetError::Deserialize(_)",
    ] {
        assert!(
            navigation_mod.contains(required) || navigation_tests.contains(required),
            "navigation asset export/test surface should contain `{required}`"
        );
    }
    for doc_anchor in [
        "F5 navigation asset typed errors",
        "runtime_15_navigation_asset_typed_errors_static_passed_cargo_deferred",
        "review_f5_navigation_asset_uses_typed_error",
        "NavigationAssetError",
        "core/framework/navigation/asset/mod.rs",
    ] {
        assert!(
            navigation_doc.contains(doc_anchor),
            "F5 navigation asset docs should record `{doc_anchor}`"
        );
    }
}
