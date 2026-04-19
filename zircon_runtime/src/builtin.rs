use std::sync::Arc;

use zircon_module::EngineModule;

use crate::{asset, extensions, foundation, graphics, input, platform, scene, script, ui};

pub fn builtin_runtime_modules() -> Vec<Arc<dyn EngineModule>> {
    let mut modules: Vec<Arc<dyn EngineModule>> = vec![
        Arc::new(foundation::FoundationModule),
        Arc::new(platform::PlatformModule),
        Arc::new(input::InputModule),
        Arc::new(asset::AssetModule),
        Arc::new(graphics::GraphicsModule),
        Arc::new(scene::SceneModule),
        Arc::new(script::ScriptModule),
        Arc::new(ui::UiModule),
    ];

    modules.extend(runtime_extension_modules());
    modules
}

fn runtime_extension_modules() -> Vec<Arc<dyn EngineModule>> {
    vec![
        Arc::new(extensions::physics::PhysicsModule),
        Arc::new(extensions::sound::SoundModule),
        Arc::new(extensions::texture::TextureModule),
        Arc::new(extensions::net::NetModule),
        Arc::new(extensions::navigation::NavigationModule),
        Arc::new(extensions::particles::ParticlesModule),
        Arc::new(extensions::animation::AnimationModule),
    ]
}
