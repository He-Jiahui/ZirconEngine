use super::PipelineVariantPinCounts;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPipelineVariantId;

fn variant(value: u32) -> MeshPipelineVariantId {
    MeshPipelineVariantId::new(value)
}

#[test]
fn repeated_cache_entries_keep_one_variant_pinned_until_the_last_unpin() {
    let mut pins = PipelineVariantPinCounts::default();
    let variant = variant(7);

    pins.pin(variant);
    pins.pin(variant);
    assert!(pins.is_pinned(variant));
    assert_eq!(pins.pinned_variant_count(), 1);

    pins.unpin(variant);
    assert!(pins.is_pinned(variant));
    pins.unpin(variant);
    assert!(!pins.is_pinned(variant));
    assert_eq!(pins.pinned_variant_count(), 0);
}

#[test]
fn replacing_with_the_same_variant_does_not_change_counts() {
    let mut pins = PipelineVariantPinCounts::default();
    let variant = variant(11);

    pins.pin(variant);
    pins.replace(variant, variant);

    assert!(pins.is_pinned(variant));
    assert_eq!(pins.pinned_variant_count(), 1);
}

#[test]
fn replacing_with_a_different_variant_moves_the_pin() {
    let mut pins = PipelineVariantPinCounts::default();
    let previous = variant(13);
    let replacement = variant(17);

    pins.pin(previous);
    pins.replace(previous, replacement);

    assert!(!pins.is_pinned(previous));
    assert!(pins.is_pinned(replacement));
    assert_eq!(pins.pinned_variant_count(), 1);
}

#[test]
#[should_panic(expected = "pipeline variant pin count must exist before unpin")]
fn unpin_rejects_an_unbalanced_cache_release() {
    PipelineVariantPinCounts::default().unpin(variant(19));
}

#[test]
fn cached_command_owner_updates_pins_without_a_second_entry_scan() {
    let source = include_str!("../cached_mesh_draw_commands.rs");

    assert!(source.contains("pipeline_variant_pins: PipelineVariantPinCounts"));
    assert!(source.contains("pipeline_variant_pins.pin(variant_id)"));
    assert!(source.contains(".replace(previous.payload.pipeline_variant_id, variant_id)"));
    assert!(source.contains("pipeline_variant_pins.unpin(entry.payload.pipeline_variant_id)"));
    assert!(source.contains("pipeline_variant_pins.clear()"));
    assert!(!source.contains("collect::<HashSet<MeshPipelineVariantId>>"));
}
