use std::path::{Path, PathBuf};

use super::super::candidates::is_svg_path;
use super::super::mui_icons;
use super::super::svg::render_svg_file_image;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_image_from_candidates(
    candidates: Vec<PathBuf>,
) -> Option<crate::ui::retained_host::primitives::Image> {
    for path in candidates {
        if let Some(image) = load_image_from_path(&path) {
            return Some(image);
        }
    }
    None
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_image_from_path(
    path: &Path,
) -> Option<crate::ui::retained_host::primitives::Image> {
    if !path.exists() {
        return None;
    }
    if mui_icons::is_module_path(path) {
        return mui_icons::render_module_image(path);
    }
    if is_svg_path(path) {
        return render_svg_file_image(path);
    }
    let image =
        crate::ui::retained_host::primitives::Image::load_from_path(path).unwrap_or_default();
    let size = image.size();
    (size.width > 0 && size.height > 0).then_some(image)
}
