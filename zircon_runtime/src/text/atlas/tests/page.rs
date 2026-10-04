use super::*;

#[test]
fn render_text_atlas_subpixel_mask_uses_distinct_rgba_page_format() {
    let page = GlyphAtlasPageSpec::new(
        GlyphAtlasPageKey::new(GlyphAtlasFormat::SubpixelMask, 0),
        UVec2::new(256, 256),
    );

    assert!(GlyphAtlasFormat::supported_formats().contains(&GlyphAtlasFormat::SubpixelMask));
    assert_eq!(page.key.format, GlyphAtlasFormat::SubpixelMask);
    assert_eq!(page.storage_format, GlyphAtlasStorageFormat::Rgba8Unorm);
    assert_eq!(
        page.sampling_semantics,
        GlyphAtlasSamplingSemantics::SubpixelCoverage
    );
    assert_eq!(page.storage_format.bytes_per_pixel(), 4);
}

#[test]
fn render_text_atlas_rgba_storage_keeps_color_and_subpixel_blend_semantics_distinct() {
    let subpixel_page = GlyphAtlasPageSpec::new(
        GlyphAtlasPageKey::new(GlyphAtlasFormat::SubpixelMask, 0),
        UVec2::new(128, 128),
    );
    let color_page = GlyphAtlasPageSpec::new(
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Color, 0),
        UVec2::new(128, 128),
    );

    assert_eq!(subpixel_page.storage_format, color_page.storage_format);
    assert_eq!(
        subpixel_page.sampling_semantics,
        GlyphAtlasSamplingSemantics::SubpixelCoverage
    );
    assert_eq!(
        color_page.sampling_semantics,
        GlyphAtlasSamplingSemantics::ColorRgba
    );
}

#[test]
fn render_text_atlas_resident_page_bytes_account_for_storage_format() {
    let atlas = GlyphAtlasSet::default()
        .with_page(GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
            UVec2::new(16, 8),
        ))
        .with_page(GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::Color, 0),
            UVec2::new(16, 8),
        ));

    assert_eq!(atlas.page_count(), 2);
    assert_eq!(atlas.resident_page_byte_len(), 16 * 8 * (1 + 4));
}

#[test]
fn render_text_atlas_persistent_slot_allocation_does_not_snapshot_resident_pages() {
    let source = include_str!("../page.rs");
    let allocation_start = source
        .find("pub(crate) fn allocate_persistent_bitmap_slot")
        .expect("persistent slot allocation owner");
    let allocation_end = source[allocation_start..]
        .find("\n    fn insert_persistent_bitmap_slot")
        .map(|offset| allocation_start + offset)
        .expect("persistent slot allocation boundary");
    let allocation = &source[allocation_start..allocation_end];

    assert!(
        !allocation.contains("collect::<Vec<_>>()"),
        "persistent slot allocation must scan resident pages directly"
    );
}

#[test]
fn bitmap_page_shadow_commits_only_accepted_zero_initialized_pages() {
    let page = GlyphAtlasPageSpec::new(
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
        UVec2::new(8, 8),
    )
    .with_generation(3);
    let mut atlas = GlyphAtlasSet::from_page(page.clone());
    let patch = GlyphAtlasBitmapPageShadowPatch {
        page_key: page.key,
        page_generation: page.generation,
        target_rect: GlyphAtlasRect {
            x: 2,
            y: 1,
            width: 2,
            height: 2,
        },
        bytes_per_row: 2,
        bytes: vec![0x7F; 4].into(),
    };
    let mut failed = GlyphAtlasBitmapPageShadowCommit::default();
    failed.zero_initialized_pages.insert(page.key);
    failed.failed_zero_initialized_pages.insert(page.key);
    failed.patches.push(patch.clone());
    atlas.commit_bitmap_page_shadow(failed);
    assert!(atlas.bitmap_page_shadow_bytes(&page).is_none());

    let mut accepted = GlyphAtlasBitmapPageShadowCommit::default();
    accepted.zero_initialized_pages.insert(page.key);
    accepted.patches.push(patch);
    atlas.commit_bitmap_page_shadow(accepted);

    let shadow = atlas.bitmap_page_shadow_bytes(&page).unwrap();
    assert_eq!(shadow.len(), page.byte_len());
    assert_eq!(shadow[1 * 8 + 2], 0x7F);
    assert_eq!(shadow[2 * 8 + 3], 0x7F);
    assert_eq!(shadow[0], 0);

    atlas.invalidate_bitmap_page_upload_state([page.key]);
    assert!(atlas.bitmap_page_shadow_bytes(&page).is_none());
}
