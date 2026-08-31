use super::*;
use crate::core::framework::render::{UiRenderNodeIdProjection, UiRenderSubmissionSegment};

#[test]
fn screen_space_ui_plan_cache_reuses_stable_submission_identity() {
    let submission = UiRenderSubmission::single(Arc::new(plan_cache_extract()));
    let mut cache = ScreenSpaceUiPlanCache::default();

    let first = cache
        .prepare(&submission, UVec2::new(320, 180), None, 7)
        .expect("visible command should produce a plan");
    let stable = cache
        .prepare(&submission, UVec2::new(320, 180), None, 7)
        .expect("stable submission should retain its plan");

    assert!(Arc::ptr_eq(&first, &stable));
}

#[test]
fn screen_space_ui_plan_cache_reuses_unchanged_segments_across_submission_wrappers() {
    let extract = plan_cache_frame_extract("stable", 1, 1.0, Some("#112233"));
    let first_submission = UiRenderSubmission::from_frame_segments(vec![Arc::clone(&extract)]);
    let next_submission = UiRenderSubmission::from_frame_segments(vec![extract]);
    let mut cache = ScreenSpaceUiPlanCache::default();

    let first = cache
        .prepare(&first_submission, UVec2::new(320, 180), None, 7)
        .expect("initial plan");
    let first_segment = Arc::clone(cache.cached_segment_plan(0).expect("initial segment plan"));
    let next = cache
        .prepare(&next_submission, UVec2::new(320, 180), None, 7)
        .expect("equivalent segmented plan");

    assert!(Arc::ptr_eq(&first, &next));
    assert!(Arc::ptr_eq(
        &first_segment,
        cache.cached_segment_plan(0).expect("retained segment plan")
    ));
}

#[test]
fn screen_space_ui_plan_cache_reuses_unchanged_command_segments_within_surface() {
    let flat = plan_cache_many_command_extract(130);
    let first_frame = Arc::new(UiRenderFrameExtract::from_extract(&flat));
    let mut changed = flat.clone();
    changed.list.commands[65].frame.x += 4.0;
    let (changed_frame, stats) = first_frame
        .patch_ranges_from_extract(&changed, &[65..66])
        .expect("fixed-cardinality patch should retain command leaves");
    let first_submission = UiRenderSubmission::single_frame(first_frame);
    let changed_submission = UiRenderSubmission::single_frame(Arc::new(changed_frame));
    let mut cache = ScreenSpaceUiPlanCache::default();

    let _ = cache.prepare(&first_submission, UVec2::new(320, 180), None, 7);
    let first_plan = Arc::clone(cache.cached_segment_plan(0).expect("first command leaf"));
    let changed_plan = Arc::clone(cache.cached_segment_plan(1).expect("changed command leaf"));
    let last_plan = Arc::clone(cache.cached_segment_plan(2).expect("last command leaf"));

    let _ = cache.prepare(&changed_submission, UVec2::new(320, 180), None, 7);

    assert_eq!(stats.cloned_segment_count, 1);
    assert!(Arc::ptr_eq(
        &first_plan,
        cache.cached_segment_plan(0).expect("reused first leaf")
    ));
    assert!(!Arc::ptr_eq(
        &changed_plan,
        cache.cached_segment_plan(1).expect("rebuilt middle leaf")
    ));
    assert!(Arc::ptr_eq(
        &last_plan,
        cache.cached_segment_plan(2).expect("reused last leaf")
    ));
}

#[test]
fn screen_space_ui_plan_cache_preserves_suffix_when_changed_segment_has_same_background_effects() {
    let first = plan_cache_frame_extract("first", 1, 0.0, None);
    let middle = plan_cache_frame_extract("middle", 2, 0.0, None);
    let changed_middle = plan_cache_frame_extract("middle.changed", 3, 0.0, None);
    let last = plan_cache_frame_extract("last", 4, 0.0, None);
    let first_submission = UiRenderSubmission::from_frame_segments(vec![
        Arc::clone(&first),
        middle,
        Arc::clone(&last),
    ]);
    let next_submission =
        UiRenderSubmission::from_frame_segments(vec![first, changed_middle, last]);
    let mut cache = ScreenSpaceUiPlanCache::default();
    let _ = cache.prepare(&first_submission, UVec2::new(320, 180), None, 7);
    let first_plan = Arc::clone(cache.cached_segment_plan(0).expect("first plan"));
    let middle_plan = Arc::clone(cache.cached_segment_plan(1).expect("middle plan"));
    let last_plan = Arc::clone(cache.cached_segment_plan(2).expect("last plan"));

    let _ = cache.prepare(&next_submission, UVec2::new(320, 180), None, 7);

    assert!(Arc::ptr_eq(
        &first_plan,
        cache.cached_segment_plan(0).expect("reused first plan")
    ));
    assert!(!Arc::ptr_eq(
        &middle_plan,
        cache.cached_segment_plan(1).expect("rebuilt middle plan")
    ));
    assert!(Arc::ptr_eq(
        &last_plan,
        cache.cached_segment_plan(2).expect("reused suffix plan")
    ));
}

#[test]
fn screen_space_ui_plan_cache_invalidates_suffix_when_background_effects_change() {
    let first = plan_cache_frame_extract("first", 1, 0.0, None);
    let middle = plan_cache_frame_extract("middle", 2, 0.0, None);
    let changed_middle = plan_cache_frame_extract("middle.changed", 3, 1.0, Some("#112233"));
    let last = plan_cache_frame_extract("last", 4, 0.0, None);
    let first_submission = UiRenderSubmission::from_frame_segments(vec![
        Arc::clone(&first),
        middle,
        Arc::clone(&last),
    ]);
    let next_submission =
        UiRenderSubmission::from_frame_segments(vec![first, changed_middle, last]);
    let mut cache = ScreenSpaceUiPlanCache::default();
    let _ = cache.prepare(&first_submission, UVec2::new(320, 180), None, 7);
    let first_plan = Arc::clone(cache.cached_segment_plan(0).expect("first plan"));
    let middle_plan = Arc::clone(cache.cached_segment_plan(1).expect("middle plan"));
    let last_plan = Arc::clone(cache.cached_segment_plan(2).expect("last plan"));

    let _ = cache.prepare(&next_submission, UVec2::new(320, 180), None, 7);

    assert!(Arc::ptr_eq(
        &first_plan,
        cache.cached_segment_plan(0).expect("reused first plan")
    ));
    assert!(!Arc::ptr_eq(
        &middle_plan,
        cache.cached_segment_plan(1).expect("rebuilt middle plan")
    ));
    assert!(!Arc::ptr_eq(
        &last_plan,
        cache
            .cached_segment_plan(2)
            .expect("invalidated suffix plan")
    ));
}

#[test]
fn screen_space_ui_segment_plan_composition_retains_local_draw_ranges() {
    let first = plan_cache_frame_extract("first", 1, 1.0, Some("#112233"));
    let second = plan_cache_frame_extract("second", 2, 1.0, Some("#445566"));
    let submission = UiRenderSubmission::from_frame_segments(vec![first, second]);
    let mut cache = ScreenSpaceUiPlanCache::default();

    let prepared = cache
        .prepare(&submission, UVec2::new(320, 180), None, 7)
        .expect("two visible segments should produce a plan");

    assert_eq!(prepared.render_segments.len(), 2);
    for segment in prepared.render_segments.iter() {
        assert_eq!(segment.vertices.len(), 6);
        assert_eq!(segment.draws.len(), 1);
        assert_eq!(segment.draws[0].vertices, 0..6);
    }
}

#[test]
fn screen_space_ui_plan_cache_invalidates_route_and_projection_domains() {
    let extract = plan_cache_frame_extract("local", 7, 1.0, Some("#112233"));
    let projection_a = UiRenderNodeIdProjection::new(1_u64 << 48, (1_u64 << 48) - 1);
    let projection_b = UiRenderNodeIdProjection::new(2_u64 << 48, (1_u64 << 48) - 1);
    let first_submission =
        UiRenderSubmission::from_submission_segments(vec![UiRenderSubmissionSegment::projected(
            Arc::clone(&extract),
            UiTreeId::new("route.a"),
            projection_a,
        )]);
    let route_submission =
        UiRenderSubmission::from_submission_segments(vec![UiRenderSubmissionSegment::projected(
            Arc::clone(&extract),
            UiTreeId::new("route.b"),
            projection_a,
        )]);
    let projection_submission =
        UiRenderSubmission::from_submission_segments(vec![UiRenderSubmissionSegment::projected(
            extract,
            UiTreeId::new("route.b"),
            projection_b,
        )]);
    let mut cache = ScreenSpaceUiPlanCache::default();
    let _ = cache.prepare(&first_submission, UVec2::new(320, 180), None, 7);
    let first_plan = Arc::clone(cache.cached_segment_plan(0).expect("first route plan"));

    let _ = cache.prepare(&route_submission, UVec2::new(320, 180), None, 7);
    let route_plan = Arc::clone(cache.cached_segment_plan(0).expect("changed route plan"));
    let _ = cache.prepare(&projection_submission, UVec2::new(320, 180), None, 7);
    let projection_plan = cache
        .cached_segment_plan(0)
        .expect("changed projection plan");

    assert!(!Arc::ptr_eq(&first_plan, &route_plan));
    assert!(!Arc::ptr_eq(&route_plan, projection_plan));
}

#[test]
fn screen_space_ui_segment_plan_composition_retains_cached_prefix_and_suffix() {
    let first = plan_cache_frame_extract("first", 1, 1.0, Some("#112233"));
    let middle = plan_cache_frame_extract("middle", 2, 1.0, Some("#445566"));
    let changed_middle = plan_cache_frame_extract("middle.changed", 3, 1.0, Some("#445566"));
    let last = plan_cache_frame_extract("last", 4, 1.0, Some("#778899"));
    let first_submission = UiRenderSubmission::from_frame_segments(vec![
        Arc::clone(&first),
        middle,
        Arc::clone(&last),
    ]);
    let next_submission =
        UiRenderSubmission::from_frame_segments(vec![first, changed_middle, last]);
    let mut cache = ScreenSpaceUiPlanCache::default();
    let _ = cache.prepare(&first_submission, UVec2::new(320, 180), None, 7);
    let first_plan = Arc::clone(cache.cached_segment_plan(0).expect("first plan"));
    let last_plan = Arc::clone(cache.cached_segment_plan(2).expect("last plan"));

    let prepared = cache
        .prepare(&next_submission, UVec2::new(320, 180), None, 7)
        .expect("mixed cached plan");

    assert!(Arc::ptr_eq(
        &first_plan,
        cache.cached_segment_plan(0).expect("cached prefix")
    ));
    assert!(Arc::ptr_eq(
        &last_plan,
        cache.cached_segment_plan(2).expect("cached suffix")
    ));
    assert_eq!(prepared.render_segments.len(), 3);
    assert!(Arc::ptr_eq(&first_plan, &prepared.render_segments[0]));
    assert!(Arc::ptr_eq(&last_plan, &prepared.render_segments[2]));
    for segment in prepared.render_segments.iter() {
        assert_eq!(segment.vertices.len(), 6);
        assert_eq!(segment.draws.len(), 1);
        assert_eq!(segment.draws[0].vertices, 0..6);
    }
}

#[test]
fn screen_space_ui_plan_cache_rebuilds_suffix_text_with_changed_prefix_background() {
    let first_background = plan_cache_frame_extract("background", 1, 1.0, Some("#112233"));
    let next_background = plan_cache_frame_extract("background.changed", 1, 1.0, Some("#445566"));
    let text = plan_cache_text_frame_extract("text", 2, "cached suffix");
    let first_submission =
        UiRenderSubmission::from_frame_segments(vec![first_background, Arc::clone(&text)]);
    let next_submission = UiRenderSubmission::from_frame_segments(vec![next_background, text]);
    let mut cache = ScreenSpaceUiPlanCache::default();

    let first = cache
        .prepare(&first_submission, UVec2::new(320, 180), None, 7)
        .expect("first text plan");
    let next = cache
        .prepare(&next_submission, UVec2::new(320, 180), None, 7)
        .expect("updated text plan");

    assert_eq!(
        first.native_texts[0].background_color,
        Some([
            0x11 as f32 / 255.0,
            0x22 as f32 / 255.0,
            0x33 as f32 / 255.0,
            1.0
        ])
    );
    assert_eq!(
        next.native_texts[0].background_color,
        Some([
            0x44 as f32 / 255.0,
            0x55 as f32 / 255.0,
            0x66 as f32 / 255.0,
            1.0
        ])
    );
}

#[test]
fn screen_space_ui_plan_cache_invalidates_each_planner_input() {
    let extract = Arc::new(plan_cache_extract());
    let first_submission = UiRenderSubmission::single(Arc::clone(&extract));
    let next_submission = UiRenderSubmission::single(extract);
    let mut cache = ScreenSpaceUiPlanCache::default();
    let first = cache
        .prepare(&first_submission, UVec2::new(320, 180), None, 7)
        .expect("initial plan");
    let new_submission = cache
        .prepare(&next_submission, UVec2::new(320, 180), None, 7)
        .expect("new submission plan");
    let new_viewport = cache
        .prepare(&next_submission, UVec2::new(640, 360), None, 7)
        .expect("new viewport plan");
    let new_background = cache
        .prepare(
            &next_submission,
            UVec2::new(640, 360),
            Some([0.1, 0.2, 0.3, 1.0]),
            7,
        )
        .expect("new background plan");
    let new_font_generation = cache
        .prepare(
            &next_submission,
            UVec2::new(640, 360),
            Some([0.1, 0.2, 0.3, 1.0]),
            8,
        )
        .expect("new font-generation plan");

    assert!(!Arc::ptr_eq(&first, &new_submission));
    assert!(!Arc::ptr_eq(&new_submission, &new_viewport));
    assert!(!Arc::ptr_eq(&new_viewport, &new_background));
    assert!(!Arc::ptr_eq(&new_background, &new_font_generation));
}

#[test]
fn screen_space_ui_plan_cache_rejects_foreign_font_collection_at_same_generation() {
    let submission = UiRenderSubmission::single(Arc::new(plan_cache_extract()));
    let first_collection = crate::text::font::FontCollectionService::from_database(
        crate::text::font::runtime_default_font_database_for_test(),
    );
    let second_collection = crate::text::font::FontCollectionService::from_database(
        crate::text::font::runtime_default_font_database_for_test(),
    );
    assert_eq!(
        first_collection.generation(),
        second_collection.generation()
    );
    let mut cache = ScreenSpaceUiPlanCache::default();

    let first = cache
        .prepare_with_font_revision(
            &submission,
            UVec2::new(320, 180),
            None,
            first_collection.revision(),
        )
        .expect("first collection plan");
    let foreign = cache
        .prepare_with_font_revision(
            &submission,
            UVec2::new(320, 180),
            None,
            second_collection.revision(),
        )
        .expect("foreign collection plan");

    assert!(!Arc::ptr_eq(&first, &foreign));
}

#[test]
fn screen_space_ui_vertex_plan_reuse_requires_exact_plan_identity() {
    let extract = Arc::new(plan_cache_extract());
    let first_submission = UiRenderSubmission::single(Arc::clone(&extract));
    let next_submission = UiRenderSubmission::single(extract);
    let mut cache = ScreenSpaceUiPlanCache::default();
    let first = cache
        .prepare(&first_submission, UVec2::new(320, 180), None, 7)
        .expect("initial plan");
    let stable = cache
        .prepare(&first_submission, UVec2::new(320, 180), None, 7)
        .expect("stable plan");
    let next = cache
        .prepare(&next_submission, UVec2::new(320, 180), None, 7)
        .expect("new submission plan");

    let first_identity = Arc::downgrade(&first);
    assert!(record::screen_space_ui_vertex_plan_reused(
        Some(&first_identity),
        &stable
    ));
    assert!(!record::screen_space_ui_vertex_plan_reused(
        Some(&first_identity),
        &next
    ));
    assert!(!record::screen_space_ui_vertex_plan_reused(None, &first));
}

#[test]
fn screen_space_ui_vertex_segment_plan_reuse_requires_exact_segment_identity() {
    let first = plan_cache_frame_extract("first", 1, 1.0, Some("#112233"));
    let middle = plan_cache_frame_extract("middle", 2, 1.0, Some("#445566"));
    let changed_middle = plan_cache_frame_extract("middle.changed", 3, 1.0, Some("#445566"));
    let last = plan_cache_frame_extract("last", 4, 1.0, Some("#778899"));
    let first_submission = UiRenderSubmission::from_frame_segments(vec![
        Arc::clone(&first),
        middle,
        Arc::clone(&last),
    ]);
    let next_submission =
        UiRenderSubmission::from_frame_segments(vec![first, changed_middle, last]);
    let mut cache = ScreenSpaceUiPlanCache::default();
    let _ = cache.prepare(&first_submission, UVec2::new(320, 180), None, 7);
    let first_plan = Arc::clone(cache.cached_segment_plan(0).expect("first plan"));
    let middle_plan = Arc::clone(cache.cached_segment_plan(1).expect("middle plan"));
    let last_plan = Arc::clone(cache.cached_segment_plan(2).expect("last plan"));
    let first_identity = Arc::downgrade(&first_plan);
    let middle_identity = Arc::downgrade(&middle_plan);
    let last_identity = Arc::downgrade(&last_plan);

    let _ = cache.prepare(&next_submission, UVec2::new(320, 180), None, 7);
    let retained_first = cache.cached_segment_plan(0).expect("retained first plan");
    let changed_middle = cache.cached_segment_plan(1).expect("changed middle plan");
    let retained_last = cache.cached_segment_plan(2).expect("retained last plan");

    assert!(record::screen_space_ui_vertex_segment_plan_reused(
        Some(&first_identity),
        retained_first
    ));
    assert!(!record::screen_space_ui_vertex_segment_plan_reused(
        Some(&middle_identity),
        changed_middle
    ));
    assert!(record::screen_space_ui_vertex_segment_plan_reused(
        Some(&last_identity),
        retained_last
    ));
    assert!(!record::screen_space_ui_vertex_segment_plan_reused(
        None,
        retained_first
    ));
}

fn plan_cache_extract() -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("runtime.ui.plan-cache"),
        list: UiRenderList {
            commands: vec![UiRenderCommand {
                node_id: UiNodeId::new(1),
                kind: UiRenderCommandKind::Quad,
                frame: UiFrame::new(0.0, 0.0, 32.0, 24.0),
                clip_frame: None,
                z_index: 0,
                style: UiResolvedStyle::default(),
                text_layout: None,
                text: None,
                image: None,
                opacity: 1.0,
            }],
        },
        raster_scale: 1.0,
    }
}

fn plan_cache_many_command_extract(command_count: usize) -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("runtime.ui.plan-cache.leaves"),
        list: UiRenderList {
            commands: (0..command_count)
                .map(|index| UiRenderCommand {
                    node_id: UiNodeId::new(index as u64 + 1),
                    kind: UiRenderCommandKind::Image,
                    frame: UiFrame::new(index as f32, 0.0, 16.0, 16.0),
                    clip_frame: None,
                    z_index: index as i32,
                    style: UiResolvedStyle::default(),
                    text_layout: None,
                    text: None,
                    image: None,
                    opacity: 1.0,
                })
                .collect(),
        },
        raster_scale: 1.0,
    }
}

fn plan_cache_frame_extract(
    tree_id: &str,
    node_id: u64,
    opacity: f32,
    background_color: Option<&str>,
) -> Arc<UiRenderFrameExtract> {
    Arc::new(UiRenderFrameExtract::from_extract(&UiRenderExtract {
        tree_id: UiTreeId::new(tree_id),
        list: UiRenderList {
            commands: vec![UiRenderCommand {
                node_id: UiNodeId::new(node_id),
                kind: UiRenderCommandKind::Quad,
                frame: UiFrame::new(0.0, 0.0, 32.0, 24.0),
                clip_frame: None,
                z_index: 0,
                style: UiResolvedStyle {
                    background_color: background_color.map(str::to_string),
                    ..UiResolvedStyle::default()
                },
                text_layout: None,
                text: None,
                image: None,
                opacity,
            }],
        },
        raster_scale: 1.0,
    }))
}

fn plan_cache_text_frame_extract(
    tree_id: &str,
    node_id: u64,
    text: &str,
) -> Arc<UiRenderFrameExtract> {
    Arc::new(UiRenderFrameExtract::from_extract(&UiRenderExtract {
        tree_id: UiTreeId::new(tree_id),
        list: UiRenderList {
            commands: vec![UiRenderCommand {
                node_id: UiNodeId::new(node_id),
                kind: UiRenderCommandKind::Text,
                frame: UiFrame::new(0.0, 0.0, 32.0, 24.0),
                clip_frame: None,
                z_index: 0,
                style: UiResolvedStyle {
                    text_render_mode: UiTextRenderMode::Native,
                    ..UiResolvedStyle::default()
                },
                text_layout: None,
                text: Some(text.to_string()),
                image: None,
                opacity: 1.0,
            }],
        },
        raster_scale: 1.0,
    }))
}
