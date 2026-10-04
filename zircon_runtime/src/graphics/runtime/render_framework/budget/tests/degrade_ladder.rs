use crate::core::framework::render::RenderFrameProfile;

use super::{BudgetDegradeLadder, DegradeStep};
use crate::graphics::runtime::render_framework::budget::GpuMemoryBudget;

#[test]
fn render_perf_degrade_ladder_fixed_order() {
    let budget = GpuMemoryBudget::new(10, 10, 10);
    let over_budget = RenderFrameProfile {
        transient_texture_peak_bytes: 11,
        ..RenderFrameProfile::default()
    };
    let mut ladder = BudgetDegradeLadder::with_hysteresis_frames(2);

    let observed = (0..7)
        .map(|_| ladder.evaluate(&over_budget, &budget).copied())
        .collect::<Vec<_>>();

    assert_eq!(
        observed,
        vec![
            Some(DegradeStep::RenderScale(0.85)),
            Some(DegradeStep::RenderScale(0.7)),
            Some(DegradeStep::GlobalMipBias(1)),
            Some(DegradeStep::DisableFeature("ssr")),
            Some(DegradeStep::DisableFeature("ssao")),
            Some(DegradeStep::DisableFeature("contact_shadow")),
            Some(DegradeStep::DisableFeature("bloom_high")),
        ]
    );
}

#[test]
fn render_perf_degrade_ladder_waits_for_hysteresis_before_recovery() {
    let budget = GpuMemoryBudget::new(10, 10, 10);
    let over_budget = RenderFrameProfile {
        transient_texture_peak_bytes: 11,
        ..RenderFrameProfile::default()
    };
    let under_budget = RenderFrameProfile::default();
    let mut ladder = BudgetDegradeLadder::with_hysteresis_frames(2);
    ladder.evaluate(&over_budget, &budget);
    ladder.evaluate(&over_budget, &budget);

    ladder.evaluate(&under_budget, &budget);
    assert_eq!(ladder.active_level(), 2);
    ladder.evaluate(&under_budget, &budget);
    assert_eq!(ladder.active_level(), 1);
    assert_eq!(ladder.settings().render_scale, 0.85);
}
