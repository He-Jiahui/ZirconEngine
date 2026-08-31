//! Shared time-authoring foundation for animation and other editor toolkits.

mod keyframe_lane;
mod model;
mod ruler;
mod section_lane;
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
mod tests;
