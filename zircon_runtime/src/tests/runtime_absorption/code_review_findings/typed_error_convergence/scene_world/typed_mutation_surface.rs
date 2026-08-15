#[test]
fn review_f5_world_spawn_bundle_surface_uses_scene_error() {
    let scene_mod = include_str!("../../../../../scene/mod.rs");
    let world_mod = include_str!("../../../../../scene/world/mod.rs");
    let world_error = include_str!("../../../../../scene/world/error.rs");
    let typed_api = include_str!("../../../../../scene/world/typed_api.rs");
    let bundle_entry = include_str!("../../../../../scene/world/typed_api/bundle_entry.rs");
    let identity = include_str!("../../../../../scene/world/identity.rs");
    let fixed_components = include_str!("../../../../../scene/world/typed_api/fixed_components.rs");
    let bundle = include_str!("../../../../../scene/ecs/bundle.rs");
    let command_facade = include_str!("../../../../../scene/ecs/commands/commands/facade.rs");
    let entity_commands =
        include_str!("../../../../../scene/ecs/commands/commands/entity_commands.rs");
    let review_findings = concat!(
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let runtime_08_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let ecs_doc = include_str!("../../../../../../../docs/zircon_runtime/scene/ecs.md");

    for anchor in [
        "pub type SceneResult<T> = std::result::Result<T, SceneError>;",
        "pub enum SceneError",
        "MissingEntity {",
        "EntityRegistry(#[from] EntityRegistryError)",
        "Storage(",
        "#[from] StorageError",
        "impl From<String> for SceneError",
    ] {
        assert!(
            world_error.contains(anchor),
            "F5 world error owner should expose typed error anchor `{anchor}`"
        );
    }
    assert!(
        world_mod.contains("pub use error::{SceneError, SceneResult};")
            && scene_mod.contains("SceneError")
            && scene_mod.contains("SceneResult")
            && scene_mod.contains("World"),
        "SceneError/SceneResult should be exported through the world and scene facades"
    );

    for forbidden in [
        "pub fn spawn<B>(&mut self, bundle: B) -> Result<EntityId, String>",
        "pub(crate) fn spawn_at<B>(&mut self, entity: EntityId, bundle: B) -> Result<EntityId, String>",
        "pub(crate) fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B) -> Result<(), String>",
        "pub fn insert<T>(&mut self, entity: EntityId, component: T) -> Result<Option<T>, String>",
        "pub fn remove<T>(&mut self, entity: EntityId) -> Result<Option<T>, String>",
        "fn insert_into(self, world: &mut World, entity: EntityId) -> Result<(), String>",
        "Result<InternalEntity, String>",
        "Result<(), String>",
    ] {
        assert!(
            !typed_api.contains(forbidden)
                && !bundle_entry.contains(forbidden)
                && !identity.contains(forbidden)
                && !fixed_components.contains(forbidden)
                && !bundle.contains(forbidden),
            "F5 should not keep public typed ECS mutation surface as String error `{forbidden}`"
        );
    }
    let insert_body = typed_api
        .split("pub fn insert<T>")
        .nth(1)
        .and_then(|source| source.split("pub fn get<T>").next())
        .expect("read World::insert body");
    let remove_body = typed_api
        .split("pub fn remove<T>")
        .nth(1)
        .and_then(|source| source.split("pub fn resource_id").next())
        .expect("read World::remove body");
    let insert_bundle_body = bundle_entry
        .split("pub(crate) fn insert_bundle<B>")
        .nth(1)
        .and_then(|source| source.split("pub(crate) fn begin_bundle_insertion").next())
        .expect("read World::insert_bundle body");
    assert!(
        !insert_body.contains("error.to_string()") && !remove_body.contains("error.to_string()"),
        "World::insert/remove should preserve storage errors through SceneError instead of stringifying them"
    );
    assert!(
        !typed_api.contains("error.to_string()")
            && !bundle_entry.contains("error.to_string()")
            && !identity.contains("error.to_string()"),
        "World typed API identity and presence helpers should preserve typed source errors instead of stringifying them"
    );
    assert!(
        insert_bundle_body.contains("let mut transaction = self.begin_bundle_insertion(entity)?;")
            && insert_bundle_body.contains("bundle.stage_into(&mut transaction)?;")
            && insert_bundle_body.contains("transaction.finish()"),
        "World::insert_bundle must retain exclusive ownership of bundle transaction publication"
    );

    for required in [
        "pub fn spawn<B>(&mut self, bundle: B) -> SceneResult<EntityId>",
        "pub(crate) fn spawn_at<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<EntityId>",
        "pub(crate) fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<()>",
        "pub fn insert<T>(&mut self, entity: EntityId, component: T) -> SceneResult<Option<T>>",
        "pub fn remove<T>(&mut self, entity: EntityId) -> SceneResult<Option<T>>",
        "pub(super) fn register_stable_entity",
        "SceneResult<InternalEntity>",
        "pub(super) fn insert_dynamic_component_presence",
        "pub(super) fn remove_dynamic_component_presence",
        "SceneError::missing_entity(\"insert component on\", entity)",
        "Err(error) => return Err(error.into())",
        ".spawn(entity, EntityLocation::new(ArchetypeId::EMPTY, row))?",
        "fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>",
        "fn stage<T>(&mut self, component: T) -> SceneResult<()>",
    ] {
        assert!(
            typed_api.contains(required)
                || bundle_entry.contains(required)
                || identity.contains(required)
                || fixed_components.contains(required)
                || bundle.contains(required),
            "F5 typed ECS mutation surface should contain `{required}`"
        );
    }
    for command_anchor in [
        "DeferredCommandOperation::Spawn",
        "DeferredCommandOperation::Insert",
        "DeferredCommandOperation::InsertBundle",
        "DeferredCommandOperation::Remove",
        "error.to_string()",
    ] {
        assert!(
            command_facade.contains(command_anchor) || entity_commands.contains(command_anchor),
            "deferred command reporting should stringify typed SceneError only at the report boundary: `{command_anchor}`"
        );
    }

    for doc_anchor in [
        "F5 world typed mutation errors",
        "world_typed_mutation_errors_coremin_check_passed_partial",
        "review_f5_world_spawn_bundle_surface_uses_scene_error",
        "SceneError::MissingEntity",
        "SceneResult",
        "F5 typed API residual typed errors",
        "runtime_15_typed_api_residual_typed_errors_static_passed_cargo_deferred",
        "f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_08_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || ecs_doc.contains(doc_anchor),
            "F5 docs should record `{doc_anchor}`"
        );
    }
    let f5_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F5 |"))
        .expect("F5 review findings top row");
    assert!(
        f5_row.contains("ECS 公共 mutation surface")
            && f5_row.ends_with("| Runtime 08 + Runtime 15 |"),
        "F5 overview row should keep only the finding and delegated owners"
    );
    assert!(
        review_findings
            .contains("f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred"),
        "F5 numbered output should record typed-error review closed status"
    );
}
