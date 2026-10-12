use super::authoring_boundary::{assert_text_excludes_authoring_tokens, SOURCE_AUTHORING_TOKENS};

#[path = "component_structure/component_registry.rs"]
mod component_registry;
#[path = "component_structure/component_storage_dispatch.rs"]
mod component_storage_dispatch;
#[path = "component_structure/component_storage_indexing.rs"]
mod component_storage_indexing;
#[path = "component_structure/dynamic_scene_owner_tree.rs"]
mod dynamic_scene_owner_tree;
#[path = "component_structure/project_serialization.rs"]
mod project_serialization;
#[path = "component_structure/runtime_08_owner_tree.rs"]
mod runtime_08_owner_tree;
#[path = "component_structure/runtime_world_domains.rs"]
mod runtime_world_domains;
