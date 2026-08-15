use super::*;

#[test]
fn render_cache_reuses_each_command_emitted_by_one_node() {
    let commands = vec![
        quad(7, UiFrame::new(4.0, 8.0, 12.0, 16.0)),
        quad(7, UiFrame::new(24.0, 8.0, 12.0, 16.0)),
    ];
    let mut cache = UiSurfaceRenderCache::default();

    let first = cache.update(extract(commands.clone()), false);
    assert_eq!(first.stats.rebuilt_command_count, 2);
    assert_eq!(first.stats.reused_command_count, 0);
    assert_eq!(first.stats.damage_rect_count, 2);

    let stable = cache.update(extract(commands.clone()), false);
    assert_eq!(stable.stats.rebuilt_command_count, 0);
    assert_eq!(stable.stats.reused_command_count, 2);
    assert_eq!(stable.stats.damage_rect_count, 0);
    assert_eq!(stable.extract.list.commands, commands);

    let serialized = serde_json::to_string(&cache).expect("cache should serialize as JSON");
    let mut restored =
        serde_json::from_str::<UiSurfaceRenderCache>(&serialized).expect("cache should restore");
    let restored_stable = restored.update(extract(commands.clone()), false);
    assert_eq!(restored_stable.stats.rebuilt_command_count, 0);
    assert_eq!(restored_stable.stats.reused_command_count, 2);

    let removed_second_command = cache.update(extract(vec![commands[0].clone()]), false);
    assert_eq!(removed_second_command.stats.rebuilt_command_count, 0);
    assert_eq!(removed_second_command.stats.reused_command_count, 1);
    assert_eq!(removed_second_command.stats.damage_rect_count, 1);
}

#[test]
fn render_cache_deserializes_legacy_single_command_entries() {
    let command = quad(9, UiFrame::new(4.0, 8.0, 12.0, 16.0));
    let legacy_entries = BTreeMap::from([(
        UiNodeId::new(9),
        UiCachedRenderCommand {
            command: command.clone(),
        },
    )]);
    let legacy_json = serde_json::json!({ "entries": legacy_entries });
    let mut cache = serde_json::from_value::<UiSurfaceRenderCache>(legacy_json)
        .expect("legacy cache should deserialize");

    let update = cache.update(extract(vec![command]), false);
    assert_eq!(update.stats.rebuilt_command_count, 0);
    assert_eq!(update.stats.reused_command_count, 1);
}
