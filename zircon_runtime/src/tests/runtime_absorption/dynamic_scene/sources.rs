pub(super) const PATCH_SOURCE: &str = include_str!("../../../scene/dynamic_scene/patch.rs");
pub(super) const DYNAMIC_SCENE_MOD_SOURCE: &str =
    include_str!("../../../scene/dynamic_scene/mod.rs");
pub(super) const SCENE_MOD_SOURCE: &str = include_str!("../../../scene/dynamic_scene/scene/mod.rs");
pub(super) const SPAWN_SOURCE: &str = include_str!("../../../scene/dynamic_scene/scene/spawn.rs");
pub(super) const BEHAVIOR_SOURCE: &str = concat!(
    include_str!("../../../scene/tests/dynamic_scene.rs"),
    include_str!("../../../scene/tests/dynamic_scene/scene_patch_document.rs")
);
pub(super) const CAPTURE_BEHAVIOR_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/capture.rs");
pub(super) const PERSISTENCE_BEHAVIOR_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/persistence.rs");
pub(super) const RETENTION_BEHAVIOR_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/retention.rs");
pub(super) const MUTATION_BEHAVIOR_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/mutation.rs");
pub(super) const SELECTION_BEHAVIOR_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/selection.rs");
pub(super) const MERGE_BEHAVIOR_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/merge.rs");
pub(super) const LOAD_BEHAVIOR_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/load.rs");
pub(super) const QUERIES_BEHAVIOR_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/queries.rs");
pub(super) const PATH_MANAGEMENT_BEHAVIOR_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/path_management.rs");
pub(super) const PATH_MANAGEMENT_ARCHIVE_MERGE_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/path_management/archive_merge.rs");
pub(super) const PATH_MANAGEMENT_MUTATION_PREVIEWS_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/path_management/mutation_previews.rs");
pub(super) const PATH_MANAGEMENT_SINGLE_SLOT_IMPORT_SOURCE: &str = include_str!(
    "../../../scene/tests/dynamic_scene_session/path_management/single_slot_import.rs"
);
pub(super) const PATH_MANAGEMENT_SINGLE_SLOT_SAVE_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/path_management/single_slot_save.rs");
pub(super) const PATH_MANAGEMENT_SLOT_COPY_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/path_management/slot_copy.rs");
pub(super) const PATH_MANAGEMENT_SLOT_MUTATIONS_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_session/path_management/slot_mutations.rs");
pub(super) const ASSET_RELOAD_BEHAVIOR_SOURCE: &str =
    include_str!("../../../scene/tests/dynamic_scene_asset_reload.rs");
pub(super) const RUNTIME_05_PLAN: &str = concat!(
    include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    ),
    include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/05/2026-07-09-scene-editor-boundary-closeout-output-records.md"
    )
);
pub(super) const RUNTIME_INDEX: &str = concat!(
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
    include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    )
);
pub(super) const DYNAMIC_SCENE_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/scene/dynamic_scene.md");
