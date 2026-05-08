use serde::{Deserialize, Serialize};

/// Flat counter bag for one UI pipeline stage or a whole frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPipelineStageCounters {
    pub input_event_count: u64,
    pub pointer_move_count: u64,
    pub focus_change_count: u64,
    pub content_measure_count: u64,
    pub template_reload_count: u64,
    pub layout_node_count: u64,
    pub full_layout_count: u64,
    pub incremental_layout_count: u64,
    pub stack_node_count: u64,
    pub hit_grid_rebuild_count: u64,
    pub render_extract_command_count: u64,
    pub render_command_reuse_count: u64,
    pub render_command_rebuild_count: u64,
    pub batch_count: u64,
    pub paint_submit_count: u64,
    pub diagnostic_record_count: u64,
}

impl UiPipelineStageCounters {
    pub fn add_assign(&mut self, other: Self) {
        self.input_event_count += other.input_event_count;
        self.pointer_move_count += other.pointer_move_count;
        self.focus_change_count += other.focus_change_count;
        self.content_measure_count += other.content_measure_count;
        self.template_reload_count += other.template_reload_count;
        self.layout_node_count += other.layout_node_count;
        self.full_layout_count += other.full_layout_count;
        self.incremental_layout_count += other.incremental_layout_count;
        self.stack_node_count += other.stack_node_count;
        self.hit_grid_rebuild_count += other.hit_grid_rebuild_count;
        self.render_extract_command_count += other.render_extract_command_count;
        self.render_command_reuse_count += other.render_command_reuse_count;
        self.render_command_rebuild_count += other.render_command_rebuild_count;
        self.batch_count += other.batch_count;
        self.paint_submit_count += other.paint_submit_count;
        self.diagnostic_record_count += other.diagnostic_record_count;
    }
}
