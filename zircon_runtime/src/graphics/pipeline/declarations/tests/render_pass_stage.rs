use super::RenderPassStage;
use crate::core::framework::render::RenderBudgetKey;

#[test]
fn render_perf_budget_table_covers_all_builtin_passes() {
    assert_eq!(
        RenderPassStage::DepthPrepass.frame_profile_budget_key(),
        RenderBudgetKey::DepthPrepass
    );
    assert_eq!(
        RenderPassStage::Lighting.frame_profile_budget_key(),
        RenderBudgetKey::DeferredLighting
    );
    assert_eq!(
        RenderPassStage::Transparent3d.frame_profile_budget_key(),
        RenderBudgetKey::Transparent
    );
    assert_eq!(
        RenderPassStage::Overlay.frame_profile_budget_key(),
        RenderBudgetKey::Ui
    );
    assert_eq!(
        RenderPassStage::Debug.frame_profile_budget_key(),
        RenderBudgetKey::Ui
    );

    let unbudgeted_stages = RenderPassStage::ALL
        .iter()
        .copied()
        .filter(|stage| stage.frame_profile_budget_key() == RenderBudgetKey::Other)
        .collect::<Vec<_>>();
    assert!(
        unbudgeted_stages.is_empty(),
        "every built-in render stage must map to a named frame budget; unbudgeted={unbudgeted_stages:?}"
    );
}

#[test]
fn render_pass_stages_keep_ui_after_overlay_and_debug() {
    assert!(RenderPassStage::Overlay < RenderPassStage::Debug);
    assert!(RenderPassStage::Debug < RenderPassStage::Ui);
    assert!(RenderPassStage::Ui < RenderPassStage::Present);
    assert!(!RenderPassStage::RENDERER_DATA_AUTHORING_STAGES.contains(&RenderPassStage::Present));

    assert_eq!(
        &RenderPassStage::RENDERER_DATA_AUTHORING_STAGES
            [RenderPassStage::RENDERER_DATA_AUTHORING_STAGES.len() - 3..],
        &[
            RenderPassStage::Overlay,
            RenderPassStage::Debug,
            RenderPassStage::Ui,
        ]
    );
}
