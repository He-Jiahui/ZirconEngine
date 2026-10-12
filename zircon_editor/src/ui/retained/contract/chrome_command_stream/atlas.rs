use super::ChromeImageUvRect;

pub(in crate::ui::retained_host::host_contract) fn atlas_subimage_rgba(
    atlas_width: u32,
    atlas_height: u32,
    rgba: &[u8],
    atlas_uv: ChromeImageUvRect,
) -> Option<(u32, u32, Vec<u8>)> {
    let (x0, y0, x1, y1) = atlas_uv_pixel_rect(atlas_width, atlas_height, atlas_uv)?;
    let width = x1.checked_sub(x0)?;
    let height = y1.checked_sub(y0)?;
    if width == 0 || height == 0 || rgba.len() != atlas_width as usize * atlas_height as usize * 4 {
        return None;
    }
    let mut subimage = Vec::with_capacity(width as usize * height as usize * 4);
    let atlas_width = atlas_width as usize;
    let width = width as usize;
    for y in y0 as usize..y1 as usize {
        let start = ((y * atlas_width) + x0 as usize) * 4;
        let end = start + width * 4;
        subimage.extend_from_slice(&rgba[start..end]);
    }
    Some((width as u32, height, subimage))
}

fn atlas_uv_pixel_rect(
    atlas_width: u32,
    atlas_height: u32,
    atlas_uv: ChromeImageUvRect,
) -> Option<(u32, u32, u32, u32)> {
    if atlas_width == 0
        || atlas_height == 0
        || !atlas_uv.min[0].is_finite()
        || !atlas_uv.min[1].is_finite()
        || !atlas_uv.max[0].is_finite()
        || !atlas_uv.max[1].is_finite()
        || atlas_uv.min[0] < 0.0
        || atlas_uv.min[1] < 0.0
        || atlas_uv.max[0] > 1.0
        || atlas_uv.max[1] > 1.0
        || atlas_uv.min[0] >= atlas_uv.max[0]
        || atlas_uv.min[1] >= atlas_uv.max[1]
    {
        return None;
    }
    let x0 = (atlas_uv.min[0] * atlas_width as f32).round() as u32;
    let y0 = (atlas_uv.min[1] * atlas_height as f32).round() as u32;
    let x1 = (atlas_uv.max[0] * atlas_width as f32).round() as u32;
    let y1 = (atlas_uv.max[1] * atlas_height as f32).round() as u32;
    (x0 < x1 && y0 < y1 && x1 <= atlas_width && y1 <= atlas_height).then_some((x0, y0, x1, y1))
}
