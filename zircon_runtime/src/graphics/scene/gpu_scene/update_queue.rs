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

/// 汇集 CPU 阴影数组的待上传槽位，并按元素跨度合并写入范围。
/// prepare 只借出计划；真正清空脏区须等 GpuScenePreparedUpload 被帧所有者接受。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuSceneUpdateQueue {
    dirty_primitives: Vec<GpuSceneDirtyRange>,
    dirty_instance_spans: Vec<GpuSceneDirtyRange>,
    dirty_lights: Vec<GpuSceneDirtyRange>,
    primitive_upload_ranges: Vec<GpuSceneUploadRange>,
    instance_upload_ranges: Vec<GpuSceneUploadRange>,
    light_upload_ranges: Vec<GpuSceneUploadRange>,
    primitive_upload_ranges_stride: Option<u64>,
    instance_upload_ranges_stride: Option<u64>,
    light_upload_ranges_stride: Option<u64>,
}

impl GpuSceneUpdateQueue {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn mark_primitive(&mut self, index: u32) {
        self.dirty_primitives
            .push(GpuSceneDirtyRange::new(index, 1));
        self.primitive_upload_ranges_stride = None;
    }

    pub(crate) fn mark_instances(&mut self, start: u32, len: u32) {
        if len == 0 {
            return;
        }
        self.dirty_instance_spans
            .push(GpuSceneDirtyRange::new(start, len));
        self.instance_upload_ranges_stride = None;
    }

    pub(crate) fn mark_light(&mut self, index: u32) {
        self.dirty_lights.push(GpuSceneDirtyRange::new(index, 1));
        self.light_upload_ranges_stride = None;
    }

    pub(crate) fn drain_primitive_upload_ranges(&mut self, stride: u64) -> &[GpuSceneUploadRange] {
        self.prepare_primitive_upload_ranges(stride);
        self.dirty_primitives.clear();
        self.primitive_upload_ranges_stride = None;
        &self.primitive_upload_ranges
    }

    pub(crate) fn prepare_primitive_upload_ranges(
        &mut self,
        stride: u64,
    ) -> &[GpuSceneUploadRange] {
        if self.primitive_upload_ranges_stride != Some(stride) {
            prepare_merged_upload_ranges(
                &mut self.dirty_primitives,
                &mut self.primitive_upload_ranges,
                stride,
            );
            self.primitive_upload_ranges_stride = Some(stride);
        }
        &self.primitive_upload_ranges
    }

    pub(crate) fn drain_instance_upload_ranges(&mut self, stride: u64) -> &[GpuSceneUploadRange] {
        self.prepare_instance_upload_ranges(stride);
        self.dirty_instance_spans.clear();
        self.instance_upload_ranges_stride = None;
        &self.instance_upload_ranges
    }

    pub(crate) fn prepare_instance_upload_ranges(&mut self, stride: u64) -> &[GpuSceneUploadRange] {
        if self.instance_upload_ranges_stride != Some(stride) {
            prepare_merged_upload_ranges(
                &mut self.dirty_instance_spans,
                &mut self.instance_upload_ranges,
                stride,
            );
            self.instance_upload_ranges_stride = Some(stride);
        }
        &self.instance_upload_ranges
    }

    pub(crate) fn drain_light_upload_ranges(&mut self, stride: u64) -> &[GpuSceneUploadRange] {
        self.prepare_light_upload_ranges(stride);
        self.dirty_lights.clear();
        self.light_upload_ranges_stride = None;
        &self.light_upload_ranges
    }

    pub(crate) fn prepare_light_upload_ranges(&mut self, stride: u64) -> &[GpuSceneUploadRange] {
        if self.light_upload_ranges_stride != Some(stride) {
            prepare_merged_upload_ranges(
                &mut self.dirty_lights,
                &mut self.light_upload_ranges,
                stride,
            );
            self.light_upload_ranges_stride = Some(stride);
        }
        &self.light_upload_ranges
    }

    pub(crate) fn discard_primitive_updates(&mut self) {
        self.dirty_primitives.clear();
        self.primitive_upload_ranges_stride = None;
    }

    pub(crate) fn discard_instance_updates(&mut self) {
        self.dirty_instance_spans.clear();
        self.instance_upload_ranges_stride = None;
    }

    pub(crate) fn discard_light_updates(&mut self) {
        self.dirty_lights.clear();
        self.light_upload_ranges_stride = None;
    }

    pub(crate) fn dirty_entry_count(&self) -> usize {
        self.dirty_primitives.len() + self.dirty_instance_spans.len() + self.dirty_lights.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.dirty_primitives.is_empty()
            && self.dirty_instance_spans.is_empty()
            && self.dirty_lights.is_empty()
    }

    #[cfg(test)]
    fn instance_upload_range_scratch_capacity(&self) -> usize {
        self.instance_upload_ranges.capacity()
    }

    #[cfg(test)]
    fn instance_upload_ranges_are_prepared(&self) -> bool {
        self.instance_upload_ranges_stride.is_some()
    }
}

fn prepare_merged_upload_ranges<'a>(
    ranges: &mut Vec<GpuSceneDirtyRange>,
    upload_ranges: &'a mut Vec<GpuSceneUploadRange>,
    stride: u64,
) -> &'a [GpuSceneUploadRange] {
    if ranges.is_empty() {
        upload_ranges.clear();
        return upload_ranges;
    }

    ranges.sort_unstable_by_key(|range| range.start);
    merge_sorted_ranges_in_place(ranges);
    upload_ranges.clear();
    upload_ranges.reserve(ranges.len().saturating_sub(upload_ranges.capacity()));
    upload_ranges.extend(
        ranges
            .iter()
            .copied()
            .map(|range| GpuSceneUploadRange::from_dirty_range(range, stride)),
    );
    upload_ranges
}

fn merge_sorted_ranges_in_place(ranges: &mut Vec<GpuSceneDirtyRange>) {
    let mut merged_len = 0;
    for read_index in 0..ranges.len() {
        let range = ranges[read_index];
        if merged_len != 0 {
            let current = &mut ranges[merged_len - 1];
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
        ranges[merged_len] = range;
        merged_len += 1;
    }
    ranges.truncate(merged_len);
}

#[cfg(test)]
#[path = "tests/update_queue.rs"]
mod tests;
