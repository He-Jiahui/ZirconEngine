use std::any::Any;

use zircon_plugin_navigation_runtime::{
    plugin_registration, DefaultNavigationManager, DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME,
    NAVIGATION_MODULE_NAME, PLUGIN_ID,
};
use zircon_runtime::asset::AssetModule;
use zircon_runtime::builtin::RuntimeModuleCompositionCompiler;
use zircon_runtime::core::framework::navigation::{
    NavMeshAsset, NavPathQuery, NavPathStatus, NavigationManager,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};
use zircon_runtime::core::manager::ManagerResolver;
use zircon_runtime::core::runtime::{CoreRuntime, TasksModule};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::navigation::{BuiltinNavigationManager, BuiltinNavigationModule};
use zircon_runtime::plugin::RuntimePluginCatalog;
use zircon_runtime::scene::SceneNavigationRuntimeHandle;

fn assert_single_owner_and_flat_path<T: Any + Send + Sync + NavigationManager>(
    runtime: &CoreRuntime,
    implementation_name: &str,
) {
    let concrete = runtime
        .resolve_driver::<T>(implementation_name)
        .expect("implementation driver must resolve");
    let scene = SceneNavigationRuntimeHandle::new(&runtime.handle())
        .expect("scene view must resolve")
        .enter()
        .expect("scene view must enter its original owner");
    let resolver = ManagerResolver::new(runtime.handle());
    let public = resolver
        .resolve(
            resolver
                .navigation_handle()
                .expect("public navigation handle"),
        )
        .expect("public navigation view must enter its original owner");
    let owner = std::ptr::from_ref(&*concrete).cast::<()>();
    assert_eq!(owner, std::ptr::from_ref(&*scene).cast::<()>());
    assert_eq!(owner, std::ptr::from_ref(&*public).cast::<()>());

    // Reuse the older manager fixture's flat geometry and 3-4-5 query.
    let handle = concrete
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 5.0))
        .expect("the owner must accept the flat navmesh");
    let mut query = NavPathQuery::new([0.0, 0.0, 0.0], [3.0, 0.0, 4.0]);
    query.nav_mesh = Some(handle);
    let path = public
        .find_path(query)
        .expect("the facade must query the mesh loaded through the same runtime");
    assert_eq!(path.status, NavPathStatus::Complete);
    assert_eq!(path.points.len(), 2);
    assert_eq!(path.length, 5.0);
    eprintln!("NAV_SERVICE_CONTROL owner={implementation_name} same_owner=true path_points=2 path_length=5");
}

#[test]
fn navigation_exported_module_admits_before_first_activation() {
    let report = plugin_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    let descriptor = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NAVIGATION_MODULE_NAME)
        .expect("production plugin must contribute its navigation module");

    let runtime = CoreRuntime::new();
    runtime.register_module(TasksModule.descriptor()).unwrap();
    runtime
        .register_module(AssetModule::default().descriptor())
        .unwrap();
    let admission = runtime.register_module(descriptor.clone());
    if let Err(ref error) = admission {
        eprintln!("NAV_CORE_ADMISSION catalog_ok=true error={error:?}");
    }
    admission.expect("catalog-contributed navigation module must register before activation");
    runtime.activate_module(NAVIGATION_MODULE_NAME).unwrap();
    assert_single_owner_and_flat_path::<DefaultNavigationManager>(
        &runtime,
        DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME,
    );
}

#[test]
fn navigation_linked_provider_is_admitted_to_runtime_composition() {
    let report = plugin_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    let target = RuntimeTargetMode::ClientRuntime;
    let catalog = RuntimePluginCatalog::from_registration_reports([report], []);
    let plan = catalog.compiled_project_plan(
        &ProjectPluginManifest {
            selections: vec![
                ProjectPluginSelection::runtime_plugin(PLUGIN_ID, true, true)
                    .with_target_modes([target]),
            ],
        },
        target,
    );
    assert_eq!(plan.linked_provider_package_ids(), &[PLUGIN_ID.to_owned()]);
    let composition = RuntimeModuleCompositionCompiler::new(&plan).compile();
    if let Err(ref rejection) = composition {
        eprintln!(
            "NAV_COMPOSITION linked={PLUGIN_ID} diagnostics={:?}",
            rejection.diagnostics()
        );
    }
    let admitted = composition.expect("the selected linked module must join the activation graph");
    assert!(admitted.module_descriptors().iter().any(|module| {
        module.name == NAVIGATION_MODULE_NAME
            && module
                .drivers
                .iter()
                .any(|driver| driver.name.as_str() == DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME)
    }));
}

#[test]
fn builtin_navigation_control_admits_one_owner_and_queries_the_flat_mesh() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(BuiltinNavigationModule.descriptor())
        .expect("the real builtin control module must register before activation");
    runtime.activate_module(NAVIGATION_MODULE_NAME).unwrap();
    assert_single_owner_and_flat_path::<BuiltinNavigationManager>(
        &runtime,
        "navigation.runtime.Driver.BuiltinNavigationRuntime",
    );
}
