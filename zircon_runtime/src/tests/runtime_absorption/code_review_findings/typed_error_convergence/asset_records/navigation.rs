#[test]
fn review_f5_navigation_asset_uses_typed_error() {
    let navigation = include_str!("../../../../../asset/assets/navigation.rs");
    let asset_assets_mod = include_str!("../../../../../asset/assets/mod.rs");
    let asset_mod = include_str!("../../../../../asset/mod.rs");
    let navigation_tests = include_str!("../../../../../asset/tests/assets/navigation.rs");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let navigation_doc =
        include_str!("../../../../../../../docs/zircon_runtime/asset/assets/navigation.md");

    for required in [
        "pub type NavigationAssetResult<T> = std::result::Result<T, NavigationAssetError>;",
        "pub enum NavigationAssetError",
        "Serialize(#[source] bincode::Error)",
        "Deserialize(#[source] bincode::Error)",
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
            asset_assets_mod.contains(required)
                || asset_mod.contains(required)
                || navigation_tests.contains(required),
            "navigation asset export/test surface should contain `{required}`"
        );
    }
    for doc_anchor in [
        "F5 navigation asset typed errors",
        "runtime_15_navigation_asset_typed_errors_static_passed_cargo_deferred",
        "review_f5_navigation_asset_uses_typed_error",
        "NavigationAssetError",
        "asset/assets/navigation.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || navigation_doc.contains(doc_anchor),
            "F5 navigation asset docs should record `{doc_anchor}`"
        );
    }
}
