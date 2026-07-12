use crate::{AiBehaviorTickLod, AI_BEHAVIOR_TICK_SYSTEM};

#[test]
fn lod_tiers_reduce_tick_rate() {
    let full = (0..8)
        .filter(|frame| AiBehaviorTickLod::Full.should_tick(*frame, 17))
        .count();
    let half = (0..8)
        .filter(|frame| AiBehaviorTickLod::Half.should_tick(*frame, 17))
        .count();
    let quarter = (0..8)
        .filter(|frame| AiBehaviorTickLod::Quarter.should_tick(*frame, 17))
        .count();

    assert_eq!((full, half, quarter), (8, 4, 2));
    assert_eq!(
        AiBehaviorTickLod::from_distance(10.0),
        AiBehaviorTickLod::Full
    );
    assert_eq!(
        AiBehaviorTickLod::from_distance(30.0),
        AiBehaviorTickLod::Half
    );
    assert_eq!(
        AiBehaviorTickLod::from_distance(100.0),
        AiBehaviorTickLod::Quarter
    );
    assert_eq!(AI_BEHAVIOR_TICK_SYSTEM, "ai.behavior_tick");
}
