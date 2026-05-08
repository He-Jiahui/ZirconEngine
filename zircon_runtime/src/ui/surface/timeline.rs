use std::collections::VecDeque;

use zircon_runtime_interface::ui::surface::{
    UiDebugTimelineFrameHandle, UiDebugTimelineFrameSummary, UiDebugTimelineRetention,
    UiDebugTimelineSnapshot, UiSurfaceDebugOptions, UiSurfaceDebugSnapshot,
};

#[derive(Clone, Debug)]
pub struct UiDebugTimelineStore {
    capacity: usize,
    next_handle: u64,
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
            next_handle: 1,
            dropped_frame_count: 0,
            selected_frame: None,
            frames: VecDeque::new(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn capture_snapshot(
        &mut self,
        snapshot: UiSurfaceDebugSnapshot,
        options: UiSurfaceDebugOptions,
    ) -> UiDebugTimelineFrameHandle {
        let handle = UiDebugTimelineFrameHandle(self.next_handle);
        self.next_handle = self.next_handle.saturating_add(1).max(1);
        let summary = frame_summary(handle, &snapshot, options);

        self.frames
            .push_back(UiDebugTimelineEntry { summary, snapshot });
        while self.frames.len() > self.capacity {
            self.frames.pop_front();
            self.dropped_frame_count = self.dropped_frame_count.saturating_add(1);
        }
        self.selected_frame = Some(handle);
        if !self.contains_handle(handle) {
            self.selected_frame = self.latest_handle();
        }
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

    fn contains_handle(&self, handle: UiDebugTimelineFrameHandle) -> bool {
        self.frames
            .iter()
            .any(|entry| entry.summary.handle == handle)
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
