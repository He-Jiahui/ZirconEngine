pub(super) fn hybrid_gi_budget_weight(ray_budget: u32) -> f32 {
    (ray_budget.max(1) as f32 / 8.0).clamp(0.125, 1.0)
}
