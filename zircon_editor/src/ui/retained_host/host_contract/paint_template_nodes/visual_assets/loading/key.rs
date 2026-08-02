use super::super::RasterTargetSize;
use super::cache::visual_asset_cache_generation;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn image_pixels_cache_key(
    base_key: &str,
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
    format!(
        "{base_key}:generation:{}:{size_key}:tint:{tint_key}",
        visual_asset_cache_generation()
    )
}

#[cfg(test)]
mod tests {
    use super::image_pixels_cache_key;
    use crate::ui::retained_host::host_contract::paint_template_nodes::RasterTargetSize;

    #[test]
    fn raster_cache_key_separates_size_and_tint_without_a_candidate_path() {
        let small = image_pixels_cache_key(
            "icon:save",
            RasterTargetSize::new(16, 16),
            Some([1, 2, 3, 4]),
        );
        let large = image_pixels_cache_key(
            "icon:save",
            RasterTargetSize::new(24, 24),
            Some([1, 2, 3, 4]),
        );
        let recolored = image_pixels_cache_key(
            "icon:save",
            RasterTargetSize::new(16, 16),
            Some([4, 3, 2, 1]),
        );

        assert_ne!(small, large);
        assert_ne!(small, recolored);
        assert!(!small.contains(".svg"));
    }
}
