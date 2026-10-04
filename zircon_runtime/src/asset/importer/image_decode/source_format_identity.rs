use image::ImageFormat;

// IBL 缓存键使用固定协议值，不依赖 image crate 枚举判别值；
// 新增解码格式应显式分配新值，未知格式交由调用者拒绝。
pub(super) fn stable_source_format_identity(format: ImageFormat) -> Option<u32> {
    Some(match format {
        ImageFormat::Png => 1,
        ImageFormat::Jpeg => 2,
        ImageFormat::Gif => 3,
        ImageFormat::WebP => 4,
        ImageFormat::Pnm => 5,
        ImageFormat::Tiff => 6,
        ImageFormat::Tga => 7,
        ImageFormat::Dds => 8,
        ImageFormat::Bmp => 9,
        ImageFormat::Ico => 10,
        ImageFormat::Hdr => 11,
        ImageFormat::OpenExr => 12,
        ImageFormat::Farbfeld => 13,
        ImageFormat::Avif => 14,
        ImageFormat::Qoi => 15,
        _ => return None,
    })
}
