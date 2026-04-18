use std::sync::Arc;

use zircon_module::EngineModule;

pub fn core_modules() -> Vec<Arc<dyn EngineModule>> {
    vec![
        Arc::new(zircon_foundation::FoundationModule),
        Arc::new(zircon_platform::PlatformModule),
        Arc::new(zircon_input::InputModule),
        Arc::new(zircon_asset::AssetModule),
        Arc::new(zircon_graphics::GraphicsModule),
        Arc::new(zircon_scene::SceneModule),
        Arc::new(zircon_script::ScriptModule),
    ]
}
