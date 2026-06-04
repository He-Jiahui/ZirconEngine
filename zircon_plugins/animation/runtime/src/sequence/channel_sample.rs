use zircon_runtime::asset::{
    AnimationChannelAsset, AnimationChannelValueAsset, AnimationInterpolationAsset,
};
use zircon_runtime::core::math::Real;

use super::interpolation::sample_hermite;

pub(crate) trait AnimationChannelSampleExt {
    fn sample(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset>;
}

impl AnimationChannelSampleExt for AnimationChannelAsset {
    fn sample(&self, time_seconds: Real) -> Option<AnimationChannelValueAsset> {
        if !time_seconds.is_finite() || self.keys.iter().any(|key| !key.time_seconds.is_finite()) {
            return None;
        }

        let first = self.keys.first()?;
        if self.keys.len() == 1 || time_seconds <= first.time_seconds {
            return Some(first.value.clone());
        }
        let last = self.keys.last()?;
        if time_seconds >= last.time_seconds {
            return Some(last.value.clone());
        }

        for pair in self.keys.windows(2) {
            let left = &pair[0];
            let right = &pair[1];
            if time_seconds < left.time_seconds || time_seconds > right.time_seconds {
                continue;
            }
            return Some(match self.interpolation {
                AnimationInterpolationAsset::Step => left.value.clone(),
                AnimationInterpolationAsset::Hermite => sample_hermite(left, right, time_seconds),
            });
        }

        Some(last.value.clone())
    }
}
