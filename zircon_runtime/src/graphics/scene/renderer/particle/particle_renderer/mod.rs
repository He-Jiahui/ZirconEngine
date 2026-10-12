#[path = "../../../particle/particle_renderer/construct.rs"]
mod construct;
#[path = "../../../particle_renderer/particle_renderer.rs"]
mod particle_renderer;
#[path = "record.rs"]
mod record;
#[path = "../../../particle/particle_renderer/record_velocity.rs"]
mod record_velocity;
#[path = "../../../particle/particle_renderer/shader_source.rs"]
mod shader_source;

pub(crate) use particle_renderer::ParticleRenderer;
