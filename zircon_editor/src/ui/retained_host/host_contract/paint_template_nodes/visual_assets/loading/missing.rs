use super::super::{HostPaintImagePixels, RasterTargetSize, ICON_TINT};
use super::cache::{cached_visual_asset_pixels, store_visual_asset_pixels};
use super::key::image_pixels_cache_key;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn missing_icon_pixels(
    base_key: &str,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let color = tint.unwrap_or(ICON_TINT);
    let cache_key = missing_icon_cache_key(base_key, target, color);
    if let Some(cached) = cached_visual_asset_pixels(&cache_key) {
        return cached;
    }
    let mut rgba = vec![0; target.width as usize * target.height as usize * 4];
    let edge = target.width.min(target.height);
    let stroke = (edge / 10).clamp(1, 3);
    let max_x = target.width.saturating_sub(1);
    let max_y = target.height.saturating_sub(1);

    for y in 0..target.height {
        for x in 0..target.width {
            let border = x < stroke
                || y < stroke
                || max_x.saturating_sub(x) < stroke
                || max_y.saturating_sub(y) < stroke;
            let diagonal = x.abs_diff(y) < stroke || x.abs_diff(max_y.saturating_sub(y)) < stroke;
            if !border && !diagonal {
                continue;
            }
            let offset = ((y * target.width + x) as usize) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }

    let image = HostPaintImagePixels {
        resource_key: cache_key.clone(),
        width: target.width,
        height: target.height,
        rgba: rgba.into(),
        atlas: None,
    };
    let image = image.is_valid().then_some(image);
    store_visual_asset_pixels(cache_key, base_key, std::iter::empty(), image.clone());
    image
}

fn missing_icon_cache_key(base_key: &str, target: RasterTargetSize, color: [u8; 4]) -> String {
    image_pixels_cache_key(
        &format!("missing-icon:{base_key}"),
        Some(target),
        Some(color),
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::RasterTargetSize;
    use super::missing_icon_cache_key;

    #[test]
    fn missing_icon_cache_key_separates_tint_variants() {
        let target = RasterTargetSize::new(16, 16).expect("valid raster size");

        assert_ne!(
            missing_icon_cache_key("icon:save", target, [1, 2, 3, 4]),
            missing_icon_cache_key("icon:save", target, [4, 3, 2, 1]),
        );
    }
}
