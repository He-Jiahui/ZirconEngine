use super::{TimelineLaneKind, TimelineTrackView};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimelineTrackRow<TrackId> {
    pub track_id: TrackId,
    pub display_name: String,
    pub lane_kind: TimelineLaneKind,
    pub key_count: usize,
    pub section_count: usize,
}

pub fn lane_kind_for_value(value_kind: &str) -> TimelineLaneKind {
    if value_kind.eq_ignore_ascii_case("float")
        || value_kind.eq_ignore_ascii_case("scalar")
        || value_kind.eq_ignore_ascii_case("number")
        || value_kind.eq_ignore_ascii_case("vector2")
        || value_kind.eq_ignore_ascii_case("vector3")
        || value_kind.eq_ignore_ascii_case("vector4")
    {
        TimelineLaneKind::Curve
    } else if value_kind.eq_ignore_ascii_case("bool") || value_kind.eq_ignore_ascii_case("boolean")
    {
        TimelineLaneKind::Boolean
    } else if value_kind.eq_ignore_ascii_case("event") || value_kind.eq_ignore_ascii_case("notify")
    {
        TimelineLaneKind::Event
    } else if value_kind.eq_ignore_ascii_case("section")
        || value_kind.eq_ignore_ascii_case("segment")
    {
        TimelineLaneKind::Section
    } else {
        TimelineLaneKind::Keyframe
    }
}

pub fn project_track_list<TrackId>(
    tracks: &[TimelineTrackView<TrackId>],
) -> Vec<TimelineTrackRow<TrackId>>
where
    TrackId: Clone,
{
    tracks
        .iter()
        .map(|track| TimelineTrackRow {
            track_id: track.id.clone(),
            display_name: track.display_name.clone(),
            lane_kind: lane_kind_for_value(&track.value_kind),
            key_count: track.keys.len(),
            section_count: track.sections.len(),
        })
        .collect()
}
