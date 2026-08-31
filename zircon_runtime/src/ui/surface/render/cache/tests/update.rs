use super::*;
use crate::ui::surface::UiSurface;

#[test]
fn render_cache_reuses_each_command_emitted_by_one_node() {
    let commands = vec![
        quad(7, UiFrame::new(4.0, 8.0, 12.0, 16.0)),
        quad(7, UiFrame::new(24.0, 8.0, 12.0, 16.0)),
    ];
    let mut cache = UiSurfaceRenderCache::default();
    let empty = extract(Vec::new());

    let first = cache.update(&empty, extract(commands.clone()), false);
    assert_eq!(first.stats.rebuilt_command_count, 2);
    assert_eq!(first.stats.reused_command_count, 0);
    assert_eq!(first.stats.damage_rect_count, 2);

    let stable = cache.update(&first.extract, extract(commands.clone()), false);
    assert_eq!(stable.stats.rebuilt_command_count, 0);
    assert_eq!(stable.stats.reused_command_count, 2);
    assert_eq!(stable.stats.damage_rect_count, 0);
    assert_eq!(stable.extract.list.commands, commands);

    let removed_second_command =
        cache.update(&stable.extract, extract(vec![commands[0].clone()]), false);
    assert_eq!(removed_second_command.stats.rebuilt_command_count, 0);
    assert_eq!(removed_second_command.stats.reused_command_count, 1);
    assert_eq!(removed_second_command.stats.damage_rect_count, 1);
}

#[test]
fn render_cache_update_retains_the_input_command_buffer_allocation() {
    let commands = vec![
        quad(3, UiFrame::new(2.0, 4.0, 8.0, 16.0)),
        quad(5, UiFrame::new(12.0, 4.0, 8.0, 16.0)),
    ];
    let extract = extract(commands);
    let command_buffer = extract.list.commands.as_ptr();
    let command_capacity = extract.list.commands.capacity();
    let mut cache = UiSurfaceRenderCache::default();
    let previous = extract(Vec::new());

    let update = cache.update(&previous, extract, false);

    assert_eq!(update.extract.list.commands.as_ptr(), command_buffer);
    assert_eq!(update.extract.list.commands.capacity(), command_capacity);
}

#[test]
fn render_cache_range_lookup_fails_closed_for_non_contiguous_node_commands() {
    let commands = vec![
        quad(3, UiFrame::new(2.0, 4.0, 8.0, 16.0)),
        quad(5, UiFrame::new(12.0, 4.0, 8.0, 16.0)),
        quad(3, UiFrame::new(22.0, 4.0, 8.0, 16.0)),
    ];
    let mut cache = UiSurfaceRenderCache::default();
    let previous = extract(Vec::new());

    let update = cache.update(&previous, extract(commands), false);

    assert!(
        cache
            .commands_for_node(&update.extract, UiNodeId::new(3))
            .is_none()
    );
    let (start, commands) = cache
        .commands_for_node(&update.extract, UiNodeId::new(5))
        .expect("the contiguous node range remains indexed");
    assert_eq!(start, 1);
    assert_eq!(commands.len(), 1);
}

#[test]
fn render_cache_reuses_commands_after_global_order_changes() {
    let first_order = vec![
        quad(3, UiFrame::new(2.0, 4.0, 8.0, 16.0)),
        quad(5, UiFrame::new(12.0, 4.0, 8.0, 16.0)),
    ];
    let next_order = vec![first_order[1].clone(), first_order[0].clone()];
    let mut cache = UiSurfaceRenderCache::default();
    let first = cache.update(&extract(Vec::new()), extract(first_order), false);

    let reordered = cache.update(&first.extract, extract(next_order.clone()), false);

    assert_eq!(reordered.stats.rebuilt_command_count, 0);
    assert_eq!(reordered.stats.reused_command_count, 2);
    assert_eq!(reordered.stats.damage_rect_count, 0);
    assert_eq!(reordered.extract.list.commands, next_order);
    assert_eq!(
        cache
            .commands_for_node(&reordered.extract, UiNodeId::new(5))
            .expect("the reordered command remains indexed")
            .0,
        0
    );
}

#[test]
fn render_cache_rebuilds_when_previous_extract_does_not_match_metadata() {
    let first_command = quad(3, UiFrame::new(2.0, 4.0, 8.0, 16.0));
    let next_command = quad(3, UiFrame::new(20.0, 4.0, 8.0, 16.0));
    let unrelated_previous = extract(vec![next_command.clone()]);
    let mut cache = UiSurfaceRenderCache::default();
    let _ = cache.update(&extract(Vec::new()), extract(vec![first_command]), false);

    let rebuilt = cache.update(
        &unrelated_previous,
        extract(vec![next_command.clone()]),
        false,
    );

    assert_eq!(rebuilt.stats.rebuilt_command_count, 1);
    assert_eq!(rebuilt.stats.reused_command_count, 0);
    assert_eq!(rebuilt.stats.damage_rect_count, 1);
    assert_eq!(rebuilt.extract.list.commands, vec![next_command]);
}

#[test]
fn render_cache_entry_is_compact_metadata_not_command_payload() {
    assert_eq!(
        std::mem::size_of::<UiCachedRenderCommandMetadata>(),
        std::mem::size_of::<usize>() + std::mem::size_of::<UiFrame>()
    );
}

#[test]
fn surface_serialization_omits_derived_render_cache() {
    let commands = vec![
        quad(7, UiFrame::new(4.0, 8.0, 12.0, 16.0)),
        quad(7, UiFrame::new(24.0, 8.0, 12.0, 16.0)),
    ];
    let mut surface = UiSurface::new(UiTreeId::new("ui.cache.surface-round-trip"));
    let update =
        surface
            .render_cache
            .update(&surface.render_extract, extract(commands.clone()), false);
    surface.render_extract = update.extract;

    let encoded = serde_json::to_value(&surface).expect("surface should serialize");
    assert!(encoded.get("render_cache").is_none());
    let mut restored =
        serde_json::from_value::<UiSurface>(encoded).expect("surface should deserialize");
    assert_eq!(restored.render_extract, surface.render_extract);
    assert_eq!(restored.render_cache, UiSurfaceRenderCache::default());

    let rebuilt =
        restored
            .render_cache
            .update(&restored.render_extract, extract(commands.clone()), false);
    assert_eq!(rebuilt.stats.rebuilt_command_count, 2);
    assert_eq!(rebuilt.stats.reused_command_count, 0);
    let stable = restored
        .render_cache
        .update(&rebuilt.extract, extract(commands), false);
    assert_eq!(stable.stats.rebuilt_command_count, 0);
    assert_eq!(stable.stats.reused_command_count, 2);
}

#[test]
fn surface_restore_damages_old_and_new_frames_when_commands_move_or_disappear() {
    let retained = quad(7, UiFrame::new(4.0, 8.0, 12.0, 16.0));
    let removed = quad(9, UiFrame::new(24.0, 8.0, 12.0, 16.0));
    let moved = quad(7, UiFrame::new(44.0, 8.0, 12.0, 16.0));
    let mut surface = UiSurface::new(UiTreeId::new("ui.cache.surface-restore-damage"));
    let update = surface.render_cache.update(
        &surface.render_extract,
        extract(vec![retained, removed]),
        false,
    );
    surface.render_extract = update.extract;

    let encoded = serde_json::to_value(&surface).expect("surface should serialize");
    let mut restored =
        serde_json::from_value::<UiSurface>(encoded).expect("surface should deserialize");
    let rebuilt =
        restored
            .render_cache
            .update(&restored.render_extract, extract(vec![moved]), false);

    assert_eq!(rebuilt.stats.rebuilt_command_count, 1);
    assert_eq!(rebuilt.stats.reused_command_count, 0);
    assert_eq!(rebuilt.stats.damage_rect_count, 3);
}
