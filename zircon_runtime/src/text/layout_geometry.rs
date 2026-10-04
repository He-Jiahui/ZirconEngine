/// Largest integer-valued logical-pixel extent that remains exactly representable by `f32`.
///
/// This is a numeric safety ceiling, not a product-tuned layout limit. Product policy may choose
/// a lower session budget after representative workload profiling.
const DEFAULT_MAX_EXACT_LAYOUT_EXTENT: f32 = 16_777_216.0;

/// Publish finite layout geometry from a wider intermediate without changing ordinary `f32`
/// arithmetic. This is the shared recovery path for finite-input overflow across text layout.
pub(crate) fn finite_geometry(value: f64) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32
    }
}

pub(crate) fn finite_f32_or_geometry(candidate: f32, exact: f64) -> f32 {
    candidate
        .is_finite()
        .then_some(candidate)
        .unwrap_or_else(|| finite_geometry(exact))
}

/// Retains exact accumulation history after the published `f32` value first overflows.
///
/// Before recovery, ordinary `f32` arithmetic remains authoritative. After recovery, every
/// publication is derived from the wider history so a later sign change cannot accidentally use
/// a previously saturated value as if it were the exact sum.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct FiniteGeometryAccumulator {
    value: f32,
    exact: f64,
    recovered: bool,
}

impl FiniteGeometryAccumulator {
    pub(crate) fn add(&mut self, value: f32) -> f32 {
        self.exact += f64::from(value);
        let candidate = self.value + value;
        if self.recovered || !candidate.is_finite() {
            self.recovered = true;
            self.value = finite_geometry(self.exact);
        } else {
            self.value = candidate;
        }
        self.value
    }

    pub(crate) const fn value(&self) -> f32 {
        self.value
    }

    pub(crate) const fn exact(&self) -> f64 {
        self.exact
    }
}

pub(crate) fn finite_sum(values: impl IntoIterator<Item = f32>) -> f32 {
    let mut sum = FiniteGeometryAccumulator::default();
    for value in values {
        sum.add(value);
    }
    sum.value()
}

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
    ResolvedLayoutPublication,
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
        // violation 只记录尝试值与预算上限；调用层再补充 owner 和源范围，形成可归因的拒绝回执。
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
#[path = "tests/layout_geometry.rs"]
mod tests;
