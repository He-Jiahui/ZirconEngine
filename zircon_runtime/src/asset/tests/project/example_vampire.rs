use std::path::{Path, PathBuf};

#[cfg(all(feature = "graphics", feature = "script"))]
mod manifest_scene_imports;
#[cfg(feature = "graphics")]
mod third_person_render_extract;

fn vampire_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("examples")
        .join("vampire")
}
