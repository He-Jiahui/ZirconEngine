#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuSceneIdSpan {
    pub(crate) start: u32,
    pub(crate) len: u32,
}

impl GpuSceneIdSpan {
    pub(crate) fn new(start: u32, len: u32) -> Self {
        debug_assert!(len > 0);
        Self { start, len }
    }

    pub(crate) fn end_exclusive(self) -> u32 {
        self.start
            .checked_add(self.len)
            .expect("gpu scene id span end overflowed u32")
    }
}

/// 为 primitive 与 instance 分配稳定的 GPU 数组槽位；释放先进入待合并区。
/// 同一帧的后续注册不能复用旧槽位，以免已录制的绘制包把新对象解释成旧对象。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuSceneIdAllocator {
    // Kept sorted and coalesced after every frame-boundary merge.
    free_spans: Vec<GpuSceneIdSpan>,
    pending_free_spans: Vec<GpuSceneIdSpan>,
    free_span_merge_scratch: Vec<GpuSceneIdSpan>,
    pending_free_spans_needs_sort: bool,
    next: u32,
    live: u32,
    high_water: u32,
}

impl GpuSceneIdAllocator {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn allocate(&mut self) -> u32 {
        self.allocate_span(1).start
    }

    pub(crate) fn allocate_span(&mut self, len: u32) -> GpuSceneIdSpan {
        assert!(len > 0, "gpu scene span allocation length must be non-zero");

        if let Some((free_index, free_span)) = self
            .free_spans
            .iter()
            .copied()
            .enumerate()
            .find(|(_, span)| span.len >= len)
        {
            let allocated = GpuSceneIdSpan::new(free_span.start, len);
            if free_span.len == len {
                self.free_spans.remove(free_index);
            } else {
                self.free_spans[free_index] = GpuSceneIdSpan::new(
                    free_span
                        .start
                        .checked_add(len)
                        .expect("gpu scene free span start overflowed u32"),
                    free_span.len - len,
                );
            }
            self.live = self
                .live
                .checked_add(len)
                .expect("gpu scene live id count overflowed u32");
            return allocated;
        }

        let allocated = GpuSceneIdSpan::new(self.next, len);
        self.next = self
            .next
            .checked_add(len)
            .expect("gpu scene id allocator exhausted u32 ids");
        self.high_water = self.high_water.max(self.next);
        self.live = self
            .live
            .checked_add(len)
            .expect("gpu scene live id count overflowed u32");
        allocated
    }

    pub(crate) fn free(&mut self, id: u32) {
        self.free_span(id, 1);
    }

    /// Defers reuse until the caller reaches the frame boundary.
    ///
    /// GPUScene ids may be referenced by in-flight command buffers. Deferring
    /// the merge prevents a newly registered primitive from aliasing an id
    /// that the current frame can still read.
    pub(crate) fn free_span(&mut self, start: u32, len: u32) {
        if len == 0 {
            return;
        }
        debug_assert!(self.live >= len);
        self.live = self.live.saturating_sub(len);
        if self
            .pending_free_spans
            .last()
            .is_some_and(|last| last.start > start)
        {
            self.pending_free_spans_needs_sort = true;
        }
        self.pending_free_spans
            .push(GpuSceneIdSpan::new(start, len));
    }

    pub(crate) fn commit_pending_frees(&mut self) {
        if self.pending_free_spans.is_empty() {
            return;
        }

        if self.pending_free_spans_needs_sort {
            self.pending_free_spans
                .sort_unstable_by_key(|span| span.start);
        }
        let required_capacity = self
            .free_spans
            .len()
            .saturating_add(self.pending_free_spans.len());
        if self.free_span_merge_scratch.capacity() < required_capacity {
            self.free_span_merge_scratch
                .reserve(required_capacity - self.free_span_merge_scratch.capacity());
        }
        self.free_span_merge_scratch.clear();

        let mut free_index = 0;
        let mut pending_index = 0;
        while free_index < self.free_spans.len() || pending_index < self.pending_free_spans.len() {
            let next_span = if pending_index == self.pending_free_spans.len()
                || (free_index < self.free_spans.len()
                    && self.free_spans[free_index].start
                        <= self.pending_free_spans[pending_index].start)
            {
                let span = self.free_spans[free_index];
                free_index += 1;
                span
            } else {
                let span = self.pending_free_spans[pending_index];
                pending_index += 1;
                span
            };
            push_coalesced_span(&mut self.free_span_merge_scratch, next_span);
        }

        std::mem::swap(&mut self.free_spans, &mut self.free_span_merge_scratch);
        self.pending_free_spans.clear();
        self.pending_free_spans_needs_sort = false;
    }

    pub(crate) fn live(&self) -> u32 {
        self.live
    }

    pub(crate) fn high_water(&self) -> u32 {
        self.high_water
    }

    pub(crate) fn free_span_count(&self) -> usize {
        self.free_spans.len()
    }

    pub(crate) fn pending_free_span_count(&self) -> usize {
        self.pending_free_spans.len()
    }

    #[cfg(test)]
    pub(crate) fn free_spans(&self) -> &[GpuSceneIdSpan] {
        &self.free_spans
    }
}

fn push_coalesced_span(spans: &mut Vec<GpuSceneIdSpan>, span: GpuSceneIdSpan) {
    if span.len == 0 {
        return;
    }
    if let Some(last) = spans.last_mut() {
        let last_end = last.end_exclusive();
        if span.start <= last_end {
            let span_end = span.end_exclusive();
            last.len = span_end.max(last_end) - last.start;
            return;
        }
    }
    spans.push(span);
}

#[cfg(test)]
#[path = "tests/id_allocator.rs"]
mod tests;
