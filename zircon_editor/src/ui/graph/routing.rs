use super::GraphPoint;

const ROUTE_CLEARANCE: f32 = 48.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphRouteStyle {
    Orthogonal,
    Bezier,
}

/// Spatial direction out of a pin, independent of its input/output semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphRouteDirection {
    Left,
    Right,
    Up,
    Down,
}

impl GraphRouteDirection {
    fn unit(self) -> GraphPoint {
        match self {
            Self::Left => GraphPoint::new(-1.0, 0.0),
            Self::Right => GraphPoint::new(1.0, 0.0),
            Self::Up => GraphPoint::new(0.0, -1.0),
            Self::Down => GraphPoint::new(0.0, 1.0),
        }
    }

    fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// Target anchor and unit travel direction in graph-local coordinates. The consumer owns the
/// arrow brush, size, line separation and transform; this projection applies no zoom or DPI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphConnectionArrow {
    pub position: GraphPoint,
    pub direction: GraphPoint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphConnectionRoute {
    style: GraphRouteStyle,
    points: Vec<GraphPoint>,
    bezier_controls: Option<(GraphPoint, GraphPoint)>,
}

impl GraphConnectionRoute {
    pub fn style(&self) -> GraphRouteStyle {
        self.style
    }

    pub fn points(&self) -> &[GraphPoint] {
        &self.points
    }

    pub fn bezier_controls(&self) -> Option<(GraphPoint, GraphPoint)> {
        self.bezier_controls
    }

    /// Unit travel tangents at the source and target, shared by paint and arrow projection.
    pub fn endpoint_tangents(&self) -> Option<(GraphPoint, GraphPoint)> {
        let source = *self.points.first()?;
        let target = *self.points.last()?;
        let (after_source, before_target) = match self.bezier_controls {
            Some(controls) => controls,
            None => (
                *self.points.iter().find(|point| **point != source)?,
                *self.points.iter().rev().find(|point| **point != target)?,
            ),
        };
        Some((
            unit_delta(source, after_source)?,
            unit_delta(before_target, target)?,
        ))
    }

    pub fn target_arrow(&self) -> Option<GraphConnectionArrow> {
        let (_, direction) = self.endpoint_tangents()?;
        Some(GraphConnectionArrow {
            position: *self.points.last()?,
            direction,
        })
    }
}

pub fn route_connection(
    source: GraphPoint,
    target: GraphPoint,
    style: GraphRouteStyle,
) -> GraphConnectionRoute {
    match style {
        GraphRouteStyle::Orthogonal => orthogonal_route(source, target),
        GraphRouteStyle::Bezier => bezier_route(source, target),
    }
}

/// Opt-in spatial pin directions. Both directions point out of their node, so the travel tangent
/// entering the target is the opposite of `target_direction`. Legacy right/left routes are exact.
/// Non-finite or unrepresentable geometry is rejected before reaching a renderer.
pub fn route_connection_with_directions(
    source: GraphPoint,
    target: GraphPoint,
    source_direction: GraphRouteDirection,
    target_direction: GraphRouteDirection,
    style: GraphRouteStyle,
) -> Option<GraphConnectionRoute> {
    if !point_is_finite(source) || !point_is_finite(target) {
        return None;
    }
    let route = if source_direction == GraphRouteDirection::Right
        && target_direction == GraphRouteDirection::Left
    {
        route_connection(source, target, style)
    } else {
        let start_unit = source_direction.unit();
        let end_unit = target_direction.unit();
        let clearance = match style {
            GraphRouteStyle::Bezier => {
                let dx = (target.x - source.x).abs();
                let dy = (target.y - source.y).abs();
                let source_span = if source_direction.is_horizontal() {
                    dx
                } else {
                    dy
                };
                let target_span = if target_direction.is_horizontal() {
                    dx
                } else {
                    dy
                };
                (source_span.max(target_span) * 0.5).max(ROUTE_CLEARANCE)
            }
            GraphRouteStyle::Orthogonal => ROUTE_CLEARANCE,
        };
        let start_lead = offset(source, start_unit, clearance);
        let end_lead = offset(target, end_unit, clearance);
        if style == GraphRouteStyle::Bezier {
            GraphConnectionRoute {
                style,
                points: vec![source, target],
                bezier_controls: Some((start_lead, end_lead)),
            }
        } else {
            let mut points = Vec::with_capacity(6);
            push_distinct(&mut points, source);
            push_distinct(&mut points, start_lead);
            match (
                source_direction.is_horizontal(),
                target_direction.is_horizontal(),
            ) {
                (true, true) => {
                    let x = start_lead.x * 0.5 + end_lead.x * 0.5;
                    push_distinct(&mut points, GraphPoint::new(x, start_lead.y));
                    push_distinct(&mut points, GraphPoint::new(x, end_lead.y));
                }
                (false, false) => {
                    let y = start_lead.y * 0.5 + end_lead.y * 0.5;
                    push_distinct(&mut points, GraphPoint::new(start_lead.x, y));
                    push_distinct(&mut points, GraphPoint::new(end_lead.x, y));
                }
                (true, false) => {
                    push_distinct(&mut points, GraphPoint::new(end_lead.x, start_lead.y));
                }
                (false, true) => {
                    push_distinct(&mut points, GraphPoint::new(start_lead.x, end_lead.y));
                }
            }
            push_distinct(&mut points, end_lead);
            push_distinct(&mut points, target);
            GraphConnectionRoute {
                style,
                points,
                bezier_controls: None,
            }
        }
    };
    if !route.points.iter().copied().all(point_is_finite)
        || route
            .bezier_controls
            .is_some_and(|(first, second)| !point_is_finite(first) || !point_is_finite(second))
    {
        return None;
    }
    let (start, end) = route.endpoint_tangents()?;
    let target_unit = target_direction.unit();
    (start == source_direction.unit() && end == GraphPoint::new(-target_unit.x, -target_unit.y))
        .then_some(route)
}

fn orthogonal_route(source: GraphPoint, target: GraphPoint) -> GraphConnectionRoute {
    let start_lead = GraphPoint::new(source.x + ROUTE_CLEARANCE, source.y);
    let end_lead = GraphPoint::new(target.x - ROUTE_CLEARANCE, target.y);
    let bend_x = if start_lead.x <= end_lead.x {
        (start_lead.x + end_lead.x) * 0.5
    } else {
        start_lead.x.max(end_lead.x) + ROUTE_CLEARANCE
    };
    let mut points = Vec::with_capacity(6);
    push_distinct(&mut points, source);
    push_distinct(&mut points, start_lead);
    push_distinct(&mut points, GraphPoint::new(bend_x, start_lead.y));
    push_distinct(&mut points, GraphPoint::new(bend_x, end_lead.y));
    push_distinct(&mut points, end_lead);
    push_distinct(&mut points, target);
    GraphConnectionRoute {
        style: GraphRouteStyle::Orthogonal,
        points,
        bezier_controls: None,
    }
}

fn bezier_route(source: GraphPoint, target: GraphPoint) -> GraphConnectionRoute {
    let control_offset = ((target.x - source.x).abs() * 0.5).max(ROUTE_CLEARANCE);
    GraphConnectionRoute {
        style: GraphRouteStyle::Bezier,
        points: vec![source, target],
        bezier_controls: Some((
            GraphPoint::new(source.x + control_offset, source.y),
            GraphPoint::new(target.x - control_offset, target.y),
        )),
    }
}

fn push_distinct(points: &mut Vec<GraphPoint>, point: GraphPoint) {
    if points.last().copied() != Some(point) {
        points.push(point);
    }
}

fn offset(point: GraphPoint, direction: GraphPoint, distance: f32) -> GraphPoint {
    GraphPoint::new(
        point.x + direction.x * distance,
        point.y + direction.y * distance,
    )
}

fn point_is_finite(point: GraphPoint) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

fn unit_delta(from: GraphPoint, to: GraphPoint) -> Option<GraphPoint> {
    let delta = GraphPoint::new(to.x - from.x, to.y - from.y);
    let length = delta.x.hypot(delta.y);
    (length.is_finite() && length > 0.0)
        .then(|| GraphPoint::new(delta.x / length, delta.y / length))
}
