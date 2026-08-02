use std::collections::HashMap;

use super::super::font_assets::{effective_text_render_mode, LoadedUiFontAsset};
use super::super::resolved_batches::{
    resolved_auto_text_render_mode, AutoTextRasterRouter, ResolvedScreenSpaceUiTextBatches,
};
use super::super::*;
use super::support::text_batch;
use crate::asset::ProjectAssetManager;
use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextWritingMode};

#[test]
fn text_backend_routing_keeps_explicit_native_out_of_sdf_atlas_batches() {
    let native = text_batch("Normal", UiTextRenderMode::Native);
    let sdf = text_batch("Signed", UiTextRenderMode::Sdf);

    let routed = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(&[native], &[sdf]);

    assert_eq!(routed.native_texts().len(), 1);
    assert_eq!(routed.native_texts()[0].text, "Normal");
    assert_eq!(routed.sdf_texts().len(), 1);
    assert_eq!(routed.sdf_texts()[0].text, "Signed");
}

#[test]
fn text_backend_routing_respects_auto_font_mode_without_crossing_backends() {
    let mut routed = ResolvedScreenSpaceUiTextBatches::default();

    routed.push_resolved_auto_text(
        text_batch("NormalAuto", UiTextRenderMode::Auto),
        UiTextRenderMode::Native,
    );
    routed.push_resolved_auto_text(
        text_batch("SdfAuto", UiTextRenderMode::Auto),
        UiTextRenderMode::Sdf,
    );

    assert_eq!(routed.native_texts().len(), 1);
    assert_eq!(routed.native_texts()[0].text, "NormalAuto");
    assert_eq!(routed.sdf_texts().len(), 1);
    assert_eq!(routed.sdf_texts()[0].text, "SdfAuto");
}

#[test]
fn text_batch_resolution_invalidates_existing_renderer_after_shared_font_publish() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let mut reader = TextRenderState::new(0);
    let mut writer = TextRenderState::new(0);
    let previous_family = reader
        .font_database()
        .default_ui_family_for_test()
        .map(str::to_owned);
    assert!(writer.set_default_ui_family("Shared Screen-Space Refresh Family"));

    let asset_manager = ProjectAssetManager::default();
    let mut font_assets = HashMap::new();
    let mut auto_router = AutoTextRasterRouter::default();
    let resolved = super::super::resolved_batches::resolve_text_batches(
        &mut reader,
        &mut font_assets,
        &asset_manager,
        &mut auto_router,
        &[],
        &[],
        &[],
    );

    assert!(resolved.font_faces_changed());
    assert!(
        !reader.refresh_shared_font_database(),
        "batch resolution must consume the pending shared generation"
    );

    let _ = writer.set_default_ui_family_asset(previous_family.as_deref());
    assert!(reader.refresh_shared_font_database());
}

#[test]
fn text_font_refresh_recomputes_internal_vertical_advances() {
    let mut text = text_batch("AB", UiTextRenderMode::Sdf);
    text.writing_mode = UiTextWritingMode::VerticalRl;
    text.glyph_advances = vec![999.0, 999.0];

    super::super::super::render::text_advances::refresh_screen_space_text_batch_glyphs(&mut text);

    assert_eq!(text.glyph_advances.len(), 2);
    assert!(text.glyph_advances.iter().all(|advance| *advance < 999.0));
}

#[test]
fn text_font_refresh_preserves_resolved_layout_vertical_advances() {
    let mut text = text_batch("AB", UiTextRenderMode::Sdf);
    text.writing_mode = UiTextWritingMode::VerticalRl;
    text.source_range = Some(UiTextRange { start: 0, end: 2 });
    text.glyph_advances = vec![11.0, 13.0];

    super::super::super::render::text_advances::refresh_screen_space_text_batch_glyphs(&mut text);

    assert_eq!(text.glyph_advances, vec![11.0, 13.0]);
}

#[test]
fn auto_text_mode_uses_font_asset_default_when_present() {
    let resolved = effective_text_render_mode(
        UiTextRenderMode::Auto,
        Some(&LoadedUiFontAsset {
            family: Some("Studio Mono".to_string()),
            render_mode: Some(UiTextRenderMode::Sdf),
            composite_font: None,
        }),
    );

    assert_eq!(resolved, UiTextRenderMode::Sdf);
}

#[test]
fn explicit_text_mode_overrides_font_asset_default() {
    let resolved = effective_text_render_mode(
        UiTextRenderMode::Native,
        Some(&LoadedUiFontAsset {
            family: Some("Studio Mono".to_string()),
            render_mode: Some(UiTextRenderMode::Sdf),
            composite_font: None,
        }),
    );

    assert_eq!(resolved, UiTextRenderMode::Native);
}

#[test]
fn auto_text_mode_falls_back_to_native_without_font_asset_default() {
    let resolved = effective_text_render_mode(UiTextRenderMode::Auto, None);

    assert_eq!(resolved, UiTextRenderMode::Native);
}

#[test]
fn auto_text_without_explicit_font_default_uses_raster_policy() {
    let small = text_batch("SmallAuto", UiTextRenderMode::Auto);
    assert_eq!(
        resolved_auto_text_render_mode(&small, None),
        UiTextRenderMode::Native
    );

    let mut large = text_batch("LargeAuto", UiTextRenderMode::Auto);
    large.font_size = 24.0;
    assert_eq!(
        resolved_auto_text_render_mode(
            &large,
            Some(&LoadedUiFontAsset {
                family: Some("Studio Sans".to_string()),
                render_mode: Some(UiTextRenderMode::Auto),
                composite_font: None,
            }),
        ),
        UiTextRenderMode::Sdf
    );
}

#[test]
fn auto_text_policy_preserves_explicit_font_render_modes() {
    let mut text = text_batch("ExplicitFontMode", UiTextRenderMode::Auto);
    text.font_size = 48.0;

    for mode in [
        UiTextRenderMode::Native,
        UiTextRenderMode::Sdf,
        UiTextRenderMode::Msdf,
        UiTextRenderMode::Mtsdf,
    ] {
        assert_eq!(
            resolved_auto_text_render_mode(
                &text,
                Some(&LoadedUiFontAsset {
                    family: Some("Studio Sans".to_string()),
                    render_mode: Some(mode),
                    composite_font: None,
                }),
            ),
            mode
        );
    }
}

#[test]
fn auto_text_effects_use_the_distance_field_policy() {
    let mut outline = text_batch("OutlinedAuto", UiTextRenderMode::Auto);
    outline.text_effects.outline = Some(
        super::super::super::render::text_effects::ScreenSpaceUiTextOutline {
            width_px: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
        },
    );
    assert_eq!(
        resolved_auto_text_render_mode(&outline, None),
        UiTextRenderMode::Sdf
    );

    let mut shadow = text_batch("ShadowedAuto", UiTextRenderMode::Auto);
    shadow.text_effects.shadow = Some(
        super::super::super::render::text_effects::ScreenSpaceUiTextShadow {
            offset_px: [1.0, 0.0],
            color: [0.0, 0.0, 0.0, 0.5],
        },
    );
    assert_eq!(
        resolved_auto_text_render_mode(&shadow, None),
        UiTextRenderMode::Sdf
    );

    let mut glow = text_batch("GlowingAuto", UiTextRenderMode::Auto);
    glow.text_effects.glow = Some(
        super::super::super::render::text_effects::ScreenSpaceUiTextGlow {
            radius_px: 2.0,
            color: [1.0, 1.0, 1.0, 1.0],
        },
    );
    assert_eq!(
        resolved_auto_text_render_mode(&glow, None),
        UiTextRenderMode::Mtsdf
    );
}

#[test]
fn auto_text_router_keeps_the_warm_route_inside_the_hysteresis_band() {
    let mut router = AutoTextRasterRouter::default();
    let mut text = text_batch("StableAuto", UiTextRenderMode::Auto);

    text.font_size = 23.0;
    text.command_generation = 1;
    router.begin_frame();
    assert_eq!(router.resolve(&text, None), UiTextRenderMode::Native);

    for (generation, size_px) in [(2, 24.5), (3, 25.9), (4, 23.5)] {
        text.command_generation = generation;
        text.font_size = size_px;
        router.begin_frame();
        assert_eq!(router.resolve(&text, None), UiTextRenderMode::Native);
        assert_eq!(router.frame_report().retained_warm_route_count, 1);
    }

    text.command_generation = 5;
    text.font_size = 26.0;
    router.begin_frame();
    assert_eq!(router.resolve(&text, None), UiTextRenderMode::Sdf);
    assert_eq!(router.frame_report().route_switch_count, 1);

    text.command_generation = 6;
    text.font_size = 22.1;
    router.begin_frame();
    assert_eq!(router.resolve(&text, None), UiTextRenderMode::Sdf);
    assert_eq!(router.frame_report().retained_warm_route_count, 1);

    text.command_generation = 7;
    text.font_size = 21.9;
    router.begin_frame();
    assert_eq!(router.resolve(&text, None), UiTextRenderMode::Native);
    assert_eq!(router.frame_report().route_switch_count, 1);
}

#[test]
fn auto_text_router_evaluates_each_command_generation_once() {
    let mut router = AutoTextRasterRouter::default();
    let mut text = text_batch("GenerationAuto", UiTextRenderMode::Auto);
    text.font_size = 12.0;
    text.command_generation = 41;

    router.begin_frame();
    assert_eq!(router.resolve(&text, None), UiTextRenderMode::Native);
    assert_eq!(router.frame_report().policy_evaluation_count, 1);

    text.font_size = 48.0;
    assert_eq!(router.resolve(&text, None), UiTextRenderMode::Native);
    assert_eq!(router.frame_report().generation_cache_hit_count, 1);
    assert_eq!(router.frame_report().policy_evaluation_count, 1);
}

#[test]
fn auto_text_router_isolates_tree_and_layout_fragment_identity() {
    let mut router = AutoTextRasterRouter::default();
    let mut native = text_batch("Native fragment", UiTextRenderMode::Auto);
    native.font_size = 12.0;
    native.command_generation = 1;
    native.route_identity = ScreenSpaceUiTextRouteIdentity::new(
        "runtime.text.tree-a",
        zircon_runtime_interface::ui::event_ui::UiNodeId::new(7),
        Some(UiTextRange { start: 0, end: 6 }),
    );
    let mut sdf = native.clone();
    sdf.font_size = 48.0;
    sdf.route_identity = ScreenSpaceUiTextRouteIdentity::new(
        "runtime.text.tree-b",
        zircon_runtime_interface::ui::event_ui::UiNodeId::new(7),
        Some(UiTextRange { start: 0, end: 6 }),
    );
    let mut second_fragment = native.clone();
    second_fragment.font_size = 48.0;
    second_fragment.route_identity = ScreenSpaceUiTextRouteIdentity::new(
        "runtime.text.tree-a",
        zircon_runtime_interface::ui::event_ui::UiNodeId::new(7),
        Some(UiTextRange { start: 7, end: 15 }),
    );

    router.begin_frame();
    assert_eq!(router.resolve(&native, None), UiTextRenderMode::Native);
    assert_eq!(router.resolve(&sdf, None), UiTextRenderMode::Sdf);
    assert_eq!(
        router.resolve(&second_fragment, None),
        UiTextRenderMode::Sdf
    );
    assert_eq!(router.frame_report().entry_count, 3);
    assert_eq!(router.frame_report().policy_evaluation_count, 3);
}

#[test]
fn auto_text_router_bounds_state_and_reclaims_idle_routes() {
    let mut router = AutoTextRasterRouter::with_capacity_for_test(2);
    for node_id in 1..=3 {
        let mut text = text_batch("BoundedAuto", UiTextRenderMode::Auto);
        text.route_identity = ScreenSpaceUiTextRouteIdentity::new(
            "runtime.text.router.capacity",
            zircon_runtime_interface::ui::event_ui::UiNodeId::new(node_id),
            None,
        );
        text.command_generation = node_id;
        router.begin_frame();
        assert_eq!(router.resolve(&text, None), UiTextRenderMode::Native);
    }
    assert_eq!(router.frame_report().entry_count, 2);
    assert_eq!(router.frame_report().capacity_eviction_count, 1);

    for _ in 0..=300 {
        router.begin_frame();
    }
    assert_eq!(router.frame_report().entry_count, 0);
    assert_eq!(router.frame_report().idle_eviction_count, 1);
}

#[test]
fn auto_text_router_scale_evaluations_are_linear_and_bounded() {
    for batch_count in [1_u64, 100, 1_000] {
        let mut router = AutoTextRasterRouter::default();
        router.begin_frame();
        for node_id in 1..=batch_count {
            let mut text = text_batch("ScaleAuto", UiTextRenderMode::Auto);
            text.route_identity = ScreenSpaceUiTextRouteIdentity::new(
                "runtime.text.router.scale",
                zircon_runtime_interface::ui::event_ui::UiNodeId::new(node_id),
                None,
            );
            text.command_generation = 1;
            let _ = router.resolve(&text, None);
        }
        assert_eq!(
            router.frame_report().policy_evaluation_count,
            batch_count as usize
        );
        assert_eq!(router.frame_report().entry_count, batch_count as usize);
        assert_eq!(router.frame_report().capacity_eviction_count, 0);
    }
}

#[test]
#[ignore = "manual 31-sample Auto route scale evidence; no machine-time acceptance threshold"]
fn auto_text_router_reports_scale_p50_p95() {
    for batch_count in [1_u64, 100, 1_000] {
        let texts = (1..=batch_count)
            .map(|node_id| {
                let mut text = text_batch("ScaleAuto", UiTextRenderMode::Auto);
                text.route_identity = ScreenSpaceUiTextRouteIdentity::new(
                    "runtime.text.router.metrics",
                    zircon_runtime_interface::ui::event_ui::UiNodeId::new(node_id),
                    None,
                );
                text
            })
            .collect::<Vec<_>>();
        let mut samples_ns = Vec::with_capacity(31);
        for _ in 0..31 {
            let mut router = AutoTextRasterRouter::default();
            router.begin_frame();
            let started = std::time::Instant::now();
            for text in &texts {
                let _ = router.resolve(text, None);
            }
            samples_ns.push(started.elapsed().as_nanos());
            assert_eq!(
                router.frame_report().policy_evaluation_count,
                batch_count as usize
            );
        }
        samples_ns.sort_unstable();
        let p50_ns = samples_ns[samples_ns.len() / 2];
        let p95_ns = samples_ns[(samples_ns.len() * 95).div_ceil(100) - 1];
        println!(
            "auto_text_batches={batch_count} policy_evaluations={batch_count} \
             p50_ns={p50_ns} p95_ns={p95_ns}"
        );
    }
}

#[test]
fn native_text_align_maps_start_end_through_text_direction() {
    assert_eq!(
        native_text_align(UiTextAlign::Start, UiTextDirection::LeftToRight),
        NativeTextAlign::Left
    );
    assert_eq!(
        native_text_align(UiTextAlign::End, UiTextDirection::LeftToRight),
        NativeTextAlign::Right
    );
    assert_eq!(
        native_text_align(UiTextAlign::Start, UiTextDirection::RightToLeft),
        NativeTextAlign::Right
    );
    assert_eq!(
        native_text_align(UiTextAlign::End, UiTextDirection::RightToLeft),
        NativeTextAlign::Left
    );
    assert_eq!(
        native_text_align(UiTextAlign::Justify, UiTextDirection::LeftToRight),
        NativeTextAlign::Justified
    );
}

#[test]
fn native_text_area_placement_snaps_fractional_origin_to_device_pixels() {
    let mut text = text_batch("editor base.zui", UiTextRenderMode::Native);
    text.frame = UiFrame::new(12.49, 7.51, 120.0, 20.0);
    text.clip_frame = Some(UiFrame::new(12.2, 7.2, 80.0, 20.0));

    let placement = native_text_area_placement(crate::core::math::UVec2::new(200, 80), &text);

    assert_eq!(placement.left, 12.0);
    assert_eq!(placement.top, 8.0);
    assert_eq!(placement.bounds.left, 12);
    assert_eq!(placement.bounds.top, 7);
    assert_eq!(placement.bounds.right, 93);
    assert_eq!(placement.bounds.bottom, 28);
}

#[test]
fn native_text_area_placement_drops_non_finite_origin_values() {
    let mut text = text_batch("folder-open.svg", UiTextRenderMode::Native);
    text.frame = UiFrame::new(f32::NAN, f32::INFINITY, 120.0, 20.0);

    let placement = native_text_area_placement(crate::core::math::UVec2::new(200, 80), &text);

    assert_eq!(placement.left, 0.0);
    assert_eq!(placement.top, 0.0);
    assert_eq!(placement.bounds.left, 0);
    assert_eq!(placement.bounds.top, 0);
}
