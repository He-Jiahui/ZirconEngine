use std::sync::Arc;

use zircon_module::EngineModule;

pub fn feature_modules() -> Vec<Arc<dyn EngineModule>> {
    vec![
        Arc::new(zircon_physics::PhysicsModule),
        Arc::new(zircon_sound::SoundModule),
        Arc::new(zircon_texture::TextureModule),
        Arc::new(zircon_ui::UiModule),
        Arc::new(zircon_net::NetModule),
        Arc::new(zircon_navigation::NavigationModule),
        Arc::new(zircon_particles::ParticlesModule),
        Arc::new(zircon_animation::AnimationModule),
    ]
}
