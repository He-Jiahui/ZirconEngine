use std::cmp::Ordering;

use zircon_runtime::core::math::Real;

use super::{BlendSpaceCompileError, BlendSpacePoint1D, BlendSpaceWeights2};

#[derive(Clone, Debug, PartialEq)]
pub struct BlendSpace1D {
    points: Box<[BlendSpacePoint1D]>,
}

impl BlendSpace1D {
    pub fn compile(
        points: impl IntoIterator<Item = BlendSpacePoint1D>,
    ) -> Result<Self, BlendSpaceCompileError> {
        let mut points = points.into_iter().collect::<Vec<_>>();
        if points.is_empty() {
            return Err(BlendSpaceCompileError::Empty);
        }
        if points.iter().any(|point| !point.position.is_finite()) {
            return Err(BlendSpaceCompileError::NonFinitePoint);
        }
        points.sort_by(|left, right| left.position.total_cmp(&right.position));
        if points
            .windows(2)
            .any(|pair| pair[0].position == pair[1].position)
        {
            return Err(BlendSpaceCompileError::DuplicatePoint);
        }
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
