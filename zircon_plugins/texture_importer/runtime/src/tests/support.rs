pub(super) fn tiny_png_bytes() -> Vec<u8> {
    use image::{ImageBuffer, ImageFormat, Rgba};

    let image = ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([0, 0, 0, 255])
        }
    });
    let mut bytes = std::io::Cursor::new(Vec::new());
    image.write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

pub(super) fn tiny_stacked_png_bytes() -> Vec<u8> {
    use image::{ImageBuffer, ImageFormat, Rgba};

    let image = ImageBuffer::<Rgba<u8>, _>::from_fn(2, 4, |_x, y| {
        if y < 2 {
            Rgba([255, 0, 0, 255])
        } else {
            Rgba([0, 0, 255, 255])
        }
    });
    let mut bytes = std::io::Cursor::new(Vec::new());
    image.write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

pub(super) fn tiny_jpeg_bytes() -> Vec<u8> {
    use image::{ImageBuffer, ImageFormat, Rgb};

    let image = ImageBuffer::<Rgb<u8>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgb([255, 0, 0])
        } else {
            Rgb([0, 0, 255])
        }
    });
    let mut bytes = std::io::Cursor::new(Vec::new());
    image.write_to(&mut bytes, ImageFormat::Jpeg).unwrap();
    bytes.into_inner()
}

pub(super) fn tiny_image_bytes(format: image::ImageFormat) -> Vec<u8> {
    if matches!(
        format,
        image::ImageFormat::Hdr | image::ImageFormat::OpenExr
    ) {
        return tiny_rgb32f_image_bytes(format);
    }

    use image::{DynamicImage, ImageBuffer, Rgb};

    let image = ImageBuffer::<Rgb<u8>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgb([255, 255, 255])
        } else {
            Rgb([0, 0, 0])
        }
    });
    let dynamic = DynamicImage::ImageRgb8(image);
    let mut bytes = std::io::Cursor::new(Vec::new());
    dynamic.write_to(&mut bytes, format).unwrap();
    bytes.into_inner()
}

pub(super) fn tiny_rgb32f_image_bytes(format: image::ImageFormat) -> Vec<u8> {
    use image::{DynamicImage, ImageBuffer, Rgb};

    let image = ImageBuffer::<Rgb<f32>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgb([1.0, 0.25, 0.0])
        } else {
            Rgb([0.0, 0.5, 1.0])
        }
    });
    let dynamic = DynamicImage::ImageRgb32F(image);
    let mut bytes = std::io::Cursor::new(Vec::new());
    dynamic.write_to(&mut bytes, format).unwrap();
    bytes.into_inner()
}

pub(super) fn tiny_psd_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"8BPS");
    bytes.extend_from_slice(&1_u16.to_be_bytes());
    bytes.extend_from_slice(&[0; 6]);
    bytes.extend_from_slice(&4_u16.to_be_bytes());
    bytes.extend_from_slice(&1_u32.to_be_bytes());
    bytes.extend_from_slice(&1_u32.to_be_bytes());
    bytes.extend_from_slice(&8_u16.to_be_bytes());
    bytes.extend_from_slice(&3_u16.to_be_bytes());
    bytes.extend_from_slice(&0_u32.to_be_bytes());
    bytes.extend_from_slice(&0_u32.to_be_bytes());
    bytes.extend_from_slice(&0_u32.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&[12, 34, 56, 200]);
    bytes
}
