//! Codec registration owned by the editor's concrete command domain.

#[path = "scene.rs"]
mod scene;

pub(crate) use scene::register_scene_command_codecs;
