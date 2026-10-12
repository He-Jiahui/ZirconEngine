#[path = "construct.rs"]
mod construct;
#[path = "layout.rs"]
mod layout;
#[path = "../../../particle/particle_vertex/particle_vertex.rs"]
mod particle_vertex;

pub(in crate::graphics::scene::scene_renderer::particle) use particle_vertex::ParticleVertex;
