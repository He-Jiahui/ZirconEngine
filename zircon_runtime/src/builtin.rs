use std::sync::Arc;

use zircon_module::EngineModule;

use crate::{asset, foundation, graphics, input, platform, scene, script, ui};

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
        Arc::new(zircon_physics::PhysicsModule),
        Arc::new(zircon_sound::SoundModule),
        Arc::new(zircon_texture::TextureModule),
        Arc::new(zircon_net::NetModule),
        Arc::new(zircon_navigation::NavigationModule),
        Arc::new(zircon_particles::ParticlesModule),
        Arc::new(zircon_animation::AnimationModule),
    ]
}
