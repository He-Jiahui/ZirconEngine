use std::ops::Deref;
use std::sync::Arc;

use zircon_runtime::asset::{ProjectAssetManager, ProjectAssetManagerAccess};
use zircon_runtime::core::manager::{manager_service_handle, RegisteredManagerService};
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
};
use zircon_runtime::graphics::WgpuRenderFramework;

const MODULE_NAME: &str = "IntegrationTestProjectAssetRuntime";
const SERVICE_NAME: &str = "IntegrationTestProjectAssetRuntime.Manager.ProjectAssetManager";

pub struct ProjectAssetTestRuntime {
    _runtime: CoreRuntime,
    access: ProjectAssetManagerAccess,
}

impl ProjectAssetTestRuntime {
    pub fn new(manager: Arc<ProjectAssetManager>) -> Self {
        let runtime = CoreRuntime::new();
        runtime
            .register_module(
                ModuleDescriptor::new(MODULE_NAME, "integration test project asset runtime")
                    .with_manager(ManagerDescriptor::new(
                        RegistryName::from_parts(
                            MODULE_NAME,
                            ServiceKind::Manager,
                            "ProjectAssetManager",
                        ),
                        StartupMode::Immediate,
                        Vec::new(),
                        Arc::new(move |_| {
                            Ok(
                                Arc::new(RegisteredManagerService::new(Arc::clone(&manager)))
                                    as ServiceObject,
                            )
                        }),
                    )),
            )
            .expect("integration test ProjectAssetManager service should register");
        runtime
            .activate_module(MODULE_NAME)
            .expect("integration test ProjectAssetManager module should activate");
        let core = runtime.handle();
        let handle = manager_service_handle(&core, SERVICE_NAME)
            .expect("integration test ProjectAssetManager handle should resolve");
        let access = ProjectAssetManagerAccess::new(core, handle);
        Self {
            _runtime: runtime,
            access,
        }
    }

    pub fn access(&self) -> ProjectAssetManagerAccess {
        self.access.clone()
    }
}

pub struct TestWgpuRenderFramework {
    _asset_runtime: ProjectAssetTestRuntime,
    framework: WgpuRenderFramework,
}

impl TestWgpuRenderFramework {
    pub fn new(asset_runtime: ProjectAssetTestRuntime, framework: WgpuRenderFramework) -> Self {
        Self {
            _asset_runtime: asset_runtime,
            framework,
        }
    }
}

impl Deref for TestWgpuRenderFramework {
    type Target = WgpuRenderFramework;

    fn deref(&self) -> &Self::Target {
        &self.framework
    }
}
