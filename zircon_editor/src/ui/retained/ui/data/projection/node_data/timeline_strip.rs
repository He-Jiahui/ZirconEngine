use crate::ui::retained_host as host_contract;

use super::super::timeline_strip::ProjectedTimelineStrip;

pub(super) fn assign_timeline_strip_fields(
    node: &mut host_contract::TemplatePaneNodeData,
    timeline_strip: ProjectedTimelineStrip,
) {
    node.timeline_strip = timeline_strip.data;
}
