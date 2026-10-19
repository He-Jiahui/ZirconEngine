//! 组合真实物理、动画、场景及资产管理器的集成测试入口，供帧契约复用。

use zircon_plugin_sdk::{TestRuntime, WeakBridge};
use zircon_runtime::asset::{self, ProjectAssetManager};
use zircon_runtime::core::{CoreHandle, ServiceCallGuard};
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

pub(super) fn runtime_asset_manager(core: &CoreHandle) -> ServiceCallGuard<ProjectAssetManager> {
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
    fn admitted_directory(path: &std::path::Path) -> asset::project::ResolvedProjectPathIdentity {
        use zircon_runtime::asset::project::ProjectPaths;
        assert!(path.is_absolute(), "fixture paths must be absolute");
        assert!(
            !path.components().any(|component| matches!(
                component,
                std::path::Component::CurDir | std::path::Component::ParentDir
            )),
            "fixture paths must not contain dot components"
        );
        for ancestor in path.ancestors() {
            let metadata = std::fs::symlink_metadata(ancestor).unwrap();
            assert!(!metadata.file_type().is_symlink(), "fixture path alias");
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                assert_eq!(
                    metadata.file_attributes() & 0x400,
                    0,
                    "fixture reparse point"
                );
            }
        }
        let resolved = ProjectPaths::resolve_existing(path).unwrap();
        assert!(std::fs::metadata(resolved.operation_path())
            .unwrap()
            .is_dir());
        let physical = resolved.display_path().to_str().unwrap().replace('\\', "/");
        let declared = path.to_str().unwrap().replace('\\', "/");
        assert!(
            declared.eq_ignore_ascii_case(&physical),
            "fixture paths must name their physical location without aliases"
        );
        ProjectPaths::resolve_identity(resolved.operation_path()).unwrap()
    }

    fn new(label: &str) -> Self {
        use zircon_runtime::asset::project::ProjectPaths;
        let env_path = |name| std::path::PathBuf::from(std::env::var_os(name).expect(name));
        let target_path = env_path("CARGO_TARGET_DIR");
        let build_path = env_path("CARGO_BUILD_BUILD_DIR");
        let smoke_path = env_path("ZIRCON_RENDERING_SMOKE_OUTPUT_DIR");
        let target = Self::admitted_directory(&target_path);
        let build = Self::admitted_directory(&build_path);
        assert_eq!(target_path.file_name().unwrap(), "target");
        assert_eq!(build_path.file_name().unwrap(), "build");
        let task_path = target_path.parent().unwrap();
        let task = Self::admitted_directory(task_path);
        assert_eq!(Self::admitted_directory(build_path.parent().unwrap()), task);
        let approved =
            Self::admitted_directory(std::path::Path::new("D:/cargo-targets/zircon-local/win32"));
        assert_eq!(
            Self::admitted_directory(task_path.parent().unwrap()),
            approved
        );
        let fixtures = Self::admitted_directory(&build_path.join("fixtures"));
        let smoke = Self::admitted_directory(&smoke_path);
        assert_eq!(
            Self::admitted_directory(smoke_path.parent().unwrap()),
            fixtures
        );
        assert!(smoke.is_within(&build));
        for name in ["TEMP", "TMP", "TMPDIR"] {
            assert_eq!(Self::admitted_directory(&env_path(name)), smoke);
        }
        let executable = std::env::current_exe().unwrap();
        let binary_path = ProjectPaths::display_path(executable.parent().unwrap());
        let binary = Self::admitted_directory(&binary_path);
        assert!(
            binary.is_within(&build) || binary.is_within(&target),
            "test binary must live in its registered task build or target"
        );
        static NEXT_FIXTURE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT_FIXTURE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = ProjectPaths::resolve_identity(smoke.operation_path().join(format!(
            "runtime55-{label}-{}-{nanos}-{id}",
            std::process::id()
        )))
        .unwrap();
        assert!(root.is_within(&smoke));
        // Every authority check precedes mkdir/write. create_dir never adopts an existing root.
        assert_eq!(Self::admitted_directory(&smoke_path), smoke);
        std::fs::create_dir(root.operation_path()).unwrap();
        let fixture = Self { root };
        assert_eq!(
            Self::admitted_directory(&ProjectPaths::display_path(fixture.root.operation_path())),
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
        let current = Self::admitted_directory(
            &zircon_runtime::asset::project::ProjectPaths::display_path(self.root.operation_path()),
        );
        assert_eq!(
            current, self.root,
            "cleanup must retain this fixture's physical identity"
        );
        std::fs::remove_dir_all(self.root.operation_path()).unwrap();
    }
}
