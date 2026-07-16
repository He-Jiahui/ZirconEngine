#[test]
fn review_f18_asset_manager_resolution_returns_registered_handle() {
    let handle = include_str!("../../../../asset/pipeline/manager/asset_manager/handle.rs");
    let contract =
        include_str!("../../../../asset/pipeline/manager/asset_manager/asset_manager.rs");
    let service = include_str!("../../../../core/manager/service.rs");
    let runtime = include_str!("../../../../core/runtime/runtime.rs");
    let runtime_handle = include_str!("../../../../core/runtime/handle/resolution.rs");
    let project_session = include_str!("../../../../dynamic_api/session/project.rs");
    let review_findings = concat!(
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!(
            "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
        )
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
    let open_project_assets = project_session
        .split("pub(super) fn open_project_assets")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn load_default_level").next())
        .expect("dynamic project asset-open owner should remain explicit");

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
        "pub fn asset_manager_handle(",
        "Result<ManagerServiceHandle<dyn AssetManager>, CoreError>",
        "manager_service_handle(core, ASSET_MANAGER_NAME)",
    ] {
        assert!(
            handle.contains(required),
            "F18 asset manager handle owner should contain `{required}`"
        );
    }
    for forbidden in [
        "AssetManagerHandle",
        "inner: Arc<dyn AssetManager>",
        "resolve_asset_manager",
    ] {
        assert!(
            !handle.contains(forbidden),
            "F18 asset manager handle owner should not retain legacy Arc-holder shape `{forbidden}`"
        );
    }
    assert!(
        service.contains("pub struct ManagerServiceHandle<T: ?Sized>")
            && service.contains("pub index: u32")
            && service.contains("pub generation: u32")
            && service.contains("pub service: RegistryName"),
        "AssetManager should use the generic versioned manager service handle"
    );
    assert!(
        open_project_assets.contains("asset_manager_handle(core)")
            && open_project_assets.contains("resolve_manager_service(core, handle)")
            && open_project_assets.contains(".open_prepared_project(project)")
            && !open_project_assets.contains("project_asset_manager_handle(core)"),
        "dynamic project startup should resolve the abstract versioned handle and transfer its prepared owner at the use point"
    );
    assert!(
        contract.contains("fn open_prepared_project(")
            && contract.contains("fn current_project_snapshot("),
        "prepared activation and deadlock-safe current-project snapshots should remain AssetManager service operations"
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
