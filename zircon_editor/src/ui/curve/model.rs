use std::collections::BTreeSet;

/// A scalar curve-space location where time advances rightward and value advances upward.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CurvePoint {
    pub time: f32,
    pub value: f32,
}

impl CurvePoint {
    pub const ZERO: Self = Self {
        time: 0.0,
        value: 0.0,
    };

    pub const fn new(time: f32, value: f32) -> Self {
        Self { time, value }
    }
}

/// A normalized two-axis range in curve coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CurveBounds {
    pub time_start: f32,
    pub time_end: f32,
    pub value_min: f32,
    pub value_max: f32,
}

impl CurveBounds {
    pub fn new(time_start: f32, time_end: f32, value_min: f32, value_max: f32) -> Self {
        let time_start = finite_or_zero(time_start);
        let time_end = finite_or(time_end, time_start);
        let value_min = finite_or_zero(value_min);
        let value_max = finite_or(value_max, value_min);
        Self {
            time_start: time_start.min(time_end),
            time_end: time_start.max(time_end),
            value_min: value_min.min(value_max),
            value_max: value_min.max(value_max),
        }
    }

    pub fn time_duration(self) -> f32 {
        self.time_end - self.time_start
    }

    pub fn value_span(self) -> f32 {
        self.value_max - self.value_min
    }

    pub fn contains(self, point: CurvePoint) -> bool {
        self.time_start <= point.time
            && point.time <= self.time_end
            && self.value_min <= point.value
            && point.value <= self.value_max
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurveInterpolation {
    Step,
    Linear,
    Hermite,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurveKey {
    pub id: String,
    pub point: CurvePoint,
    pub in_tangent: Option<f32>,
    pub out_tangent: Option<f32>,
}

impl CurveKey {
    pub fn new(id: impl Into<String>, point: CurvePoint) -> Self {
        Self {
            id: id.into(),
            point,
            in_tangent: None,
            out_tangent: None,
        }
    }

    pub fn with_tangents(mut self, in_tangent: Option<f32>, out_tangent: Option<f32>) -> Self {
        self.in_tangent = in_tangent.filter(|value| value.is_finite());
        self.out_tangent = out_tangent.filter(|value| value.is_finite());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurveView<CurveId> {
    pub id: CurveId,
    pub display_name: String,
    pub interpolation: CurveInterpolation,
    pub keys: Vec<CurveKey>,
}

impl<CurveId> CurveView<CurveId> {
    pub fn keys_in_bounds(&self, bounds: CurveBounds) -> Vec<&CurveKey> {
        self.keys
            .iter()
            .filter(|key| bounds.contains(key.point))
            .collect()
    }
}

/// Identifies the independently selectable parts of one authored curve key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CurveElementKind {
    Key,
    InTangent,
    OutTangent,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CurveElementRef<CurveId> {
    pub curve_id: CurveId,
    pub key_id: String,
    pub kind: CurveElementKind,
}

impl<CurveId> CurveElementRef<CurveId> {
    pub fn new(curve_id: CurveId, key_id: impl Into<String>, kind: CurveElementKind) -> Self {
        Self {
            curve_id,
            key_id: key_id.into(),
            kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurveSelection<CurveId> {
    elements: BTreeSet<CurveElementRef<CurveId>>,
}

impl<CurveId> Default for CurveSelection<CurveId> {
    fn default() -> Self {
        Self {
            elements: BTreeSet::new(),
        }
    }
}

impl<CurveId> CurveSelection<CurveId>
where
    CurveId: Ord,
{
    pub fn elements(&self) -> &BTreeSet<CurveElementRef<CurveId>> {
        &self.elements
    }

    pub fn contains(&self, element: &CurveElementRef<CurveId>) -> bool {
        self.elements.contains(element)
    }

    pub fn replace<I>(&mut self, elements: I) -> bool
    where
        I: IntoIterator<Item = CurveElementRef<CurveId>>,
    {
        let next = elements.into_iter().collect::<BTreeSet<_>>();
        if self.elements == next {
            return false;
        }
        self.elements = next;
        true
    }
}

/// Domain-owned curve mutation protocol. The foundation owns neither animation channels nor
/// field-editor values; toolkits return their own reversible delta through this interface.
pub trait CurveModel: Send {
    type CurveId: Clone + Eq + Ord;
    type Delta: Clone;
    type Error;

    fn curves(&self) -> Vec<CurveView<Self::CurveId>>;
    fn apply(&mut self, delta: Self::Delta) -> Result<Self::Delta, Self::Error>;
}

fn finite_or_zero(value: f32) -> f32 {
    finite_or(value, 0.0)
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    value.is_finite().then_some(value).unwrap_or(fallback)
}
