mod asset_reload_selection_status;
mod patch_preview_api;
mod patch_preview_behavior;
mod patch_preview_status_docs;
mod session_capture_persistence;
mod session_load_query_path;
mod session_retention_mutation_merge;

const PATCH_SOURCE: &str = include_str!("../../scene/dynamic_scene/patch.rs");
const DYNAMIC_SCENE_MOD_SOURCE: &str = include_str!("../../scene/dynamic_scene/mod.rs");
const SCENE_MOD_SOURCE: &str = include_str!("../../scene/dynamic_scene/scene/mod.rs");
const SPAWN_SOURCE: &str = include_str!("../../scene/dynamic_scene/scene/spawn.rs");
const BEHAVIOR_SOURCE: &str = include_str!("../../scene/tests/dynamic_scene.rs");
const CAPTURE_BEHAVIOR_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/capture.rs");
const PERSISTENCE_BEHAVIOR_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/persistence.rs");
const RETENTION_BEHAVIOR_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/retention.rs");
const MUTATION_BEHAVIOR_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/mutation.rs");
const SELECTION_BEHAVIOR_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/selection.rs");
const MERGE_BEHAVIOR_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/merge.rs");
const LOAD_BEHAVIOR_SOURCE: &str = include_str!("../../scene/tests/dynamic_scene_session/load.rs");
const QUERIES_BEHAVIOR_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/queries.rs");
const PATH_MANAGEMENT_BEHAVIOR_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/path_management.rs");
const PATH_MANAGEMENT_ARCHIVE_MERGE_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/path_management/archive_merge.rs");
const PATH_MANAGEMENT_MUTATION_PREVIEWS_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/path_management/mutation_previews.rs");
const PATH_MANAGEMENT_SINGLE_SLOT_IMPORT_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/path_management/single_slot_import.rs");
const PATH_MANAGEMENT_SINGLE_SLOT_SAVE_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/path_management/single_slot_save.rs");
const PATH_MANAGEMENT_SLOT_COPY_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/path_management/slot_copy.rs");
const PATH_MANAGEMENT_SLOT_MUTATIONS_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_session/path_management/slot_mutations.rs");
const ASSET_RELOAD_BEHAVIOR_SOURCE: &str =
    include_str!("../../scene/tests/dynamic_scene_asset_reload.rs");
const RUNTIME_05_PLAN: &str = include_str!(
    "../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
);
const RUNTIME_INDEX: &str = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
const DYNAMIC_SCENE_DOC: &str =
    include_str!("../../../../docs/zircon_runtime/scene/dynamic_scene.md");
