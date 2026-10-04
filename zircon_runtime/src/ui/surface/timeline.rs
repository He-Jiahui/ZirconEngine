use std::collections::VecDeque;

use zircon_runtime_interface::ui::surface::{
    UiDebugTimelineFrameHandle, UiDebugTimelineFrameSummary, UiDebugTimelineRetention,
    UiDebugTimelineSnapshot, UiSurfaceDebugOptions, UiSurfaceDebugSnapshot,
};

/// 调试快照的有界历史，供反射器选择过去的帧；不参与运行时帧调度或 surface 恢复。
/// 捕获新帧会选中最新记录，容量满后旧句柄失效，外部应处理选择失败。
#[derive(Clone, Debug)]
pub struct UiDebugTimelineStore {
    capacity: usize,
    next_handle: Option<u64>,
    dropped_frame_count: u64,
    selected_frame: Option<UiDebugTimelineFrameHandle>,
    frames: VecDeque<UiDebugTimelineEntry>,
}

#[derive(Clone, Debug)]
struct UiDebugTimelineEntry {
    summary: UiDebugTimelineFrameSummary,
    snapshot: UiSurfaceDebugSnapshot,
}

impl UiDebugTimelineStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            next_handle: Some(1),
            dropped_frame_count: 0,
            selected_frame: None,
            frames: VecDeque::new(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Captures a snapshot with the next unique timeline handle.
    ///
    /// # Panics
    ///
    /// Panics after every nonzero `u64` frame handle has been issued.
    pub fn capture_snapshot(
        &mut self,
        snapshot: UiSurfaceDebugSnapshot,
        options: UiSurfaceDebugOptions,
    ) -> UiDebugTimelineFrameHandle {
        let handle_value = self
            .next_handle
            .expect("UI debug timeline frame handle space exhausted");
        let handle = UiDebugTimelineFrameHandle(handle_value);
        self.next_handle = handle_value.checked_add(1);
        let summary = frame_summary(handle, &snapshot, options);

        self.frames
            .push_back(UiDebugTimelineEntry { summary, snapshot });
        while self.frames.len() > self.capacity {
            self.frames.pop_front();
            self.dropped_frame_count = self.dropped_frame_count.saturating_add(1);
        }
        self.selected_frame = Some(handle);
        handle
    }

    pub fn select_frame(&mut self, handle: UiDebugTimelineFrameHandle) -> bool {
        if !self.contains_handle(handle) {
            return false;
        }
        self.selected_frame = Some(handle);
        true
    }

    pub fn latest_handle(&self) -> Option<UiDebugTimelineFrameHandle> {
        self.frames.back().map(|entry| entry.summary.handle)
    }

    pub fn selected_snapshot(&self) -> Option<&UiSurfaceDebugSnapshot> {
        let selected = self.selected_frame.or_else(|| self.latest_handle())?;
        self.frames
            .iter()
            .find(|entry| entry.summary.handle == selected)
            .map(|entry| &entry.snapshot)
    }

    pub fn snapshot(&self) -> UiDebugTimelineSnapshot {
        let selected_frame = self
            .selected_frame
            .filter(|handle| self.contains_handle(*handle))
            .or_else(|| self.latest_handle());
        UiDebugTimelineSnapshot {
            selected_frame,
            summaries: self
                .frames
                .iter()
                .map(|entry| entry.summary.clone())
                .collect(),
            frames: self
                .frames
                .iter()
                .map(|entry| entry.snapshot.clone())
                .collect(),
            retention: UiDebugTimelineRetention {
                capacity: self.capacity,
                len: self.frames.len(),
                first_frame: self.frames.front().map(|entry| entry.summary.handle),
                latest_frame: self.latest_handle(),
                selected_frame,
                dropped_frame_count: self.dropped_frame_count,
            },
        }
    }

    // 捕获按顺序分发句柄，淘汰只发生在队首，因此保留范围可用两端判断，无需扫描快照。
    fn contains_handle(&self, handle: UiDebugTimelineFrameHandle) -> bool {
        let Some((first, last)) = self.frames.front().zip(self.frames.back()) else {
            return false;
        };
        first.summary.handle.0 <= handle.0 && handle.0 <= last.summary.handle.0
    }
}

fn frame_summary(
    handle: UiDebugTimelineFrameHandle,
    snapshot: &UiSurfaceDebugSnapshot,
    options: UiSurfaceDebugOptions,
) -> UiDebugTimelineFrameSummary {
    UiDebugTimelineFrameSummary {
        handle,
        frame_index: snapshot.capture.frame_index.unwrap_or(handle.0),
        captured_at_millis: snapshot.capture.captured_at_millis,
        source_target_id: snapshot.tree_id.0.clone(),
        source_label: snapshot
            .capture
            .surface_name
            .clone()
            .unwrap_or_else(|| snapshot.tree_id.0.clone()),
        schema_version: snapshot.capture.schema_version,
        node_count: snapshot.nodes.len(),
        render_command_count: snapshot.render.command_count,
        hit_grid_cell_count: snapshot.hit_test.cell_count,
        invalidation_dirty_count: snapshot.invalidation.dirty_node_count,
        has_damage_region: snapshot.damage.damage_region.is_some(),
        warning_count: snapshot.invalidation.warnings.len() + snapshot.damage.warnings.len(),
        selected_node: snapshot.capture.selected_node,
        capture_options: options,
    }
}

#[cfg(test)]
#[path = "timeline/tests/handle_range_tests.rs"]
mod handle_range_tests;
