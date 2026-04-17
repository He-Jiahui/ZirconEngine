use image::GenericImageView;

use crate::types::GraphicsError;

pub(super) fn decode_icon_rgba(
    bytes: &[u8],
    label: &str,
) -> Result<(u32, u32, Vec<u8>), GraphicsError> {
    let image = image::load_from_memory(bytes)
        .map_err(|error| GraphicsError::Asset(format!("{label}: {error}")))?;
    let luma = image.to_luma8();
    let (width, height) = image.dimensions();
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for alpha in luma.into_raw() {
        rgba.extend_from_slice(&[255, 255, 255, alpha]);
    }

    Ok((width, height, rgba))
}
