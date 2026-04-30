use std::path::PathBuf;

pub(super) fn load_preview_image(source: &str, icon_name: &str) -> slint::Image {
    for path in preview_image_candidates(source, icon_name) {
        if path.exists() {
            return slint::Image::load_from_path(&path).unwrap_or_default();
        }
    }
    slint::Image::default()
}

fn preview_image_candidates(source: &str, icon_name: &str) -> Vec<PathBuf> {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut candidates = Vec::new();
    if !source.is_empty() {
        candidates.push(assets.join(source));
        candidates.push(assets.join("icons").join(source));
    }
    if !icon_name.is_empty() {
        candidates.push(
            assets
                .join("icons")
                .join("ionicons")
                .join(format!("{icon_name}.svg")),
        );
    }
    candidates
}
