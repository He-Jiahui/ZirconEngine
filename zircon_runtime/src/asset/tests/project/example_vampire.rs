use std::path::{Path, PathBuf};

mod manifest_scene_imports;
mod third_person_render_extract;

fn vampire_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("examples")
        .join("vampire")
}
