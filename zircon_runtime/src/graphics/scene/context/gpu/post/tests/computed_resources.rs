#[test]
fn exposure_graph_passes_consume_frame_prepared_params_without_queue_access() {
    let source = include_str!("../computed_resources.rs");
    let exposure = source
        .split("pub(in crate::graphics::scene::scene_renderer) fn record_exposure_histogram_to_resource")
        .nth(1)
        .and_then(|source| source.split("pub(in crate::graphics::scene::scene_renderer) fn record_clustered_lighting_to_resources").next())
        .expect("exposure graph-pass section");

    assert!(!exposure.contains("self.queue"));
    assert!(!exposure.contains("append_pre_submit_buffer_uploads"));
    assert!(exposure.contains("execute_exposure_histogram("));
    assert!(exposure.contains("execute_exposure_resolve("));
}

#[test]
fn clustered_lighting_appends_feature_uploads_to_the_light_grid_pass() {
    let source = include_str!("../computed_resources.rs");
    let clustered = source
        .split("fn record_clustered_lighting_to_resources")
        .nth(1)
        .expect("clustered-lighting graph-pass section");
    let execute = clustered
        .find("execute_clustered_lighting(")
        .expect("clustered-lighting execution");
    let append = clustered[execute..]
        .find("self.append_pre_submit_buffer_uploads(")
        .map(|offset| execute + offset)
        .expect("clustered-lighting upload append");

    assert!(execute < append);
}
