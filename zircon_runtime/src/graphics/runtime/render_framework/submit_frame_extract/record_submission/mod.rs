mod record;
mod record_capture;
mod record_history;
mod record_present;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use record::{
    record_submission, update_hybrid_gi_runtime, update_virtual_geometry_runtime,
};
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use record_history::record_history;
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use record_present::record_present_submission;

#[cfg(test)]
mod tests {
    #[test]
    fn submission_record_hot_path_reuses_pipeline_and_centralizes_vg_execution_stats() {
        let capture_source = include_str!("record_capture.rs");
        let present_source = include_str!("record_present.rs");
        let record_source = include_str!("record.rs");
        let non_viewport_source = include_str!("../submit/record_camera_history.rs");

        assert!(capture_source.contains("compiled_pipeline_shared()"));
        assert!(capture_source.contains("if !frame.capture_admitted"));
        assert!(capture_source.contains("register_async_capture"));
        assert!(!capture_source.contains("capture_graph_dump"));
        assert!(present_source.contains("compiled_pipeline_shared()"));
        assert!(!capture_source.contains(concat!("compiled_pipeline()", ".clone()")));
        assert!(!present_source.contains(concat!("compiled_pipeline()", ".clone()")));
        assert!(!record_source.contains("BTreeSet"));
        assert!(!record_source.contains(concat!("virtual_geometry_indirect_", "segment_count")));
        assert!(!non_viewport_source.contains(concat!(
            "virtual_geometry_feedback,",
            "\n            ",
            "0,"
        )));
    }
}
