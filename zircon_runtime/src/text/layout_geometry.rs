/// Largest integer-valued logical-pixel extent that remains exactly representable by `f32`.
///
/// This is a numeric safety ceiling, not a product-tuned layout limit. Product policy may choose
/// a lower session budget after representative workload profiling.
const DEFAULT_MAX_EXACT_LAYOUT_EXTENT: f32 = 16_777_216.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextLayoutGeometryBudget {
    max_axis_extent: f32,
    max_accumulated_extent: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextLayoutGeometryViolation {
    pub(crate) attempted_extent: f32,
    pub(crate) admitted_extent: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TextLayoutAxisConstraint {
    Bounded(f32),
    Unbounded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextLayoutGeometryOwner {
    IntrinsicMeasurement,
    TableAvailableTrackExtent,
    TablePreferredCell,
    TableColumnTracks,
    TableRowTracks,
    TableCellFrame,
    TableAggregate,
}

impl TextLayoutGeometryBudget {
    pub(crate) fn new(max_axis_extent: f32, max_accumulated_extent: f32) -> Option<Self> {
        if !max_axis_extent.is_finite()
            || max_axis_extent <= 0.0
            || !max_accumulated_extent.is_finite()
            || max_accumulated_extent < max_axis_extent
        {
            return None;
        }
        Some(Self {
            max_axis_extent,
            max_accumulated_extent,
        })
    }

    pub(crate) const fn max_axis_extent(self) -> f32 {
        self.max_axis_extent
    }

    pub(crate) const fn max_accumulated_extent(self) -> f32 {
        self.max_accumulated_extent
    }

    pub(crate) fn admit_axis_extent(self, extent: f32) -> Result<f32, TextLayoutGeometryViolation> {
        self.admit(extent, self.max_axis_extent)
    }

    pub(crate) fn admit_accumulated_extent(
        self,
        extent: f32,
    ) -> Result<f32, TextLayoutGeometryViolation> {
        self.admit(extent, self.max_accumulated_extent)
    }

    pub(crate) fn admit_coordinate(
        self,
        coordinate: f32,
    ) -> Result<f32, TextLayoutGeometryViolation> {
        let magnitude = coordinate.abs();
        if coordinate.is_finite() && magnitude <= self.max_accumulated_extent {
            Ok(coordinate)
        } else {
            Err(self.violation(magnitude, self.max_accumulated_extent))
        }
    }

    pub(crate) fn checked_add_accumulated(
        self,
        left: f32,
        right: f32,
    ) -> Result<f32, TextLayoutGeometryViolation> {
        self.admit_accumulated_extent(left)?;
        self.admit_accumulated_extent(right)?;
        let attempted = f64::from(left) + f64::from(right);
        if attempted > f64::from(self.max_accumulated_extent) {
            return Err(self.violation(attempted as f32, self.max_accumulated_extent));
        }
        self.admit_accumulated_extent(attempted as f32)
    }

    pub(crate) fn checked_scale_accumulated(
        self,
        extent: f32,
        count: usize,
    ) -> Result<f32, TextLayoutGeometryViolation> {
        self.admit_axis_extent(extent)?;
        let attempted = f64::from(extent) * count as f64;
        if !attempted.is_finite() || attempted > f64::from(self.max_accumulated_extent) {
            return Err(self.violation(attempted as f32, self.max_accumulated_extent));
        }
        self.admit_accumulated_extent(attempted as f32)
    }

    fn admit(self, extent: f32, admitted_extent: f32) -> Result<f32, TextLayoutGeometryViolation> {
        if extent.is_finite() && extent >= 0.0 && extent <= admitted_extent {
            Ok(extent)
        } else {
            Err(self.violation(extent, admitted_extent))
        }
    }

    const fn violation(
        self,
        attempted_extent: f32,
        admitted_extent: f32,
    ) -> TextLayoutGeometryViolation {
        TextLayoutGeometryViolation {
            attempted_extent,
            admitted_extent,
        }
    }
}

impl TextLayoutAxisConstraint {
    pub(crate) fn from_request_extent(
        extent: f32,
        budget: TextLayoutGeometryBudget,
    ) -> Result<Self, TextLayoutGeometryViolation> {
        if extent == f32::INFINITY {
            return Ok(Self::Unbounded);
        }
        budget.admit_axis_extent(extent).map(Self::Bounded)
    }

    pub(crate) const fn request_extent(self) -> f32 {
        match self {
            Self::Bounded(extent) => extent,
            Self::Unbounded => f32::INFINITY,
        }
    }

    pub(crate) const fn bounded_extent(self) -> Option<f32> {
        match self {
            Self::Bounded(extent) => Some(extent),
            Self::Unbounded => None,
        }
    }

    pub(crate) fn subtract_accumulated(
        self,
        consumed: f32,
        budget: TextLayoutGeometryBudget,
    ) -> Result<Self, TextLayoutGeometryViolation> {
        budget.admit_accumulated_extent(consumed)?;
        match self {
            Self::Bounded(extent) => Ok(Self::Bounded((extent - consumed).max(0.0))),
            Self::Unbounded => Ok(Self::Unbounded),
        }
    }
}

impl Default for TextLayoutGeometryBudget {
    fn default() -> Self {
        Self {
            max_axis_extent: DEFAULT_MAX_EXACT_LAYOUT_EXTENT,
            max_accumulated_extent: DEFAULT_MAX_EXACT_LAYOUT_EXTENT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_MAX_EXACT_LAYOUT_EXTENT, TextLayoutAxisConstraint, TextLayoutGeometryBudget,
    };

    #[test]
    fn default_budget_uses_the_f32_exact_integer_boundary() {
        let budget = TextLayoutGeometryBudget::default();

        assert_eq!(budget.max_axis_extent(), DEFAULT_MAX_EXACT_LAYOUT_EXTENT);
        assert_eq!(
            budget.max_accumulated_extent(),
            DEFAULT_MAX_EXACT_LAYOUT_EXTENT
        );
    }

    #[test]
    fn budget_rejects_non_finite_negative_and_oversized_axis_extents() {
        let budget = TextLayoutGeometryBudget::new(100.0, 200.0).expect("valid budget");

        assert_eq!(budget.admit_axis_extent(100.0), Ok(100.0));
        assert!(budget.admit_axis_extent(f32::NAN).is_err());
        assert!(budget.admit_axis_extent(f32::INFINITY).is_err());
        assert!(budget.admit_axis_extent(-1.0).is_err());
        assert!(budget.admit_axis_extent(100.5).is_err());
    }

    #[test]
    fn accumulated_arithmetic_rejects_invalid_operands_and_budget_overflow() {
        let budget = TextLayoutGeometryBudget::new(100.0, 200.0).expect("valid budget");

        assert_eq!(budget.checked_add_accumulated(80.0, 120.0), Ok(200.0));
        assert!(budget.checked_add_accumulated(-10.0, 20.0).is_err());
        assert!(budget.checked_add_accumulated(120.0, 81.0).is_err());
        assert_eq!(budget.checked_scale_accumulated(20.0, 10), Ok(200.0));
        assert!(budget.checked_scale_accumulated(20.0, 11).is_err());
        assert!(budget.checked_scale_accumulated(f32::INFINITY, 0).is_err());
    }

    #[test]
    fn constructor_requires_a_positive_finite_ordered_budget() {
        assert!(TextLayoutGeometryBudget::new(0.0, 1.0).is_none());
        assert!(TextLayoutGeometryBudget::new(2.0, 1.0).is_none());
        assert!(TextLayoutGeometryBudget::new(1.0, f32::INFINITY).is_none());
        assert!(TextLayoutGeometryBudget::new(f32::NAN, 1.0).is_none());
    }

    #[test]
    fn positive_infinity_is_only_admitted_as_request_metadata() {
        let budget = TextLayoutGeometryBudget::new(100.0, 200.0).expect("valid budget");

        assert_eq!(
            TextLayoutAxisConstraint::from_request_extent(f32::INFINITY, budget),
            Ok(TextLayoutAxisConstraint::Unbounded)
        );
        assert_eq!(
            TextLayoutAxisConstraint::from_request_extent(75.0, budget),
            Ok(TextLayoutAxisConstraint::Bounded(75.0))
        );
        assert!(TextLayoutAxisConstraint::from_request_extent(f32::NEG_INFINITY, budget).is_err());
        assert!(TextLayoutAxisConstraint::from_request_extent(f32::NAN, budget).is_err());
        assert!(budget.admit_axis_extent(f32::INFINITY).is_err());
    }
}
