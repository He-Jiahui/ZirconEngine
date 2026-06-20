use std::path::Path;

use super::super::RasterTargetSize;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn image_pixels_cache_key(
    base_key: &str,
    path: &Path,
    target: Option<RasterTargetSize>,
    tint: Option<[u8; 4]>,
) -> String {
    let size_key = target
        .map(|target| format!("{}x{}", target.width, target.height))
        .unwrap_or_else(|| "intrinsic".to_string());
    let tint_key = tint
        .map(|tint| {
            format!(
                "{:02x}{:02x}{:02x}{:02x}",
                tint[0], tint[1], tint[2], tint[3]
            )
        })
        .unwrap_or_else(|| "none".to_string());
    format!("{base_key}:{size_key}:tint:{tint_key}:{}", path.display())
}
