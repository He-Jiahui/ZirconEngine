use crate::core::framework::render::RenderGraphPassProfileMetrics;

#[test]
fn pass_profile_metrics_are_available_from_the_framework_render_root() {
    let metrics = RenderGraphPassProfileMetrics::new(3, 5, 7);

    assert_eq!(metrics.draw_count, 3);
    assert_eq!(metrics.instance_count, 5);
    assert_eq!(metrics.state_change_count, 7);
}
