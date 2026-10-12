//! Shared time-authoring foundation for animation and other editor toolkits.

#[path = "keyframe_lane.rs"]
mod keyframe_lane;
#[path = "model.rs"]
mod model;
#[path = "ruler.rs"]
mod ruler;
#[path = "section_lane.rs"]
mod section_lane;
#[path = "track_list.rs"]
mod track_list;

pub use keyframe_lane::{keyframes_in_range, TimelineKeyframeLane};
pub use model::{
    TimelineElementRef, TimelineKey, TimelineLaneKind, TimelineModel, TimelineRange,
    TimelineSection, TimelineSelection, TimelineTrackView,
};
pub use ruler::{build_timeline_ruler_ticks, TimelineRulerTick, TimelineSnapSettings};
pub use section_lane::{
    section_overlap_verdict, TimelineSectionOverlapPolicy, TimelineSectionOverlapVerdict,
};
pub use track_list::{lane_kind_for_value, project_track_list, TimelineTrackRow};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
