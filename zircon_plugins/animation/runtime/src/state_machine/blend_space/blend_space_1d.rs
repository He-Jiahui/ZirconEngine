use std::cmp::Ordering;

use zircon_runtime::core::framework::animation::compiler::state_machine::AnimationCompiledBlendSpace1DSample;
use zircon_runtime::core::math::Real;

use super::{BlendSpaceCompileError, BlendSpaceWeights2};

#[derive(Clone, Copy, Debug, PartialEq)]
struct PreparedPoint1D {
    position: Real,
    sample: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlendSpace1D {
    points: Box<[PreparedPoint1D]>,
}

impl BlendSpace1D {
    pub(super) fn from_compiled(
        samples: &[AnimationCompiledBlendSpace1DSample],
    ) -> Result<Self, BlendSpaceCompileError> {
        let mut points = samples
            .iter()
            .enumerate()
            .map(|(sample, source)| {
                Ok(PreparedPoint1D {
                    position: source.position,
                    sample: u32::try_from(sample)
                        .map_err(|_| BlendSpaceCompileError::CapacityExceeded)?,
                })
            })
            .collect::<Result<Vec<_>, BlendSpaceCompileError>>()?;
        points.sort_by(|left, right| left.position.total_cmp(&right.position));
        Ok(Self {
            points: points.into_boxed_slice(),
        })
    }

    pub fn sample(&self, value: Real) -> Option<BlendSpaceWeights2> {
        let value = value.is_finite().then_some(value)?;
        let first = self.points.first()?;
        let last = self.points.last()?;
        if value <= first.position {
            return Some(BlendSpaceWeights2::new([
                (first.sample, 1.0),
                (first.sample, 0.0),
            ]));
        }
        if value >= last.position {
            return Some(BlendSpaceWeights2::new([
                (last.sample, 1.0),
                (last.sample, 0.0),
            ]));
        }
        let upper = self
            .points
            .partition_point(|point| point.position.total_cmp(&value) != Ordering::Greater);
        let (left, right) = (self.points[upper - 1], self.points[upper]);
        let target = (value - left.position) / (right.position - left.position);
        Some(BlendSpaceWeights2::new([
            (left.sample, 1.0 - target),
            (right.sample, target),
        ]))
    }
}
