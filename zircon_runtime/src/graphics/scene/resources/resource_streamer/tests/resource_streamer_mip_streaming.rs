use std::collections::HashMap;

use crate::core::resource::ResourceId;

use super::*;

fn demand(
    label: &str,
    mip_count: u8,
    resident_mips: core::ops::Range<u8>,
    screen_coverage: u16,
    streaming_enabled: bool,
    stable_order: u64,
) -> MipStreamingDemand {
    MipStreamingDemand {
        texture: ResourceId::from_stable_label(label),
        mip_count,
        resident_mips,
        screen_coverage,
        streaming_enabled,
        stable_order,
        upload_bytes: 0,
        resident_bytes: 0,
        wanted_bytes: 0,
        forced_eviction_bytes: 0,
    }
}

impl MipStreamingDemand {
    fn with_upload_bytes(mut self, upload_bytes: u64) -> Self {
        self.upload_bytes = upload_bytes;
        self
    }

    fn with_resident_bytes(mut self, resident_bytes: u64, wanted_bytes: u64) -> Self {
        self.resident_bytes = resident_bytes;
        self.wanted_bytes = wanted_bytes;
        self.forced_eviction_bytes = wanted_bytes;
        self
    }
}

#[test]
fn render_mip_streaming_wanted_mip_tracks_screen_coverage_and_bias() {
    assert_eq!(wanted_mip_start(6, u16::MAX, 0), 0);
    assert_eq!(wanted_mip_start(6, u16::MAX / 4, 0), 1);
    assert_eq!(wanted_mip_start(6, u16::MAX / 16, 0), 2);
    assert_eq!(wanted_mip_start(6, u16::MAX / 16, 2), 4);
    assert_eq!(wanted_mip_start(6, 0, 0), 5);
}

#[test]
fn render_mip_streaming_prioritizes_visible_promotions_with_a_bounded_queue() {
    let near = demand("mip-streaming-near", 6, 2..6, u16::MAX, true, 7);
    let far = demand("mip-streaming-far", 6, 0..6, u16::MAX / 16, true, 3);

    let plans = plan_mip_streaming(
        [near, far],
        MipStreamingSettings {
            max_transitions: 1,
            max_upload_bytes: u64::MAX,
            max_resident_bytes: u64::MAX,
            current_resident_bytes: 0,
            hysteresis_mips: 0,
            mip_bias: 0,
        },
    );

    assert_eq!(plans.len(), 1);
    assert_eq!(
        plans[0].texture,
        ResourceId::from_stable_label("mip-streaming-near")
    );
    assert_eq!(plans[0].resident_mips, 2..6);
    assert_eq!(plans[0].wanted_mips, 0..6);
}

#[test]
fn render_mip_streaming_defers_promotions_that_exceed_the_frame_upload_budget() {
    let large = demand("mip-streaming-large", 6, 2..6, u16::MAX, true, 0).with_upload_bytes(64);
    let small = demand("mip-streaming-small", 6, 2..6, u16::MAX / 2, true, 1).with_upload_bytes(16);

    let plans = plan_mip_streaming(
        [large, small],
        MipStreamingSettings {
            max_transitions: 2,
            max_upload_bytes: 32,
            max_resident_bytes: u64::MAX,
            current_resident_bytes: 0,
            hysteresis_mips: 0,
            mip_bias: 0,
        },
    );

    assert_eq!(plans.len(), 1);
    assert_eq!(
        plans[0].texture,
        ResourceId::from_stable_label("mip-streaming-small")
    );
    assert_eq!(plans[0].upload_bytes, 16);
}

#[test]
fn render_mip_streaming_respects_persistent_texture_budget_before_promotion() {
    let promotion = demand("mip-streaming-budget-promotion", 6, 2..6, u16::MAX, true, 0)
        .with_upload_bytes(32)
        .with_resident_bytes(64, 128);
    let eviction =
        demand("mip-streaming-budget-eviction", 6, 0..6, 0, true, 1).with_resident_bytes(128, 64);
    let plans = plan_mip_streaming(
        [promotion, eviction],
        MipStreamingSettings {
            max_transitions: 2,
            max_upload_bytes: u64::MAX,
            max_resident_bytes: 96,
            current_resident_bytes: 128,
            hysteresis_mips: 0,
            mip_bias: 0,
        },
    );

    assert_eq!(plans.len(), 1);
    assert_eq!(
        plans[0].texture,
        ResourceId::from_stable_label("mip-streaming-budget-eviction")
    );
    assert!(plans[0].wanted_bytes < plans[0].resident_bytes);
}

#[test]
fn render_mip_streaming_evicts_lowest_priority_texture_before_global_mip_bias() {
    let high_priority = demand("mip-streaming-budget-high", 6, 0..6, u16::MAX, true, 0)
        .with_resident_bytes(128, 128);
    let low_priority =
        demand("mip-streaming-budget-low", 6, 0..6, 0, true, 1).with_resident_bytes(128, 64);
    let plans = plan_mip_streaming(
        [high_priority, low_priority],
        MipStreamingSettings {
            max_transitions: 1,
            max_upload_bytes: u64::MAX,
            max_resident_bytes: 96,
            current_resident_bytes: 256,
            hysteresis_mips: 0,
            mip_bias: 0,
        },
    );

    assert_eq!(plans.len(), 1);
    assert_eq!(
        plans[0].texture,
        ResourceId::from_stable_label("mip-streaming-budget-low")
    );
    assert!(plans[0].wanted_mips.start > plans[0].resident_mips.start);
}

#[test]
fn render_mip_streaming_hysteresis_avoids_single_mip_thrash() {
    let demand = demand("mip-streaming-hysteresis", 5, 1..5, u16::MAX, true, 0);

    assert!(plan_mip_streaming(
        [demand.clone()],
        MipStreamingSettings {
            max_transitions: 1,
            max_upload_bytes: u64::MAX,
            max_resident_bytes: u64::MAX,
            current_resident_bytes: 0,
            hysteresis_mips: 1,
            mip_bias: 0,
        },
    )
    .is_empty());

    let plans = plan_mip_streaming(
        [demand],
        MipStreamingSettings {
            max_transitions: 1,
            max_upload_bytes: u64::MAX,
            max_resident_bytes: u64::MAX,
            current_resident_bytes: 0,
            hysteresis_mips: 0,
            mip_bias: 0,
        },
    );
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].wanted_mips, 0..5);
}

#[test]
fn render_mip_streaming_keeps_the_tail_mip_resident() {
    let plans = plan_mip_streaming(
        [demand("mip-streaming-tail", 5, 9..10, u16::MAX, true, 0)],
        MipStreamingSettings {
            max_transitions: 1,
            max_upload_bytes: u64::MAX,
            max_resident_bytes: u64::MAX,
            current_resident_bytes: 0,
            hysteresis_mips: 0,
            mip_bias: 0,
        },
    );

    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].resident_mips, 4..5);
    assert_eq!(plans[0].wanted_mips, 0..5);
}

#[test]
fn render_mip_streaming_disabled_texture_restores_full_residency_without_hysteresis() {
    let plans = plan_mip_streaming(
        [demand("mip-streaming-disabled", 5, 3..5, 0, false, 0)],
        MipStreamingSettings {
            max_transitions: 1,
            max_upload_bytes: u64::MAX,
            max_resident_bytes: u64::MAX,
            current_resident_bytes: 0,
            hysteresis_mips: u8::MAX,
            mip_bias: 4,
        },
    );

    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].resident_mips, 3..5);
    assert_eq!(plans[0].wanted_mips, 0..5);
}

#[test]
fn render_mip_streaming_coalesces_visible_instances_by_texture_and_coverage() {
    let texture = ResourceId::from_stable_label("mip-streaming-shared");
    let other_texture = ResourceId::from_stable_label("mip-streaming-other");
    let coalesced = coalesce_mip_streaming_visibility([
        MipStreamingVisibility {
            texture,
            screen_coverage: 2_000,
            stable_order: 9,
        },
        MipStreamingVisibility {
            texture,
            screen_coverage: 6_000,
            stable_order: 8,
        },
        MipStreamingVisibility {
            texture,
            screen_coverage: 6_000,
            stable_order: 3,
        },
        MipStreamingVisibility {
            texture: other_texture,
            screen_coverage: 4_000,
            stable_order: 4,
        },
    ]);

    assert_eq!(coalesced.len(), 2);
    let selected = coalesced.get(&texture).expect("shared texture is retained");
    assert_eq!(selected.screen_coverage, 6_000);
    assert_eq!(selected.stable_order, 3);
}

#[test]
fn render_mip_streaming_keeps_offscreen_resident_textures_eligible_for_budget_eviction() {
    let visible = ResourceId::from_stable_label("mip-streaming-visible");
    let offscreen = ResourceId::from_stable_label("mip-streaming-offscreen");
    let visibility = include_non_visible_resident_texture_visibility(
        HashMap::from([(
            visible,
            MipStreamingVisibility {
                texture: visible,
                screen_coverage: u16::MAX,
                stable_order: 7,
            },
        )]),
        [offscreen, visible],
    );

    assert_eq!(visibility.len(), 2);
    assert_eq!(visibility[&visible].screen_coverage, u16::MAX);
    assert_eq!(visibility[&offscreen].screen_coverage, 0);
    assert!(visibility[&offscreen].stable_order > visibility[&visible].stable_order);
}

#[test]
fn render_mip_streaming_state_commits_only_a_successful_promotion() {
    let plan = MipStreamingPlan {
        texture: ResourceId::from_stable_label("mip-streaming-state"),
        resident_mips: 2..6,
        wanted_mips: 0..6,
        priority: u32::from(u16::MAX),
        upload_bytes: 0,
        resident_bytes: 0,
        wanted_bytes: 0,
    };
    let mut state = MipStreamingState::default();

    let task = state
        .begin(plan.clone())
        .expect("the initial promotion is scheduled");
    assert_eq!(task.kind, MipStreamingTransitionKind::Promotion);
    assert!(state.begin(plan.clone()).is_none());
    assert_eq!(state.finish(&task, false), None);

    let retry = state.begin(plan).expect("a failed task can be retried");
    assert_eq!(state.finish(&retry, true), Some(0..6));
}

#[test]
fn render_mip_streaming_state_classifies_eviction_and_rejects_stale_completion() {
    let plan = MipStreamingPlan {
        texture: ResourceId::from_stable_label("mip-streaming-eviction"),
        resident_mips: 0..6,
        wanted_mips: 3..6,
        priority: 0,
        upload_bytes: 0,
        resident_bytes: 0,
        wanted_bytes: 0,
    };
    let mut state = MipStreamingState::default();
    let first = state.begin(plan.clone()).expect("first task is scheduled");
    assert_eq!(first.kind, MipStreamingTransitionKind::Eviction);
    assert_eq!(state.finish(&first, false), None);

    let retry = state.begin(plan).expect("failed eviction is retried");
    assert_eq!(state.finish(&first, true), None);
    assert_eq!(state.finish(&retry, true), Some(3..6));
}

#[test]
fn render_mip_streaming_screen_coverage_tracks_projection_without_understreaming_overrides() {
    let camera = crate::core::framework::render::ViewportCameraSnapshot::default();
    let near =
        quantized_screen_coverage(&camera, crate::core::math::Vec3::new(0.0, 0.0, -4.0), 1.0);
    let far =
        quantized_screen_coverage(&camera, crate::core::math::Vec3::new(0.0, 0.0, -16.0), 1.0);
    assert!(near > far);
    assert!(far > 0);

    let mut orthographic = camera.clone();
    orthographic.projection_mode = crate::core::framework::render::ProjectionMode::Orthographic;
    orthographic.ortho_size = 4.0;
    assert_eq!(
        quantized_screen_coverage(
            &orthographic,
            crate::core::math::Vec3::new(0.0, 0.0, -4.0),
            1.0,
        ),
        quantized_screen_coverage(
            &orthographic,
            crate::core::math::Vec3::new(0.0, 0.0, -32.0),
            1.0,
        )
    );

    orthographic.projection_override = Some(crate::core::math::Mat4::IDENTITY);
    assert_eq!(
        quantized_screen_coverage(
            &orthographic,
            crate::core::math::Vec3::new(0.0, 0.0, -32.0),
            1.0,
        ),
        u16::MAX
    );
}

#[test]
fn render_mip_streaming_rebuild_commits_only_after_matching_success() {
    let source = include_str!("../resource_streamer_mip_streaming.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("mip streaming implementation before tests");
    // BUG: [CR-R02-runtime_wave12_graphics_resource_streamer-0002] 该守卫读取父文件，但重建及发布实现位于 frame_apply 子模块；首个查找返回 None，随后的 expect 必然 panic。
    let rebuild = implementation
        .find("self.rebuild_texture_mip_streaming_task(")
        .expect("streaming task rebuilds a replacement resource");
    let finish = implementation
        .find("self.finish_texture_mip_streaming_task(&task, true)")
        .expect("replacement waits for matching successful completion");
    let publish = implementation
        .find("self.textures.insert(")
        .expect("successful task atomically publishes prepared texture");

    assert!(rebuild < finish);
    assert!(finish < publish);
}
