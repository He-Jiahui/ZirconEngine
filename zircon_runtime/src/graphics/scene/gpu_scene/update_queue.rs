pub(crate) const GPU_SCENE_DIRTY_RANGE_MERGE_GAP: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuSceneDirtyRange {
    pub(crate) start: u32,
    pub(crate) len: u32,
}

impl GpuSceneDirtyRange {
    pub(crate) fn new(start: u32, len: u32) -> Self {
        debug_assert!(len > 0);
        Self { start, len }
    }

    pub(crate) fn end_exclusive(self) -> u32 {
        self.start
            .checked_add(self.len)
            .expect("gpu scene dirty range end overflowed u32")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuSceneUploadRange {
    pub(crate) start: u32,
    pub(crate) len: u32,
    pub(crate) byte_offset: u64,
    pub(crate) byte_len: u64,
}

impl GpuSceneUploadRange {
    fn from_dirty_range(range: GpuSceneDirtyRange, stride: u64) -> Self {
        Self {
            start: range.start,
            len: range.len,
            byte_offset: u64::from(range.start) * stride,
            byte_len: u64::from(range.len) * stride,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuSceneUpdateQueue {
    dirty_primitives: Vec<GpuSceneDirtyRange>,
    dirty_instance_spans: Vec<GpuSceneDirtyRange>,
}

impl GpuSceneUpdateQueue {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn mark_primitive(&mut self, index: u32) {
        self.dirty_primitives
            .push(GpuSceneDirtyRange::new(index, 1));
    }

    pub(crate) fn mark_instances(&mut self, start: u32, len: u32) {
        if len == 0 {
            return;
        }
        self.dirty_instance_spans
            .push(GpuSceneDirtyRange::new(start, len));
    }

    pub(crate) fn drain_primitive_upload_ranges(
        &mut self,
        stride: u64,
    ) -> Vec<GpuSceneUploadRange> {
        drain_merged_upload_ranges(&mut self.dirty_primitives, stride)
    }

    pub(crate) fn drain_instance_upload_ranges(&mut self, stride: u64) -> Vec<GpuSceneUploadRange> {
        drain_merged_upload_ranges(&mut self.dirty_instance_spans, stride)
    }

    pub(crate) fn dirty_entry_count(&self) -> usize {
        self.dirty_primitives.len() + self.dirty_instance_spans.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.dirty_primitives.is_empty() && self.dirty_instance_spans.is_empty()
    }
}

fn drain_merged_upload_ranges(
    ranges: &mut Vec<GpuSceneDirtyRange>,
    stride: u64,
) -> Vec<GpuSceneUploadRange> {
    if ranges.is_empty() {
        return Vec::new();
    }

    ranges.sort_by_key(|range| range.start);
    let merged = merge_sorted_ranges(ranges.drain(..));
    merged
        .into_iter()
        .map(|range| GpuSceneUploadRange::from_dirty_range(range, stride))
        .collect()
}

fn merge_sorted_ranges(
    ranges: impl IntoIterator<Item = GpuSceneDirtyRange>,
) -> Vec<GpuSceneDirtyRange> {
    let mut merged: Vec<GpuSceneDirtyRange> = Vec::new();
    for range in ranges {
        if range.len == 0 {
            continue;
        }
        if let Some(current) = merged.last_mut() {
            let current_end = current.end_exclusive();
            let merge_limit = current_end
                .checked_add(GPU_SCENE_DIRTY_RANGE_MERGE_GAP)
                .expect("gpu scene dirty range merge limit overflowed u32");
            if range.start <= merge_limit {
                let range_end = range.end_exclusive();
                current.len = range_end.max(current_end) - current.start;
                continue;
            }
        }
        merged.push(range);
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_gpu_scene_update_queue_merges_adjacent_dirty_ranges() {
        let mut queue = GpuSceneUpdateQueue::new();
        queue.mark_instances(23, 2);
        queue.mark_instances(3, 1);
        queue.mark_instances(12, 2);
        queue.mark_instances(4, 1);
        queue.mark_instances(40, 1);

        let ranges = queue.drain_instance_upload_ranges(144);

        assert_eq!(
            ranges,
            vec![
                GpuSceneUploadRange {
                    start: 3,
                    len: 11,
                    byte_offset: 432,
                    byte_len: 1584,
                },
                GpuSceneUploadRange {
                    start: 23,
                    len: 2,
                    byte_offset: 3312,
                    byte_len: 288,
                },
                GpuSceneUploadRange {
                    start: 40,
                    len: 1,
                    byte_offset: 5760,
                    byte_len: 144,
                },
            ]
        );
        assert!(queue.is_empty());

        queue.mark_primitive(7);
        queue.mark_primitive(7);
        assert_eq!(
            queue.drain_primitive_upload_ranges(80),
            vec![GpuSceneUploadRange {
                start: 7,
                len: 1,
                byte_offset: 560,
                byte_len: 80,
            }]
        );
    }
}
