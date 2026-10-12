#[test]
fn startup_scene_submission_reuses_the_document_route() {
    let source = include_str!("../scene_picker_actions.rs");

    assert!(source.contains("fn open_startup_scene"));
    assert!(source.contains("self.runtime.begin_scene_picker()"));
    assert!(source.contains("SceneOpenRequest::new(scene_uri)"));
    assert!(source.contains(".submit_scene_open_request(ticket, request)"));
}
