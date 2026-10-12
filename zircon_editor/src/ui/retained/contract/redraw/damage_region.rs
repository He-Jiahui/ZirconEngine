use super::super::data::FrameRect;
use super::super::frame_geometry::union_frame;

const DAMAGE_RECT_CAPACITY: usize = 3;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostDamageRegion {
    frames: [FrameRect; DAMAGE_RECT_CAPACITY],
    len: usize,
    bounding_frame: FrameRect,
    source_rect_count: usize,
    simplification_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HostDamageRegionMetrics {
    pub(crate) rect_count: usize,
    pub(crate) source_rect_count: usize,
    pub(crate) simplification_count: usize,
    pub(crate) represented_area: f64,
    pub(crate) bounding_area: f64,
    pub(crate) bounding_overdraw_area: f64,
}

impl HostDamageRegion {
    pub(super) fn from_frame(frame: FrameRect) -> Self {
        let mut frames = std::array::from_fn(|_| FrameRect::default());
        frames[0] = frame.clone();
        Self {
            frames,
            len: 1,
            bounding_frame: frame,
            source_rect_count: 1,
            simplification_count: 0,
        }
    }

    pub(super) fn merge(mut self, next: Self) -> Self {
        let bounding_frame = union_frame(&self.bounding_frame, &next.bounding_frame);
        let source_rect_count = self
            .source_rect_count
            .saturating_add(next.source_rect_count);
        let next_simplification_count = next.simplification_count;
        for frame in next.frames().iter().cloned() {
            self.insert_represented_frame(frame);
        }
        self.source_rect_count = source_rect_count;
        self.simplification_count = self
            .simplification_count
            .saturating_add(next_simplification_count);
        self.bounding_frame = bounding_frame;
        self
    }

    pub(super) fn bounding_frame(&self) -> &FrameRect {
        &self.bounding_frame
    }

    pub(super) fn metrics(&self) -> HostDamageRegionMetrics {
        let represented_area = represented_union_area(self.frames());
        let bounding_area = frame_area(&self.bounding_frame);
        HostDamageRegionMetrics {
            rect_count: self.len,
            source_rect_count: self.source_rect_count,
            simplification_count: self.simplification_count,
            represented_area,
            bounding_area,
            bounding_overdraw_area: (bounding_area - represented_area).max(0.0),
        }
    }

    fn frames(&self) -> &[FrameRect] {
        &self.frames[..self.len]
    }

    fn insert_represented_frame(&mut self, frame: FrameRect) {
        if self
            .frames()
            .iter()
            .any(|current| contains(current, &frame))
        {
            return;
        }

        let mut index = 0;
        while index < self.len {
            if contains(&frame, &self.frames[index]) {
                self.remove(index);
            } else {
                index += 1;
            }
        }

        if self.len < DAMAGE_RECT_CAPACITY {
            self.frames[self.len] = frame;
            self.len += 1;
        } else {
            let merge_index = self.least_added_area_index(&frame);
            self.frames[merge_index] = union_frame(&self.frames[merge_index], &frame);
            self.simplification_count = self.simplification_count.saturating_add(1);
            self.remove_frames_contained_by(merge_index);
        }
    }

    fn least_added_area_index(&self, frame: &FrameRect) -> usize {
        self.frames()
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| {
                added_union_area(left, frame).total_cmp(&added_union_area(right, frame))
            })
            .map(|(index, _)| index)
            .expect("a full damage region must contain a merge candidate")
    }

    fn remove_frames_contained_by(&mut self, container_index: usize) {
        let container = self.frames[container_index].clone();
        let mut index = 0;
        while index < self.len {
            if index != container_index && contains(&container, &self.frames[index]) {
                self.remove(index);
                if index < container_index {
                    return self.remove_frames_contained_by(container_index - 1);
                }
            } else {
                index += 1;
            }
        }
    }

    fn remove(&mut self, index: usize) {
        for next in index + 1..self.len {
            self.frames[next - 1] = self.frames[next].clone();
        }
        self.len -= 1;
        self.frames[self.len] = FrameRect::default();
    }
}

fn added_union_area(current: &FrameRect, next: &FrameRect) -> f64 {
    frame_area(&union_frame(current, next)) - frame_area(current)
}

fn contains(outer: &FrameRect, inner: &FrameRect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.right() >= inner.right()
        && outer.bottom() >= inner.bottom()
}

fn represented_union_area(frames: &[FrameRect]) -> f64 {
    let individual_area = frames.iter().map(frame_area).sum::<f64>();
    let pair_area = (0..frames.len())
        .flat_map(|left| (left + 1..frames.len()).map(move |right| (left, right)))
        .filter_map(|(left, right)| intersect(&frames[left], &frames[right]))
        .map(|frame| frame_area(&frame))
        .sum::<f64>();
    let triple_area = if frames.len() == DAMAGE_RECT_CAPACITY {
        intersect(&frames[0], &frames[1])
            .and_then(|frame| intersect(&frame, &frames[2]))
            .map_or(0.0, |frame| frame_area(&frame))
    } else {
        0.0
    };
    individual_area - pair_area + triple_area
}

fn intersect(left: &FrameRect, right: &FrameRect) -> Option<FrameRect> {
    let x = left.x.max(right.x);
    let y = left.y.max(right.y);
    let right_edge = left.right().min(right.right());
    let bottom_edge = left.bottom().min(right.bottom());
    (right_edge > x && bottom_edge > y).then_some(FrameRect {
        x,
        y,
        width: right_edge - x,
        height: bottom_edge - y,
    })
}

fn frame_area(frame: &FrameRect) -> f64 {
    f64::from(frame.width) * f64::from(frame.height)
}
