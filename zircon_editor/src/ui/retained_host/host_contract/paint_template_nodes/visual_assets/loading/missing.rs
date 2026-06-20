use super::super::{HostPaintImagePixels, RasterTargetSize, ICON_TINT};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn missing_icon_pixels(
    base_key: &str,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let color = tint.unwrap_or(ICON_TINT);
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
        resource_key: format!("missing-icon:{base_key}:{}x{}", target.width, target.height),
        width: target.width,
        height: target.height,
        rgba,
        atlas: None,
    };
    image.is_valid().then_some(image)
}
