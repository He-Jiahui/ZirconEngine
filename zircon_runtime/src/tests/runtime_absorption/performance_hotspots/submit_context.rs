#[path = "submit_context/camera_loop_sharing.rs"]
mod camera_loop_sharing;
#[path = "submit_context/feedback_sidebands.rs"]
mod feedback_sidebands;
#[path = "submit_context/source_extract_payloads.rs"]
mod source_extract_payloads;
#[path = "submit_context/sources.rs"]
mod sources;
#[path = "submit_context/split_layout.rs"]
mod split_layout;
#[path = "submit_context/status_docs.rs"]
mod status_docs;

#[test]
fn runtime_07_submit_context_shares_large_extract_payloads() {
    let sources = sources::SubmitContextSources::load();

    source_extract_payloads::assert_source_extract_payloads_are_shared(&sources);
    camera_loop_sharing::assert_camera_loop_uses_shared_sources(&sources);
    feedback_sidebands::assert_feedback_sidebands_move_owned_payloads(&sources);
    status_docs::assert_submit_context_status_docs(&sources);
}
