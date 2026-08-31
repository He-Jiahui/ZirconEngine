use crate::ui::surface::{UiInvalidationReason, UiSurface, UiSurfaceRebuildReport};
use zircon_runtime_interface::{ProfileSnapshot, ui::event_ui::UiNodeId};

use super::MEASURED_FRAMES;
use super::support::assert_counter_frame_count;

pub(super) fn mutate(surface: &mut UiSurface, frame_index: usize) {
    let node_id = UiNodeId::new(2);
    let text = if frame_index % 2 == 0 {
        "L0000"
    } else {
        "L0001"
    };
    {
        let node = surface
            .tree
            .node_mut(node_id)
            .expect("localized baseline label should exist");
        let metadata = node
            .template_metadata
            .as_mut()
            .expect("localized baseline label should retain text metadata");
        metadata
            .attributes
            .insert("text".to_string(), toml::Value::String(text.to_string()));
    }
    surface
        .invalidate_node(node_id, UiInvalidationReason::Text)
        .expect("localized baseline label invalidation should succeed");
}

pub(super) fn assert_patch(rebuild: UiSurfaceRebuildReport, label_count: usize) {
    assert!(rebuild.dirty_flags.text);
    assert_eq!(rebuild.dirty_node_count, 1);
    assert!(rebuild.layout_recomputed);
    assert!(rebuild.arranged_rebuilt);
    assert!(rebuild.render_rebuilt);
    assert_eq!(rebuild.layout_visited_node_count, 1);
    assert_eq!(rebuild.arranged_outer_node_visit_count, 1);
    assert!(rebuild.hit_grid_outer_node_visit_count <= 1);
    assert_eq!(rebuild.render_outer_node_visit_count, 1);
    assert!(rebuild.render_command_rebuilt_count <= 1);
    assert!(rebuild.render_command_reused_count <= label_count);
    assert_eq!(rebuild.text_shape_cache_miss_count, 0);
}

pub(super) fn record_profile(rebuild: UiSurfaceRebuildReport) {
    crate::profile_counter!(
        "runtime",
        "ui_text.localized_dirty.layout_visited_nodes",
        rebuild.layout_visited_node_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.localized_dirty.layout_skipped_nodes",
        rebuild.layout_skipped_node_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.localized_dirty.arranged_outer_visited_nodes",
        rebuild.arranged_outer_node_visit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.localized_dirty.hit_grid_outer_visited_nodes",
        rebuild.hit_grid_outer_node_visit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.localized_dirty.render_outer_visited_nodes",
        rebuild.render_outer_node_visit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.localized_dirty.render_commands_rebuilt",
        rebuild.render_command_rebuilt_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.localized_dirty.render_commands_reused",
        rebuild.render_command_reused_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.localized_dirty.layout_micros",
        rebuild.layout_elapsed_micros
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.localized_dirty.arranged_micros",
        rebuild.arranged_elapsed_micros
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.localized_dirty.hit_grid_micros",
        rebuild.hit_grid_elapsed_micros
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.localized_dirty.render_micros",
        rebuild.render_elapsed_micros
    );
}

pub(super) fn assert_complete_capture(snapshot: &ProfileSnapshot) {
    for counter in [
        "ui_text.localized_dirty.layout_visited_nodes",
        "ui_text.localized_dirty.layout_skipped_nodes",
        "ui_text.localized_dirty.arranged_outer_visited_nodes",
        "ui_text.localized_dirty.hit_grid_outer_visited_nodes",
        "ui_text.localized_dirty.render_outer_visited_nodes",
        "ui_text.localized_dirty.render_commands_rebuilt",
        "ui_text.localized_dirty.render_commands_reused",
        "ui_text.localized_dirty.layout_micros",
        "ui_text.localized_dirty.arranged_micros",
        "ui_text.localized_dirty.hit_grid_micros",
        "ui_text.localized_dirty.render_micros",
    ] {
        assert_counter_frame_count(snapshot, counter);
    }
    for counter in [
        "ui_text.localized_dirty.layout_visited_nodes",
        "ui_text.localized_dirty.arranged_outer_visited_nodes",
        "ui_text.localized_dirty.hit_grid_outer_visited_nodes",
        "ui_text.localized_dirty.render_outer_visited_nodes",
        "ui_text.localized_dirty.render_commands_rebuilt",
    ] {
        assert_counter_at_most(snapshot, counter, 1.0);
    }
}

fn assert_counter_at_most(snapshot: &ProfileSnapshot, name: &str, maximum: f64) {
    let samples = snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .collect::<Vec<_>>();
    assert_eq!(samples.len(), MEASURED_FRAMES);
    assert!(
        samples.iter().all(|counter| counter.value <= maximum),
        "localized text-dirty baseline requires `{name}` <= {maximum}"
    );
}
