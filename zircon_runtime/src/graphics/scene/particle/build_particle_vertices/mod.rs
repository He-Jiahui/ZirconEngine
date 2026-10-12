#[path = "../../../build_particle_vertices/build_particle_vertices.rs"]
mod build_particle_vertices;

pub(in crate::graphics::scene::scene_renderer::particle) use build_particle_vertices::build_particle_vertices;
