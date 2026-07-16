use super::*;

#[test]
fn render_text_atlas_allocates_missing_page_before_lru_eviction() {
    let pages = vec![
        resident_page(GlyphAtlasFormat::AlphaMask, 0, 3, false),
        resident_page(GlyphAtlasFormat::AlphaMask, 2, 2, false),
    ];

    assert_eq!(
        page_residency_decision(&pages, GlyphAtlasFormat::AlphaMask, 3),
        GlyphAtlasPageResidencyDecision::Allocate(page_key(GlyphAtlasFormat::AlphaMask, 1))
    );
}

#[test]
fn render_text_atlas_rebuild_residency_prefers_unreferenced_page_before_allocation() {
    let pages = vec![resident_page(GlyphAtlasFormat::AlphaMask, 0, 3, false)];

    assert_eq!(
        page_rebuild_residency_decision(&pages, GlyphAtlasFormat::AlphaMask, 3),
        GlyphAtlasPageResidencyDecision::Evict(page_key(GlyphAtlasFormat::AlphaMask, 0))
    );
}

#[test]
fn render_text_atlas_evicts_lru_page() {
    let pages = vec![
        resident_page(GlyphAtlasFormat::Sdf, 0, 1, false),
        resident_page(GlyphAtlasFormat::Sdf, 1, 4, true),
        resident_page(GlyphAtlasFormat::Sdf, 2, 2, false),
    ];

    assert_eq!(
        page_residency_decision(&pages, GlyphAtlasFormat::Sdf, 3),
        GlyphAtlasPageResidencyDecision::Evict(page_key(GlyphAtlasFormat::Sdf, 0))
    );
}

#[test]
fn render_text_atlas_lru_refuses_to_evict_pages_referenced_this_frame() {
    let pages = vec![
        resident_page(GlyphAtlasFormat::Msdf, 0, 1, true),
        resident_page(GlyphAtlasFormat::Msdf, 1, 2, true),
    ];

    assert_eq!(
        page_residency_decision(&pages, GlyphAtlasFormat::Msdf, 2),
        GlyphAtlasPageResidencyDecision::Blocked
    );
}

#[test]
fn render_text_atlas_residency_applies_eviction_as_page_rebuild() {
    let mut pages = vec![
        resident_page(GlyphAtlasFormat::Color, 0, 1, false),
        resident_page(GlyphAtlasFormat::Color, 1, 2, false),
    ];
    let decision = page_residency_decision(&pages, GlyphAtlasFormat::Color, 2);

    let reservation = apply_page_residency_decision(&mut pages, decision, UVec2::new(512, 512), 8);

    assert_eq!(
        reservation.decision,
        GlyphAtlasPageResidencyDecision::Evict(page_key(GlyphAtlasFormat::Color, 0))
    );
    assert_eq!(
        reservation.page,
        Some(
            GlyphAtlasPageSpec::new(page_key(GlyphAtlasFormat::Color, 0), UVec2::new(512, 512),)
                .with_generation(1)
        )
    );
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].spec().size, UVec2::new(512, 512));
    assert_eq!(pages[0].spec().generation, 1);
}

fn resident_page(
    format: GlyphAtlasFormat,
    page_index: u32,
    last_used_frame: u64,
    referenced_in_frame: bool,
) -> GlyphAtlasResidentPage {
    let mut page = GlyphAtlasResidentPage::reserved(
        GlyphAtlasPageSpec::new(page_key(format, page_index), UVec2::new(1024, 1024)),
        last_used_frame,
    );
    if !referenced_in_frame {
        page.clear_frame_reference();
    }
    page
}

fn page_key(format: GlyphAtlasFormat, page_index: u32) -> GlyphAtlasPageKey {
    GlyphAtlasPageKey::new(format, page_index)
}
