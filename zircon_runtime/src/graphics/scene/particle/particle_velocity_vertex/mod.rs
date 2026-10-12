#[path = "construct.rs"]
mod construct;
#[path = "layout.rs"]
mod layout;
#[path = "../../../particle_velocity_vertex/particle_velocity_vertex.rs"]
mod particle_velocity_vertex;

pub(in crate::graphics::scene::scene_renderer::particle) use particle_velocity_vertex::ParticleVelocityVertex;
