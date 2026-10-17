//! 组合真实物理、动画、场景及资产管理器的集成测试入口，供帧契约复用。
use std::sync::Arc;

use zircon_plugin_sdk::{TestRuntime, WeakBridge};
use zircon_runtime::asset::{self, ProjectAssetManager};
use zircon_runtime::core::CoreHandle;
use zircon_runtime::plugin::RuntimeExtensionCatalogReport;

use zircon_plugin_physics_runtime::PhysicsQueryInterface;

pub(super) fn runtime_with_physics_animation_scene_asset() -> ScopedTestRuntime {
    let physics_plugin = zircon_plugin_physics_runtime::runtime_plugin();
    let animation_plugin = zircon_plugin_animation_runtime::runtime_plugin();
    let fixture = ManagedConfigFixture::new("animation-physics");
    let runtime = TestRuntime::builder()
        .with_config_file_path(fixture.config_path())
        .with_runtime_plugin(&physics_plugin)
        .with_runtime_plugin(&animation_plugin)
        .build()
        .unwrap();
    ScopedTestRuntime {
        runtime,
        _fixture: fixture,
    }
}

pub(super) fn runtime_physics_query_bridge(
    extension_report: &RuntimeExtensionCatalogReport,
) -> WeakBridge<dyn PhysicsQueryInterface> {
    extension_report
        .registry
        .frozen_bridge_table()
        .resolve_weak::<dyn PhysicsQueryInterface>()
}

pub(super) fn runtime_with_scene_asset_only() -> ScopedTestRuntime {
    let fixture = ManagedConfigFixture::new("animation-scene");
    let runtime = TestRuntime::builder()
        .with_config_file_path(fixture.config_path())
        .build()
        .unwrap();
    ScopedTestRuntime {
        runtime,
        _fixture: fixture,
    }
}

pub(super) fn runtime_asset_manager(core: &CoreHandle) -> Arc<ProjectAssetManager> {
    zircon_runtime::core::manager::resolve_manager_service(
        core,
        asset::project_asset_manager_handle(core).unwrap(),
    )
    .unwrap()
}

// Runtime drops before the fixture; callers close their later-created level/service locals first.
pub(super) struct ScopedTestRuntime {
    runtime: TestRuntime,
    _fixture: ManagedConfigFixture,
}

impl std::ops::Deref for ScopedTestRuntime {
    type Target = TestRuntime;
    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

struct ManagedConfigFixture {
    root: zircon_runtime::asset::project::ResolvedProjectPathIdentity,
}

impl ManagedConfigFixture {
    fn new(label: &str) -> Self {
        use zircon_runtime::asset::ProjectPaths;
        static NEXT_FIXTURE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let approved = ProjectPaths::resolve_existing("E:/cargo-targets/zircon-local")
            .expect("the managed E drive fixture root must already exist");
        let display = ProjectPaths::display_path(approved.operation_path());
        assert!(
            display
                .to_str()
                .expect("managed root must be Unicode")
                .replace('\\', "/")
                .eq_ignore_ascii_case("E:/cargo-targets/zircon-local"),
            "the approved root must resolve to the physical E drive root"
        );
        let approved = ProjectPaths::resolve_identity(approved.operation_path()).unwrap();
        let executable = std::env::current_exe().unwrap();
        let binary = ProjectPaths::resolve_identity(executable.parent().unwrap()).unwrap();
        assert!(
            binary.is_within(&approved),
            "test binary must physically live under the managed E root"
        );
        let id = NEXT_FIXTURE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = ProjectPaths::resolve_identity(binary.operation_path().join(format!(
            "runtime55-{label}-{}-{nanos}-{id}",
            std::process::id()
        )))
        .unwrap();
        assert!(root.is_within(&binary));
        // Every authority check precedes mkdir/write. create_dir never adopts an existing root.
        std::fs::create_dir(root.operation_path()).unwrap();
        let fixture = Self { root };
        assert_eq!(
            ProjectPaths::resolve_identity(fixture.root.operation_path()).unwrap(),
            fixture.root
        );
        fixture
    }

    fn config_path(&self) -> std::path::PathBuf {
        self.root.operation_path().join("config.json")
    }
}

impl Drop for ManagedConfigFixture {
    fn drop(&mut self) {
        // The declaring scope/wrapper closes all runtimes, handles and worker owners first.
        let current =
            zircon_runtime::asset::ProjectPaths::resolve_identity(self.root.operation_path())
                .unwrap();
        assert_eq!(
            current, self.root,
            "cleanup must retain this fixture's physical identity"
        );
        std::fs::remove_dir_all(self.root.operation_path()).unwrap();
    }
}
