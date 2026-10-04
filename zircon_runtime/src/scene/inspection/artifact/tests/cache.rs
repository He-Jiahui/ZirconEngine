use std::collections::BTreeSet;
use std::sync::RwLock;

use crate::scene::NodeKind;

use super::{WorldInspectionArtifactCache, WorldInspectionArtifactDiagnostics};
use crate::scene::World;

#[test]
fn clone_generation_guard_rebuilds_a_split_publication_snapshot() {
    let mut source = World::empty();
    let renamed = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let stale_artifact = source.inspection_artifact();
    source.rename_node(renamed, "Current name").unwrap();
    let expected_generation = source.world_generation();
    source.inspection_artifact();

    let split_publication_cache = WorldInspectionArtifactCache {
        artifact: RwLock::new(Some(stale_artifact)),
        fields: RwLock::new(None),
        dirty_field_entities: RwLock::new(BTreeSet::new()),
        diagnostics: RwLock::new(WorldInspectionArtifactDiagnostics::default()),
        hierarchy_full_rebuild_required: RwLock::new(false),
        dirty_hierarchy_names: RwLock::new(BTreeSet::new()),
    };
    let mut cloned = source.clone();
    cloned.inspection_artifact_cache =
        split_publication_cache.clone_for_world_generation(expected_generation);

    let artifact = cloned.inspection_artifact();
    assert_eq!(artifact.generation(), expected_generation);
    assert_eq!(
        artifact
            .hierarchy_row(renamed)
            .map(|row| row.display_name.as_str()),
        Some("Current name")
    );
}
