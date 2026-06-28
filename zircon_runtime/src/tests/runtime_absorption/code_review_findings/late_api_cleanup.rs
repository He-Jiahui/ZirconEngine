#[test]
fn review_f11_shading_model_registry_has_no_dead_plugin_registration_surface() {
    let registry = include_str!("../../../graphics/material/shading_models/registry.rs");
    let core_contract = include_str!("../../../core/framework/render/material/shading_model.rs");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let render_index = include_str!("../../../../../docs/plans/zircon_runtime/render/index.md");
    let material_doc =
        include_str!("../../../../../docs/zircon_runtime/core/framework/render/material.md");

    assert!(
        !registry.contains("#[allow(dead_code)]"),
        "shading-model registry should not preserve disconnected production API behind dead_code"
    );
    for forbidden_dead_surface in [
        "pub(crate) fn supported_channels(",
        "pub(crate) fn len(",
        "pub(crate) fn register_plugin(",
        "PluginIdBelowReservedRange",
    ] {
        assert!(
            !registry.contains(forbidden_dead_surface)
                && !core_contract.contains(forbidden_dead_surface),
            "F11 should not reintroduce disconnected shading-model registry surface `{forbidden_dead_surface}`"
        );
    }

    for live_registry_anchor in [
        "fn resolve_token(&self, token: &str)",
        "self.resolve_token(&model.as_token())",
        "pub(crate) fn register_builtin(",
        "RequiredChannelsUnsupported",
    ] {
        assert!(
            registry.contains(live_registry_anchor),
            "shading-model registry should retain live built-in resolver anchor `{live_registry_anchor}`"
        );
    }

    for doc_anchor in [
        "F11 shading-model registry dead API removal",
        "render_shading_model_registry_dead_api_removed_coremin_passed",
        "review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
        "custom shading-model plugin registration remains a future Plan 08 surface",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || render_index.contains(doc_anchor)
                || material_doc.contains(doc_anchor),
            "F11 docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f15_editor_pane_data_conversion_top_row_uses_projection_owners() {
    let pane_mod = include_str!(
        "../../../../../zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs"
    );
    let template_node_projection = include_str!(
        "../../../../../zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_node_projection.rs"
    );
    let animation_projection = include_str!(
        "../../../../../zircon_editor/src/ui/retained_host/ui/pane_data_conversion/animation_projection.rs"
    );
    let apply_pane_conversion = include_str!(
        "../../../../../zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs"
    );
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let runtime_15 = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let module_doc =
        include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
    let editor_workbench_doc =
        include_str!("../../../../../docs/editor-and-tooling/editor-workbench-shell.md");
    let f15_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F15 |"))
        .expect("engine-code-review findings should keep a top-level F15 row");

    for required in [
        "editor `pane_data_conversion` 投影函数样板复制已由 child projection owners 收束",
        "pane_data_conversion/mod.rs",
        "template_node_projection.rs",
        "animation_projection.rs",
        "apply_presentation/pane_conversion.rs",
        "runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred",
        "runtime_15_editor_retained_host_pane_data_conversion_uses_child_projection_owners",
    ] {
        assert!(
            f15_row.contains(required),
            "F15 top review row should record current projection-owner state `{required}`"
        );
    }
    for stale_finding in [
        "editor/.../pane_data_conversion/mod.rs:74",
        "to_host_contract_pane` ~228 行",
        "animation_template_projection` ~326 行",
        "抽 `project_nodes<T>()` 泛型 helper + 按 header/body/fields 拆",
    ] {
        assert!(
            !f15_row.contains(stale_finding),
            "F15 top review row should not keep stale unresolved finding text `{stale_finding}`"
        );
    }

    for required in [
        "mod animation_projection;",
        "mod template_node_projection;",
        "pub(crate) use self::animation_projection::",
        "use self::template_runtime_projection::",
    ] {
        assert!(
            pane_mod.contains(required),
            "pane_data_conversion root should retain child projection-owner wiring `{required}`"
        );
    }
    for moved_owner in [
        "fn animation_template_projection(",
        "fn to_host_contract_animation_editor_pane(",
        "fn to_host_contract_pane(",
        "pub(super) fn project_nodes<",
    ] {
        assert!(
            !pane_mod.contains(moved_owner),
            "pane_data_conversion root should not regain moved owner `{moved_owner}`"
        );
    }
    for required in [
        "pub(super) fn project_nodes<T, F>(",
        "pub(super) fn project_node_vec<T, F>(",
    ] {
        assert!(
            template_node_projection.contains(required),
            "template node projection should own shared node helper `{required}`"
        );
    }
    assert!(
        animation_projection.contains("fn animation_template_projection(")
            && animation_projection.contains("PanePayload::AnimationSequenceV1")
            && animation_projection.contains("PanePayload::AnimationGraphV1"),
        "animation projection owner should retain sequence and graph payload conversion"
    );
    assert!(
        apply_pane_conversion.contains("pub(super) fn to_host_contract_pane(")
            && apply_pane_conversion.contains("has_animation_payload")
            && apply_pane_conversion.contains("to_host_contract_animation_editor_pane("),
        "apply presentation owner should retain pane routing"
    );

    for doc_anchor in [
        "Runtime 15 M3 editor retained-host pane data conversion projection owner guard",
        "runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred",
        "review_f15_editor_pane_data_conversion_top_row_uses_projection_owners",
        "runtime_15_editor_retained_host_pane_data_conversion_uses_child_projection_owners",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || runtime_15.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || module_doc.contains(doc_anchor)
                || editor_workbench_doc.contains(doc_anchor),
            "F15 docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f17_entity_path_option_lookup_uses_get_verb() {
    let path_resolution = include_str!("../../../scene/world/property_access/path_resolution.rs");
    let runtime_apply = include_str!("../../../animation/sequence/apply.rs");
    let runtime_target = include_str!("../../../animation/sequence/target.rs");
    let plugin_apply =
        include_str!("../../../../../zircon_plugins/animation/runtime/src/sequence/apply.rs");
    let plugin_target =
        include_str!("../../../../../zircon_plugins/animation/runtime/src/sequence/target.rs");
    let property_paths_runtime_mutation =
        include_str!("../../../scene/tests/property_paths/runtime_mutation.rs");
    let property_paths_read = include_str!("../../../scene/tests/property_paths/read_paths.rs");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let runtime_08 = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let ecs_doc = include_str!("../../../../../docs/zircon_runtime/scene/ecs.md");
    let animation_doc = include_str!(
        "../../../../../docs/assets-and-rendering/runtime-physics-animation-assets.md"
    );
    let editor_boundary_doc =
        include_str!("../../../../../docs/editor-and-tooling/runtime-editor-boundary-cleanup.md");
    let f17_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F17 |"))
        .expect("F17 review findings top row");

    assert!(
        f17_row
            .contains("f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred")
            && f17_row.ends_with("| convention + Runtime 08 / review closed |"),
        "F17 top row should record lookup review closed status"
    );

    let old_option_lookup = ["resolve", "entity", "path"].join("_");
    assert!(
        path_resolution
            .contains("pub fn get_entity_by_path(&self, path: &EntityPath) -> Option<EntityId>"),
        "F17 entity path Option lookup should use get_* naming"
    );
    assert!(
        !path_resolution.contains(&old_option_lookup),
        "F17 should hard-cut the old resolve-verb entity path Option API"
    );

    for (name, source) in [
        ("runtime animation apply", runtime_apply),
        ("runtime animation target", runtime_target),
        ("plugin animation apply", plugin_apply),
        ("plugin animation target", plugin_target),
        (
            "property path runtime mutation tests",
            property_paths_runtime_mutation,
        ),
        ("property path read tests", property_paths_read),
    ] {
        assert!(
            source.contains("get_entity_by_path("),
            "F17 consumer `{name}` should use get_entity_by_path"
        );
        assert!(
            !source.contains(&old_option_lookup),
            "F17 consumer `{name}` should not keep the old resolve-verb entity path lookup"
        );
    }

    for doc_anchor in [
        "F17 entity path Option lookup verb rename",
        "runtime_08_entity_path_lookup_getter_rename_coremin_check_passed",
        "f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred",
        "review_f17_entity_path_option_lookup_uses_get_verb",
        "get_entity_by_path",
        "old resolve-verb entity path method absent",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || runtime_08.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || ecs_doc.contains(doc_anchor)
                || animation_doc.contains(doc_anchor)
                || editor_boundary_doc.contains(doc_anchor),
            "F17 docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f18_asset_manager_resolution_returns_registered_handle() {
    let resolver =
        include_str!("../../../asset/pipeline/manager/asset_manager/resolve_asset_manager.rs");
    let handle =
        include_str!("../../../asset/pipeline/manager/asset_manager/asset_manager_handle.rs");
    let runtime = include_str!("../../../core/runtime/runtime.rs");
    let runtime_handle = include_str!("../../../core/runtime/handle/resolution.rs");
    let project_session = include_str!("../../../dynamic_api/session/project.rs");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let runtime_10 = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let dynamic_session_doc =
        include_str!("../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let asset_facade_doc = include_str!("../../../../../docs/zircon_runtime/asset/facade.md");
    let f18_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F18 |"))
        .expect("F18 review findings top row");

    assert!(
        f18_row
            .contains("f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred")
            && f18_row.ends_with("| Runtime 10 / review closed |"),
        "F18 top row should record manager-resolution review closed status"
    );

    for generic_manager_anchor in [
        "pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError>",
        "self.handle().resolve_manager(name)",
        "let service = self.resolve_named_service(name, Some(ServiceKind::Manager))?;",
        "downcast_resolved_service(name, service)",
    ] {
        assert!(
            runtime.contains(generic_manager_anchor)
                || runtime_handle.contains(generic_manager_anchor),
            "F18 generic manager resolution should keep registered-handle shape `{generic_manager_anchor}`"
        );
    }

    for required in [
        "pub fn resolve_asset_manager(core: &CoreHandle) -> Result<Arc<AssetManagerHandle>, CoreError>",
        "core.resolve_manager::<AssetManagerHandle>(ASSET_MANAGER_NAME)",
    ] {
        assert!(
            resolver.contains(required),
            "F18 asset manager resolver should contain `{required}`"
        );
    }
    for forbidden in [
        "Result<Arc<dyn AssetManager>, CoreError>",
        ".map(|holder| holder.shared())",
    ] {
        assert!(
            !resolver.contains(forbidden),
            "F18 asset manager resolver should not return trait objects directly or hide handle conversion `{forbidden}`"
        );
    }
    assert!(
        handle.contains("pub struct AssetManagerHandle")
            && handle.contains("inner: Arc<dyn AssetManager>")
            && handle.contains("pub fn shared(&self) -> Arc<dyn AssetManager>"),
        "AssetManagerHandle should remain the registered manager handle that owns the object-safe shared service"
    );
    assert!(
        project_session.contains("resolve_asset_manager(core)")
            && project_session.contains("let asset_manager = asset_manager.shared();")
            && project_session.contains(".open_project(&self.root_display())"),
        "dynamic project startup should make the trait-object conversion explicit at the caller boundary"
    );

    for doc_anchor in [
        "F18 asset manager resolution return shape",
        "runtime_10_asset_manager_resolution_handle_shape_coremin_check_passed",
        "f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred",
        "review_f18_asset_manager_resolution_returns_registered_handle",
        "Result<Arc<AssetManagerHandle>, CoreError>",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || runtime_10.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || dynamic_session_doc.contains(doc_anchor)
                || asset_facade_doc.contains(doc_anchor),
            "F18 docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f19_scene_renderer_construction_modules_use_construct_names() {
    let core_mod = include_str!("../../../graphics/scene/scene_renderer/core/mod.rs");
    let core_construct_mod = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_core_construct/mod.rs"
    );
    let core_construct_layouts = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_core_construct/layouts/mod.rs"
    );
    let core_construct_scene_bind_group = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_core_construct/scene_bind_group_bundle/mod.rs"
    );
    let renderer_construct_mod =
        include_str!("../../../graphics/scene/scene_renderer/core/scene_renderer_construct/mod.rs");
    let renderer_construct = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_construct/construct.rs"
    );
    let renderer_construct_new_with_icon_source = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs"
    );
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let render_index = include_str!("../../../../../docs/plans/zircon_runtime/render/index.md");
    let runtime_15 = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let shadow_doc =
        include_str!("../../../../../docs/zircon_runtime/graphics/scene/scene_renderer/shadow.md");
    let f19_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F19 |"))
        .expect("F19 review findings top row");

    assert!(
        f19_row.contains(
            "f19_scene_renderer_construction_top_row_closed_status_static_passed_cargo_deferred"
        ) && f19_row.ends_with("| convention + render index / review closed |"),
        "F19 top row should record scene renderer construction naming review closed status"
    );

    let core_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/graphics/scene/scene_renderer/core");
    let old_core_construct_owner = ["scene_renderer_core", "new"].join("_");
    let old_renderer_construct_owner = ["scene_renderer", "new"].join("_");
    assert!(
        core_dir.join("scene_renderer_core_construct").is_dir()
            && core_dir.join("scene_renderer_construct").is_dir(),
        "F19 scene renderer construction owners should live in construct-named directories"
    );
    assert!(
        !core_dir.join(&old_core_construct_owner).exists()
            && !core_dir.join(&old_renderer_construct_owner).exists(),
        "F19 should hard-cut old *_new construction directories instead of keeping migration paths"
    );

    for required in [
        "mod scene_renderer_core_construct;",
        "mod scene_renderer_construct;",
    ] {
        assert!(
            core_mod.contains(required),
            "scene renderer core module wiring should contain `{required}`"
        );
    }

    for (name, source) in [
        ("core/mod.rs", core_mod),
        ("scene_renderer_core_construct/mod.rs", core_construct_mod),
        (
            "scene_renderer_core_construct/layouts/mod.rs",
            core_construct_layouts,
        ),
        (
            "scene_renderer_core_construct/scene_bind_group_bundle/mod.rs",
            core_construct_scene_bind_group,
        ),
        ("scene_renderer_construct/mod.rs", renderer_construct_mod),
        ("scene_renderer_construct/construct.rs", renderer_construct),
        (
            "scene_renderer_construct/new_with_icon_source.rs",
            renderer_construct_new_with_icon_source,
        ),
    ] {
        for forbidden in [&old_core_construct_owner, &old_renderer_construct_owner] {
            assert!(
                !source.contains(forbidden),
                "F19 should not leave old construction owner `{forbidden}` in {name}"
            );
        }
    }

    for doc_anchor in [
        "F19 scene renderer construction module rename",
        "render_scene_renderer_construct_modules_coremin_passed",
        "f19_scene_renderer_construction_top_row_closed_status_static_passed_cargo_deferred",
        "review_f19_scene_renderer_construction_modules_use_construct_names",
        "scene_renderer_core_construct",
        "scene_renderer_construct",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || render_index.contains(doc_anchor)
                || runtime_15.contains(doc_anchor)
                || shadow_doc.contains(doc_anchor),
            "F19 docs should record `{doc_anchor}`"
        );
    }
}
