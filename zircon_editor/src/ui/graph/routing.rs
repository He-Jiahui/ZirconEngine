use super::GraphPoint;

const ROUTE_CLEARANCE: f32 = 48.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphRouteStyle {
    Orthogonal,
    Bezier,
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
