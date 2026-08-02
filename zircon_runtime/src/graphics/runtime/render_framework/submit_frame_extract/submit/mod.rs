mod build_runtime_frame;
mod build_virtual_geometry_debug_snapshot;
mod camera_loop;
mod collect_runtime_feedback;
mod present_frame_extract;
mod record_camera_history;
mod release_previous_history;
mod resolve_history_handle;
mod submit;
mod submit_runtime_frame;
mod update_particle_previous_state;
mod update_temporal_camera_history;

pub(in crate::graphics::runtime::render_framework) use present_frame_extract::{
    present_frame_extract, present_frame_extract_with_ui,
};
pub(in crate::graphics::runtime::render_framework) use submit::submit_frame_extract;
pub(in crate::graphics::runtime::render_framework) use submit::submit_frame_extract_with_ui;
pub(in crate::graphics::runtime::render_framework) use submit_runtime_frame::submit_runtime_frame;

#[cfg(test)]
mod tests {
    #[test]
    fn retained_viewport_submit_paths_use_async_capture_without_sync_rgba_readback() {
        let extract_submit = include_str!("submit.rs");
        let runtime_submit = include_str!("submit_runtime_frame.rs");
        let present_submit = include_str!("present_frame_extract.rs");

        for source in [extract_submit, runtime_submit, present_submit] {
            assert!(source.contains("render_frame_with_pipeline_async_capture_task_pool"));
            assert!(!source.contains("render_frame_with_pipeline_task_pool("));
            assert!(!source.contains("finish_viewport_frame"));
        }
    }
}
