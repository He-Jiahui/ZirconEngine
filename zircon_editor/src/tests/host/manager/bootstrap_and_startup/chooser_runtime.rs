use std::ops::Deref;
use std::path::Path;
use std::time::{Duration, Instant};

use zircon_runtime::core::runtime::modules::{
    TasksModule, TimeModule, TASKS_MODULE_NAME, TIME_MODULE_NAME,
};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::engine_module::EngineModule;

pub(super) struct ChooserRuntime(CoreRuntime);

impl Deref for ChooserRuntime {
    type Target = CoreRuntime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for ChooserRuntime {
    fn drop(&mut self) {
        // Match the product owner's order: stop watchers before closing IO admission.
        let deadline = Instant::now() + Duration::from_secs(10);
        let core = self.0.handle();
        {
            let handle = zircon_runtime::asset::project_asset_manager_handle(&core).unwrap();
            let manager =
                zircon_runtime::core::manager::resolve_manager_service(&core, handle).unwrap();
            while !manager.shutdown_project_watchers_until(deadline) {
                assert!(Instant::now() < deadline, "chooser watchers did not join");
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        let _ = self
            .0
            .shutdown_until(deadline)
            .expect("chooser runtime shutdown");
    }
}

/// These chooser tests admit the real editor graph without resolving its lazy GPU service.
pub(super) fn with_config_path(path: &Path) -> ChooserRuntime {
    let runtime = CoreRuntime::new();
    let mut foundation = zircon_runtime::foundation::module_descriptor();
    zircon_runtime::foundation::bind_config_file_path(&mut foundation, path.to_path_buf())
        .expect("private chooser configuration path");
    runtime.store_config_value(
        crate::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
        serde_json::json!([]),
    );
    for descriptor in [
        foundation,
        TasksModule.descriptor(),
        TimeModule.descriptor(),
        zircon_runtime::platform::module_descriptor(),
        zircon_runtime::asset::module_descriptor(),
        zircon_runtime::scene::module_descriptor(),
        zircon_runtime::text::TextModule::default().descriptor(),
        zircon_runtime::graphics::graphics_module_descriptor(),
        zircon_runtime::input::module_descriptor(),
        zircon_runtime::ui::module_descriptor(),
        crate::ui::host::module::module_descriptor(),
    ] {
        runtime.register_module(descriptor).unwrap();
    }
    for name in [
        zircon_runtime::foundation::FOUNDATION_MODULE_NAME,
        TASKS_MODULE_NAME,
        TIME_MODULE_NAME,
        zircon_runtime::platform::PLATFORM_MODULE_NAME,
        zircon_runtime::asset::ASSET_MODULE_NAME,
        zircon_runtime::core::framework::scene::SCENE_MODULE_NAME,
        zircon_runtime::text::TEXT_MODULE_NAME,
        zircon_runtime::core::framework::render::GRAPHICS_MODULE_NAME,
        zircon_runtime::input::INPUT_MODULE_NAME,
        zircon_runtime::ui::UI_MODULE_NAME,
        crate::ui::host::module::EDITOR_MODULE_NAME,
    ] {
        runtime.activate_module(name).unwrap();
    }
    crate::tests::support::configure_editor_test_runtime_build_set(&runtime);
    ChooserRuntime(runtime)
}
