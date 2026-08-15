use super::*;

#[test]
fn render_cache_counts_a_shared_damage_frame_once() {
    let shared_frame = UiFrame::new(4.0, 8.0, 12.0, 16.0);
    let commands = (0..1_024)
        .map(|node_id| quad(node_id, shared_frame))
        .collect::<Vec<_>>();
    let mut cache = UiSurfaceRenderCache::default();

    let first = cache.update(extract(commands), false);

    assert_eq!(first.stats.rebuilt_command_count, 1_024);
    assert_eq!(first.stats.damage_rect_count, 1);
}
