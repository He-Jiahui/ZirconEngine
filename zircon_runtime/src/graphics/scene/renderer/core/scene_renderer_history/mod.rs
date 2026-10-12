#[path = "../../../../scene_renderer_history/prepare_history_textures.rs"]
mod prepare_history_textures;
#[path = "../../../core/scene_renderer_history/release_history.rs"]
mod release_history;

pub(super) use prepare_history_textures::prepare_history_textures;
