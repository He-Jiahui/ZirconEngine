use crate::core::framework::render::RenderBudgetKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderPassStage {
    DepthPrepass,
    Shadow,
    Deferred,
    AmbientOcclusion,
    Lighting,
    Opaque2d,
    AlphaMask2d,
    Transparent2d,
    Opaque3d,
    AlphaMask3d,
    Transparent3d,
    Opaque,
    Transparent,
    PostProcess,
    Overlay,
    Debug,
    Ui,
    Present,
}

impl RenderPassStage {
    pub const ALL: &[Self] = &[
        Self::DepthPrepass,
        Self::Shadow,
        Self::Deferred,
        Self::AmbientOcclusion,
        Self::Lighting,
        Self::Opaque2d,
        Self::AlphaMask2d,
        Self::Transparent2d,
        Self::Opaque3d,
        Self::AlphaMask3d,
        Self::Transparent3d,
        Self::Opaque,
        Self::Transparent,
        Self::PostProcess,
        Self::Overlay,
        Self::Debug,
        Self::Ui,
        Self::Present,
    ];

    pub(crate) const COUNT: usize = Self::ALL.len();

    pub const RENDERER_DATA_AUTHORING_STAGES: &[Self] = &[
        Self::DepthPrepass,
        Self::Shadow,
        Self::Deferred,
        Self::AmbientOcclusion,
        Self::Lighting,
        Self::Opaque2d,
        Self::AlphaMask2d,
        Self::Transparent2d,
        Self::Opaque3d,
        Self::AlphaMask3d,
        Self::Transparent3d,
        Self::PostProcess,
        // Keep declared screen-space content ordered with the runtime graph tail.
        Self::Overlay,
        Self::Debug,
        Self::Ui,
    ];

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::DepthPrepass => 0,
            Self::Shadow => 1,
            Self::Deferred => 2,
            Self::AmbientOcclusion => 3,
            Self::Lighting => 4,
            Self::Opaque2d => 5,
            Self::AlphaMask2d => 6,
            Self::Transparent2d => 7,
            Self::Opaque3d => 8,
            Self::AlphaMask3d => 9,
            Self::Transparent3d => 10,
            Self::Opaque => 11,
            Self::Transparent => 12,
            Self::PostProcess => 13,
            Self::Overlay => 14,
            Self::Debug => 15,
            Self::Ui => 16,
            Self::Present => 17,
        }
    }

    pub(crate) const fn frame_profile_budget_key(self) -> RenderBudgetKey {
        match self {
            Self::DepthPrepass => RenderBudgetKey::DepthPrepass,
            Self::Shadow => RenderBudgetKey::Shadow,
            Self::Deferred
            | Self::Opaque2d
            | Self::AlphaMask2d
            | Self::Opaque3d
            | Self::AlphaMask3d
            | Self::Opaque => RenderBudgetKey::BasePass,
            Self::AmbientOcclusion => RenderBudgetKey::Ssao,
            Self::Lighting => RenderBudgetKey::DeferredLighting,
            Self::Transparent2d | Self::Transparent3d | Self::Transparent => {
                RenderBudgetKey::Transparent
            }
            Self::PostProcess => RenderBudgetKey::PostProcess,
            Self::Ui | Self::Overlay | Self::Debug | Self::Present => RenderBudgetKey::Ui,
        }
    }

    pub const fn authoring_name(self) -> &'static str {
        match self {
            Self::DepthPrepass => "DepthPrepass",
            Self::Shadow => "Shadow",
            Self::Deferred => "Deferred",
            Self::AmbientOcclusion => "AmbientOcclusion",
            Self::Lighting => "Lighting",
            Self::Opaque2d => "Opaque2d",
            Self::AlphaMask2d => "AlphaMask2d",
            Self::Transparent2d => "Transparent2d",
            Self::Opaque3d => "Opaque3d",
            Self::AlphaMask3d => "AlphaMask3d",
            Self::Transparent3d => "Transparent3d",
            Self::Opaque => "Opaque",
            Self::Transparent => "Transparent",
            Self::PostProcess => "PostProcess",
            Self::Ui => "Ui",
            Self::Overlay => "Overlay",
            Self::Debug => "Debug",
            Self::Present => "Present",
        }
    }

    pub fn from_renderer_data_authoring_name(value: &str) -> Option<Self> {
        Self::RENDERER_DATA_AUTHORING_STAGES
            .iter()
            .copied()
            .find(|stage| stage.authoring_name() == value)
    }
}

#[cfg(test)]
mod tests {
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
        assert!(
            !RenderPassStage::RENDERER_DATA_AUTHORING_STAGES.contains(&RenderPassStage::Present)
        );

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
}
