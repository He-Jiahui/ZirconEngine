use zircon_runtime::core::framework::animation::AnimationGpuSkinningReadiness;

use super::MAX_SKIN_JOINTS;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimationGpuSkinningDecision {
    Gpu,
    CpuFallback { diagnostic: String },
}

impl AnimationGpuSkinningDecision {
    pub fn select(readiness: &AnimationGpuSkinningReadiness, joint_count: usize) -> Self {
        if joint_count > MAX_SKIN_JOINTS {
            return Self::CpuFallback {
                diagnostic: format!("GPU skinning supports at most {MAX_SKIN_JOINTS} joints"),
            };
        }
        if !readiness.ready_for_gpu_skinning() {
            return Self::CpuFallback {
                diagnostic: "GPU skinning is not ready; using CPU fallback".into(),
            };
        }
        Self::Gpu
    }

    pub const fn is_cpu_fallback(&self) -> bool {
        matches!(self, Self::CpuFallback { .. })
    }

    pub fn diagnostic(&self) -> Option<&str> {
        match self {
            Self::Gpu => None,
            Self::CpuFallback { diagnostic } => Some(diagnostic),
        }
    }
}
