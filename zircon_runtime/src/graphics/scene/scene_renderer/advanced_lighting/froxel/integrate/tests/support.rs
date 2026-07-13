use super::*;

pub(super) fn write_output_png(path: PathBuf, texels: &[[f32; 4]]) {
    const SCALE: u32 = 16;
    let mut image =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::new(TEST_OUTPUT[0] * SCALE, TEST_OUTPUT[1] * SCALE);
    for y in 0..TEST_OUTPUT[1] {
        for x in 0..TEST_OUTPUT[0] {
            let sample = texels[(y * TEST_OUTPUT[0] + x) as usize];
            let mapped = [sample[0], sample[1], sample[2]].map(|value| {
                let reinhard = value.max(0.0) / (1.0 + value.max(0.0));
                (reinhard.powf(1.0 / 2.2) * 255.0 + 0.5) as u8
            });
            for py in 0..SCALE {
                for px in 0..SCALE {
                    image.put_pixel(
                        x * SCALE + px,
                        y * SCALE + py,
                        Rgba(if px == 0 || py == 0 {
                            [5, 7, 9, 255]
                        } else {
                            [mapped[0], mapped[1], mapped[2], 255]
                        }),
                    );
                }
            }
        }
    }
    image.save_with_format(path, ImageFormat::Png).unwrap();
}

pub(super) fn render_test_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
}

pub(super) fn f16_bits_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits >> 15) << 31;
    let exponent = u32::from((bits >> 10) & 0x1f);
    let mantissa = u32::from(bits & 0x03ff);
    let f32_bits = match exponent {
        0 if mantissa == 0 => sign,
        0 => {
            let mut normalized = mantissa;
            let mut shift = 0_u32;
            while normalized & 0x0400 == 0 {
                normalized <<= 1;
                shift += 1;
            }
            sign | ((113_u32.saturating_sub(shift)) << 23) | ((normalized & 0x03ff) << 13)
        }
        0x1f => sign | 0x7f80_0000 | (mantissa << 13),
        _ => sign | ((exponent + 112) << 23) | (mantissa << 13),
    };
    f32::from_bits(f32_bits)
}

pub(super) fn test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::PRIMARY;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-volumetric-chain-test-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}
