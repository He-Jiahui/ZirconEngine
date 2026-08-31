use std::collections::BTreeSet;

use crate::core::editor_authoring_extension::{TimelineEditorDescriptor, TimelineTrackDescriptor};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineRange {
    pub start: f32,
    pub end: f32,
}

impl TimelineRange {
    pub fn new(start: f32, end: f32) -> Self {
        let start = start.is_finite().then_some(start).unwrap_or_default();
        let end = end.is_finite().then_some(end).unwrap_or(start);
        if start <= end {
            Self { start, end }
        } else {
            Self {
                start: end,
                end: start,
            }
        }
    }

    pub fn duration(self) -> f32 {
        self.end - self.start
    }

    pub fn clamp(self, time: f32) -> f32 {
        time.is_finite()
            .then_some(time.clamp(self.start, self.end))
            .unwrap_or(self.start)
    }

    pub fn contains(self, time: f32) -> bool {
        self.start <= time && time <= self.end
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineKey {
    pub id: String,
    pub time: f32,
    pub label: String,
}

impl TimelineKey {
    pub fn new(id: impl Into<String>, time: f32, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            time: time.is_finite().then_some(time).unwrap_or_default(),
            label: label.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineSection {
    pub id: String,
    pub label: String,
    pub range: TimelineRange,
}

impl TimelineSection {
    pub fn new(id: impl Into<String>, label: impl Into<String>, range: TimelineRange) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            range,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimelineLaneKind {
    Keyframe,
    Curve,
    Boolean,
    Event,
    Section,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineTrackView<TrackId> {
    pub id: TrackId,
    pub display_name: String,
    pub value_kind: String,
    pub keys: Vec<TimelineKey>,
    pub sections: Vec<TimelineSection>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimelineElementRef<TrackId> {
    Key {
        track_id: TrackId,
        key_id: String,
    },
    Section {
        track_id: TrackId,
        section_id: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TimelineSelection<TrackId> {
    elements: BTreeSet<TimelineElementRef<TrackId>>,
}

impl<TrackId> TimelineSelection<TrackId>
where
    TrackId: Ord,
{
    pub fn elements(&self) -> &BTreeSet<TimelineElementRef<TrackId>> {
        &self.elements
    }

    pub fn contains(&self, element: &TimelineElementRef<TrackId>) -> bool {
        self.elements.contains(element)
    }

    pub fn replace<I>(&mut self, elements: I) -> bool
    where
        I: IntoIterator<Item = TimelineElementRef<TrackId>>,
    {
        let next = elements.into_iter().collect::<BTreeSet<_>>();
        if self.elements == next {
            return false;
        }
        self.elements = next;
        true
    }
}

/// A domain-owned timeline mutation protocol. The foundation never stores an animation sequence,
/// montage, or section asset; toolkits return their own inverse delta through this interface.
pub trait TimelineModel: Send {
    type TrackId: Clone + Eq + Ord;
    type Delta: Clone;
    type Error;

    fn descriptor(&self) -> &TimelineEditorDescriptor;
    fn track_catalog(&self) -> &[TimelineTrackDescriptor];
    fn range(&self) -> TimelineRange;
    fn playhead(&self) -> f32;
    fn tracks(&self) -> Vec<TimelineTrackView<Self::TrackId>>;
    fn apply(&mut self, delta: Self::Delta) -> Result<Self::Delta, Self::Error>;
}
