use super::graphics_default::GraphicsPreset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphicsTier {
    Low,
    Medium,
    High,
    Ultra,
}

impl GraphicsTier {
    pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Ultra];

    pub const fn auto_governor_default(self) -> bool {
        !matches!(self, Self::Ultra)
    }
}

impl GraphicsPreset {
    pub const fn runtime_tier(self) -> GraphicsTier {
        match self {
            Self::Low => GraphicsTier::Low,
            Self::Medium => GraphicsTier::Medium,
            Self::High | Self::Advanced => GraphicsTier::High,
            Self::Ultra => GraphicsTier::Ultra,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphicsRuntimeBudget {
    pub target_hz: u16,
    pub min_render_scale_desktop: f64,
    pub min_render_scale_mobile: f64,
    pub max_render_scale: f64,
    pub drop_frame_ms: f64,
    pub urgent_frame_ms: f64,
    pub recover_frame_ms: f64,
    pub drop_step: f64,
    pub urgent_drop_step: f64,
    pub recover_step: f64,
    pub recover_stable_seconds: f64,
    pub cooldown_seconds: f64,
}

impl GraphicsRuntimeBudget {
    pub fn presentation_interval_seconds(self) -> f64 {
        1.0 / f64::from(self.target_hz)
    }

    pub const fn min_render_scale(self, mobile: bool) -> f64 {
        if mobile {
            self.min_render_scale_mobile
        } else {
            self.min_render_scale_desktop
        }
    }
}

pub const GRAPHICS_RUNTIME_BUDGETS: [GraphicsRuntimeBudget; 4] = [
    GraphicsRuntimeBudget {
        target_hz: 60,
        min_render_scale_desktop: 0.65,
        min_render_scale_mobile: 0.55,
        max_render_scale: 1.0,
        drop_frame_ms: 22.0,
        urgent_frame_ms: 34.0,
        recover_frame_ms: 17.5,
        drop_step: 0.08,
        urgent_drop_step: 0.12,
        recover_step: 0.06,
        recover_stable_seconds: 6.0,
        cooldown_seconds: 1.1,
    },
    GraphicsRuntimeBudget {
        target_hz: 60,
        min_render_scale_desktop: 0.72,
        min_render_scale_mobile: 0.55,
        max_render_scale: 1.0,
        drop_frame_ms: 24.0,
        urgent_frame_ms: 34.0,
        recover_frame_ms: 17.0,
        drop_step: 0.1,
        urgent_drop_step: 0.15,
        recover_step: 0.05,
        recover_stable_seconds: 7.0,
        cooldown_seconds: 1.35,
    },
    GraphicsRuntimeBudget {
        target_hz: 60,
        min_render_scale_desktop: 0.7,
        min_render_scale_mobile: 0.6,
        max_render_scale: 1.0,
        drop_frame_ms: 22.0,
        urgent_frame_ms: 32.0,
        recover_frame_ms: 15.0,
        drop_step: 0.1,
        urgent_drop_step: 0.15,
        recover_step: 0.05,
        recover_stable_seconds: 3.0,
        cooldown_seconds: 0.85,
    },
    GraphicsRuntimeBudget {
        target_hz: 60,
        min_render_scale_desktop: 0.78,
        min_render_scale_mobile: 0.68,
        max_render_scale: 1.0,
        drop_frame_ms: 24.0,
        urgent_frame_ms: 34.0,
        recover_frame_ms: 15.0,
        drop_step: 0.08,
        urgent_drop_step: 0.12,
        recover_step: 0.04,
        recover_stable_seconds: 3.0,
        cooldown_seconds: 0.85,
    },
];

pub const fn graphics_runtime_budget(tier: GraphicsTier) -> &'static GraphicsRuntimeBudget {
    &GRAPHICS_RUNTIME_BUDGETS[match tier {
        GraphicsTier::Low => 0,
        GraphicsTier::Medium => 1,
        GraphicsTier::High => 2,
        GraphicsTier::Ultra => 3,
    }]
}
