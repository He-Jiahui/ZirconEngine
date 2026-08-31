use super::*;

#[test]
fn typed_world_required_resource_accessors_use_direct_missing_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api.rs"),
    );
    let resource = source
        .split("pub fn resource<T>")
        .nth(1)
        .and_then(|text| text.split("pub fn get_resource<T>").next())
        .expect("read required resource accessor body")
        .replace("\r\n", "\n");
    let resource_mut = source
        .split("pub fn resource_mut<T>")
        .nth(1)
        .and_then(|text| text.split("pub fn get_resource_mut<T>").next())
        .expect("read required mutable resource accessor body")
        .replace("\r\n", "\n");

    assert!(
        resource.contains("let Some(resource) = self.get_resource::<T>() else")
            && resource.contains("requested missing scene resource {}")
            && resource.contains("std::any::type_name::<T>()")
            && resource.contains("};\n\n        resource")
            && !resource.contains(".unwrap_or_else(")
            && resource_mut.contains("let Some(resource) = self.get_resource_mut::<T>() else")
            && resource_mut.contains("requested missing scene resource {}")
            && resource_mut.contains("std::any::type_name::<T>()")
            && resource_mut.contains("};\n\n        resource")
            && !resource_mut.contains(".unwrap_or_else("),
        "typed required resource accessors must use direct missing-resource branches instead of unwrap_or_else closures"
    );
}

#[test]
fn world_set_joint_self_connection_uses_direct_option_branch() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("component_access.rs"),
    );
    let set_joint = source
        .split("pub fn set_joint")
        .nth(1)
        .and_then(|text| text.split("pub fn set_point_light").next())
        .expect("read set_joint body");

    assert!(
        set_joint.contains("let joint_connects_to_self = match &joint")
            && set_joint.contains("Some(joint) => joint.connected_entity == Some(entity)")
            && set_joint.contains("None => false")
            && set_joint.contains("if joint_connects_to_self")
            && !set_joint.contains(".and_then(|joint| joint.connected_entity)"),
        "World::set_joint must use a direct Option branch for self-connection validation"
    );
}

#[test]
fn ecs_typed_api_compile_owners_keep_internal_types_in_their_lowest_visible_domain() {
    let scene_root = manifest_dir().join("src").join("scene").join("world");
    let query = read_source(&scene_root.join("query.rs"));
    let bundle_transaction =
        read_source(&scene_root.join("typed_api").join("bundle_transaction.rs"));
    let deferred_bundle_staging = read_source(
        &scene_root
            .join("typed_api")
            .join("bundle_transaction")
            .join("deferred_bundle_staging.rs"),
    );
    let diagnostics = read_source(&scene_root.join("compiled_binding").join("diagnostics.rs"));
    let compiled_binding = read_source(&scene_root.join("compiled_binding").join("mod.rs"));

    let query_ecs_import = query
        .split("use crate::scene::ecs::{")
        .nth(1)
        .and_then(|text| text.split("};").next())
        .expect("read query ECS import");
    let bundle_ecs_import = bundle_transaction
        .split("use crate::scene::ecs::{")
        .nth(1)
        .and_then(|text| text.split("};").next())
        .expect("read bundle transaction ECS import");

    assert!(query_ecs_import.contains("ChangeTick"));
    assert!(!bundle_ecs_import.contains("Bundle,"));
    assert!(deferred_bundle_staging.contains("Bundle, BundleStaging"));
    assert!(
        diagnostics.contains("pub(in super::super) struct CompiledScenePropertyAccessDiagnostics")
    );
    assert!(
        compiled_binding
            .contains("pub(super) use diagnostics::CompiledScenePropertyAccessDiagnostics;")
    );
    assert!(!diagnostics.contains("pub(crate) struct CompiledScenePropertyAccessDiagnostics"));
}

#[test]
fn ecs_typed_api_bundle_transaction_keeps_staging_in_its_leaf_owner() {
    let transaction_root = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("bundle_transaction.rs"),
    );
    let staging = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("bundle_transaction")
            .join("staging.rs"),
    );
    let deferred_staging = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("bundle_transaction")
            .join("deferred_bundle_staging.rs"),
    );

    assert!(transaction_root.contains("mod staging;"));
    assert!(staging.contains("fn stage_default_node_record_components"));
    assert!(staging.contains("fn stage_deferred_remove"));
    assert!(deferred_staging.contains("fn new_deferred_existing"));
    assert!(deferred_staging.contains("fn stage_deferred_bundle"));
    assert!(!transaction_root.contains("fn stage_default_node_record_components"));
    assert!(!transaction_root.contains("fn new_deferred_existing"));
    assert!(!transaction_root.contains("fn stage_deferred_bundle"));
}
