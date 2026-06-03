const EFFECT_LUT_WIDTH: u32 = 64;
const EFFECT_LUT_BYTES_PER_TEXEL: u32 = 4;
const EFFECT_LUT_3D_SIZE: u32 = 2;

pub(super) fn effect_lut_texture_view(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-effect-lut-fallback"),
        size: wgpu::Extent3d {
            width: EFFECT_LUT_WIDTH,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        &effect_lut_rgba_data(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(EFFECT_LUT_WIDTH * EFFECT_LUT_BYTES_PER_TEXEL),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: EFFECT_LUT_WIDTH,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub(super) fn effect_lut_texture_3d_view(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-effect-lut-3d-fallback"),
        size: wgpu::Extent3d {
            width: EFFECT_LUT_3D_SIZE,
            height: EFFECT_LUT_3D_SIZE,
            depth_or_array_layers: EFFECT_LUT_3D_SIZE,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        &effect_lut_3d_rgba_data(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(EFFECT_LUT_3D_SIZE * EFFECT_LUT_BYTES_PER_TEXEL),
            rows_per_image: Some(EFFECT_LUT_3D_SIZE),
        },
        wgpu::Extent3d {
            width: EFFECT_LUT_3D_SIZE,
            height: EFFECT_LUT_3D_SIZE,
            depth_or_array_layers: EFFECT_LUT_3D_SIZE,
        },
    );
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn effect_lut_rgba_data() -> Vec<u8> {
    let mut bytes = Vec::with_capacity((EFFECT_LUT_WIDTH * EFFECT_LUT_BYTES_PER_TEXEL) as usize);
    for index in 0..EFFECT_LUT_WIDTH {
        let t = index as f32 / (EFFECT_LUT_WIDTH - 1) as f32;
        let shaped = t * t * (3.0 - 2.0 * t);
        let value = (shaped * u8::MAX as f32).round() as u8;
        bytes.extend_from_slice(&[value, value, value, u8::MAX]);
    }
    bytes
}

fn effect_lut_3d_rgba_data() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(
        (EFFECT_LUT_3D_SIZE * EFFECT_LUT_3D_SIZE * EFFECT_LUT_3D_SIZE * EFFECT_LUT_BYTES_PER_TEXEL)
            as usize,
    );
    for blue in 0..EFFECT_LUT_3D_SIZE {
        for green in 0..EFFECT_LUT_3D_SIZE {
            for red in 0..EFFECT_LUT_3D_SIZE {
                let scale = u8::MAX as f32 / (EFFECT_LUT_3D_SIZE - 1) as f32;
                bytes.extend_from_slice(&[
                    (red as f32 * scale).round() as u8,
                    (green as f32 * scale).round() as u8,
                    (blue as f32 * scale).round() as u8,
                    u8::MAX,
                ]);
            }
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::{
        effect_lut_3d_rgba_data, effect_lut_rgba_data, EFFECT_LUT_3D_SIZE,
        EFFECT_LUT_BYTES_PER_TEXEL, EFFECT_LUT_WIDTH,
    };

    #[test]
    fn generated_effect_lut_is_s_curve_with_stable_texture_stride() {
        let bytes = effect_lut_rgba_data();

        assert_eq!(
            bytes.len(),
            (EFFECT_LUT_WIDTH * EFFECT_LUT_BYTES_PER_TEXEL) as usize
        );
        assert_eq!(&bytes[0..4], &[0, 0, 0, 255]);
        let last = bytes.len() - EFFECT_LUT_BYTES_PER_TEXEL as usize;
        assert_eq!(&bytes[last..], &[255, 255, 255, 255]);

        let midpoint = (EFFECT_LUT_WIDTH / 2 * EFFECT_LUT_BYTES_PER_TEXEL) as usize;
        assert!(bytes[midpoint] > 127);
        assert_eq!(bytes[midpoint], bytes[midpoint + 1]);
        assert_eq!(bytes[midpoint], bytes[midpoint + 2]);
        assert_eq!(bytes[midpoint + 3], 255);
    }

    #[test]
    fn generated_effect_lut_3d_is_identity_cube() {
        let bytes = effect_lut_3d_rgba_data();

        assert_eq!(
            bytes.len(),
            (EFFECT_LUT_3D_SIZE
                * EFFECT_LUT_3D_SIZE
                * EFFECT_LUT_3D_SIZE
                * EFFECT_LUT_BYTES_PER_TEXEL) as usize
        );
        assert_eq!(&bytes[0..4], &[0, 0, 0, 255]);
        let last = bytes.len() - EFFECT_LUT_BYTES_PER_TEXEL as usize;
        assert_eq!(&bytes[last..], &[255, 255, 255, 255]);
    }
}
