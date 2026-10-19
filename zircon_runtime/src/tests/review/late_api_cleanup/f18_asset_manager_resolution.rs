//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
#[test]
fn review_f18_asset_manager_resolution_returns_registered_handle() {
    let handle = include_str!("../../../asset/pipeline/manager/asset_manager/handle.rs");
    let contract = include_str!("../../../asset/pipeline/manager/asset_manager/asset_manager.rs");
    let service = include_str!("../../../core/manager/service.rs");
    let runtime = include_str!("../../../core/runtime/runtime.rs");
    let runtime_handle = include_str!("../../../core/runtime/handle/resolution.rs");
    let project_session = include_str!("../../../dynamic_api/session/project.rs");
    let review_findings = crate::runtime_test_history_concat!(
        include_str!("../../../../../docs/architecture/runtime-architecture-review-m0.md"),
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let convention =
        include_str!("../../../../../docs/testing-and-validation/development-conventions.md");
    let runtime_10 = include_str!(
        "../../../../../docs/testing-and-validation/runtime-dynamic-api-validation-worklist.md"
    );
    let runtime_index = include_str!(
        "../../../../../docs/crates/zircon_runtime/structure/runtime-absorption-evidence.md"
    );
    let dynamic_session_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/dynamic_api/session.md");
    let asset_facade_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/asset/facade.md");
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

    for required in [
        "pub fn enter(&self) -> Result<ServiceCallGuard<T>, CoreError>",
        "core.enter_service(&self.identity)",
        "impl Drop for ServiceCallAdmission",
        "self.core.release_service_call(&self.identity);",
        "self.core.release_service_call(source);",
    ] {
        assert!(
            runtime_handle.contains(required),
            "manager call admission must retain {required}"
        );
    }
    for required in [
        "pub fn resolve_manager<T: Any + Send + Sync>(",
        ") -> Result<ServiceCallGuard<T>, CoreError>",
        "self.handle.resolve_manager(name)",
    ] {
        assert!(
            runtime.contains(required),
            "current core/runtime/runtime.rs must retain {required}"
        );
    }
    for required in [
        "pub fn resolve_manager<T: Any + Send + Sync>(",
        ") -> Result<ServiceCallGuard<T>, CoreError>",
        "self.resolve_manager_handle(name)?.enter()",
        "self.resolve_service_handle(name, ServiceKind::Manager)",
        "self.validate_registered_service_identity(identity, ServiceKind::Manager)?;",
    ] {
        assert!(
            runtime_handle.contains(required),
            "current core/runtime/handle/resolution.rs must retain {required}"
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
            && service.contains("pub(crate) index: u32")
            && service.contains("pub(crate) generation: u32")
            && service.contains("pub(crate) service: RegistryName"),
        "AssetManager should use the generic versioned manager service handle"
    );
    for required in [
        "runtime: CoreWeak",
        "std::ptr::eq(self.runtime.inner.as_ptr(), Arc::as_ptr(&core.inner))",
        "if !handle.belongs_to(self)",
        "resolve_registered_manager::<RegisteredManagerService<T>>",
    ] {
        assert!(
            service.contains(required),
            "current core/manager/service.rs must retain {required}"
        );
    }
    assert!(
        open_project_assets.contains("asset_manager_handle(core)")
            && open_project_assets.contains("resolve_manager_service(core, handle)")
            && open_project_assets.contains(".open_prepared_project(project_for_activation)")
            && !open_project_assets.contains("project_asset_manager_handle(core)"),
        "dynamic project startup should resolve the abstract versioned handle and transfer its prepared owner at the use point"
    );
    for required in [
        "let project_for_activation = project.clone();",
        "match asset_manager.open_prepared_project(project_for_activation)",
        "self.project.take();",
        "project.restore_scene_component_registry(previous_project_registry);",
        "*runtime_extensions = previous_extensions;",
    ] {
        assert!(
            open_project_assets.contains(required),
            "prepared activation: {required}"
        );
    }
    let activation = open_project_assets
        .find("match asset_manager.open_prepared_project(")
        .unwrap();
    assert!(
        open_project_assets
            .find("let project_for_activation = project.clone();")
            .unwrap()
            < activation
    );
    assert!(activation < open_project_assets.find("self.project.take();").unwrap());
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
