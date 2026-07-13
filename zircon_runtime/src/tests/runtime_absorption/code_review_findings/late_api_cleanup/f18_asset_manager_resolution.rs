#[test]
fn review_f18_asset_manager_resolution_returns_registered_handle() {
    let resolver =
        include_str!("../../../../asset/pipeline/manager/asset_manager/resolve_asset_manager.rs");
    let handle =
        include_str!("../../../../asset/pipeline/manager/asset_manager/asset_manager_handle.rs");
    let runtime = include_str!("../../../../core/runtime/runtime.rs");
    let runtime_handle = include_str!("../../../../core/runtime/handle/resolution.rs");
    let project_session = include_str!("../../../../dynamic_api/session/project.rs");
    let review_findings = concat!(
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let runtime_10 = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let dynamic_session_doc =
        include_str!("../../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let asset_facade_doc = include_str!("../../../../../../docs/zircon_runtime/asset/facade.md");
    let f18_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F18 |"))
        .expect("F18 review findings top row");

    assert!(
        f18_row.contains("asset manager resolution") && f18_row.ends_with("| Runtime 10 |"),
        "F18 overview row should keep only the finding and Runtime 10 owner"
    );
    assert!(
        review_findings
            .contains("f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred"),
        "F18 numbered output should record manager-resolution review closed status"
    );

    for generic_manager_anchor in [
        "pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError>",
        "self.handle().resolve_manager(name)",
        "let service = self.resolve_named_service(name, Some(ServiceKind::Manager))?;",
        "downcast_resolved_service(name, service)",
    ] {
        assert!(
            runtime.contains(generic_manager_anchor)
                || runtime_handle.contains(generic_manager_anchor),
            "F18 generic manager resolution should keep registered-handle shape `{generic_manager_anchor}`"
        );
    }

    for required in [
        "pub fn resolve_asset_manager(core: &CoreHandle) -> Result<Arc<AssetManagerHandle>, CoreError>",
        "core.resolve_manager::<AssetManagerHandle>(ASSET_MANAGER_NAME)",
    ] {
        assert!(
            resolver.contains(required),
            "F18 asset manager resolver should contain `{required}`"
        );
    }
    for forbidden in [
        "Result<Arc<dyn AssetManager>, CoreError>",
        ".map(|holder| holder.shared())",
    ] {
        assert!(
            !resolver.contains(forbidden),
            "F18 asset manager resolver should not return trait objects directly or hide handle conversion `{forbidden}`"
        );
    }
    assert!(
        handle.contains("pub struct AssetManagerHandle")
            && handle.contains("inner: Arc<dyn AssetManager>")
            && handle.contains("pub fn shared(&self) -> Arc<dyn AssetManager>"),
        "AssetManagerHandle should remain the registered manager handle that owns the object-safe shared service"
    );
    assert!(
        project_session.contains("resolve_asset_manager(core)")
            && project_session.contains("let asset_manager = asset_manager.shared();")
            && project_session.contains(".open_project(&self.root_display())"),
        "dynamic project startup should make the trait-object conversion explicit at the caller boundary"
    );

    for doc_anchor in [
        "F18 asset manager resolution return shape",
        "runtime_10_asset_manager_resolution_handle_shape_coremin_check_passed",
        "f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred",
        "review_f18_asset_manager_resolution_returns_registered_handle",
        "Result<Arc<AssetManagerHandle>, CoreError>",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || runtime_10.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || dynamic_session_doc.contains(doc_anchor)
                || asset_facade_doc.contains(doc_anchor),
            "F18 docs should record `{doc_anchor}`"
        );
    }
}
