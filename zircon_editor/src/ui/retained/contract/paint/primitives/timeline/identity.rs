use super::super::super::super::data::TemplatePaneNodeData;

pub(super) enum TimelinePrimitiveKind {
    Dot,
    Connector,
    Separator,
}

pub(super) fn timeline_primitive_kind(
    node: &TemplatePaneNodeData,
) -> Option<TimelinePrimitiveKind> {
    let component_role = node.component_role.as_str();
    let role = node.role.as_str();
    if matches_timeline_role(component_role, role, &["timeline-dot", "TimelineDot"]) {
        Some(TimelinePrimitiveKind::Dot)
    } else if matches_timeline_role(
        component_role,
        role,
        &["timeline-connector", "TimelineConnector"],
    ) {
        Some(TimelinePrimitiveKind::Connector)
    } else if matches_timeline_role(
        component_role,
        role,
        &["timeline-separator", "TimelineSeparator"],
    ) {
        Some(TimelinePrimitiveKind::Separator)
    } else {
        None
    }
}

fn matches_timeline_role(component_role: &str, role: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|candidate| component_role == *candidate || role == *candidate)
}
