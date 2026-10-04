use super::*;
use crate::text::sdf::SdfMode;

#[test]
/// 防止打包器只检查最后一页而浪费旧页空隙，保持离线产物页面数与稳定输入的分配契约。
fn font_sdf_pack_reuses_earlier_page_before_allocating_another() {
    let generated = vec![
        generated_glyph(1, 32, 16),
        generated_glyph(2, 40, 60),
        generated_glyph(3, 30, 16),
    ];

    let (pages, glyphs) = pack_generated_glyphs(generated, 64).unwrap();

    assert_eq!(pages.len(), 2);
    assert_eq!(glyphs[2].page_index, 0);
}

/// 构造单通道紧密行像素，让页面回收测试只聚焦分配顺序，避免字体和任务池影响。
fn generated_glyph(glyph_id: u32, width: u32, height: u32) -> GeneratedGlyph {
    GeneratedGlyph {
        codepoint: glyph_id,
        glyph_id,
        data: SdfGlyphData {
            size: UVec2::new(width, height),
            bitmap_left: 0.0,
            bitmap_bottom: 0.0,
            advance: width as f32,
            ascent: height as f32,
            pixels: vec![glyph_id as u8; (width * height) as usize],
            channels: 1,
            spread_px: 8.0,
            mode: SdfMode::Sdf,
        },
    }
}
