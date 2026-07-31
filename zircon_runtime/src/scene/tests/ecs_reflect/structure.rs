#[test]
fn type_registry_register_borrows_short_path_during_lookup_update() {
    let source = include_str!("../../reflect/type_registry.rs");
    let register = source
        .split("pub fn register(&mut self, registration: RuntimeTypeRegistration)")
        .nth(1)
        .and_then(|text| text.split("pub fn register_resource").next())
        .expect("read TypeRegistry::register body");

    assert!(
        register.contains(
            "let short_type_path = registration.registration.type_path.short_type_path.as_str();"
        ) && register.contains("self.update_short_path_lookup(&type_path, short_type_path);")
            && register.contains("self.registrations.insert(type_path, registration);")
            && !register.contains("short_type_path.clone()")
            && !register.contains("self.update_short_path_lookup(&type_path, &short_type_path);"),
        "TypeRegistry::register must borrow the short type path for lookup maintenance before moving the runtime registration"
    );
}

#[test]
fn type_registry_contains_uses_direct_maps_without_resolve_error_allocation() {
    let source = include_str!("../../reflect/type_registry.rs");
    let contains = source
        .split("pub fn contains(&self, type_path: &str) -> bool")
        .nth(1)
        .and_then(|text| text.split("pub fn contains_type_path").next())
        .expect("read TypeRegistry::contains body");

    assert!(
        contains.contains("self.registrations.contains_key(type_path)")
            && contains.contains("self.short_paths.contains_key(type_path)")
            && !contains.contains("self.resolve(type_path).is_ok()"),
        "TypeRegistry::contains must answer from direct full-path and unambiguous short-path maps without constructing resolve errors"
    );
}

#[test]
fn type_registry_runtime_registration_uses_direct_lookup_before_error_paths() {
    let source = include_str!("../../reflect/type_registry.rs");
    let runtime_registration = source
        .split("pub fn runtime_registration(")
        .nth(1)
        .and_then(|text| text.split("pub fn resolve(&self, type_path: &str)").next())
        .expect("read TypeRegistry::runtime_registration body");

    assert!(
        runtime_registration
            .contains("if let Some(registration) = self.registrations.get(type_path)")
            && runtime_registration.contains("return Ok(registration);")
            && runtime_registration
                .contains("if let Some(resolved) = self.short_paths.get(type_path)")
            && runtime_registration.contains(".get(resolved)")
            && runtime_registration.contains("ReflectError::AmbiguousShortTypePath")
            && runtime_registration.contains("ReflectError::UnknownType")
            && !runtime_registration.contains("let resolved = self.resolve(type_path)?;"),
        "TypeRegistry::runtime_registration must look up full and short paths directly before constructing error payloads"
    );
}

#[test]
fn type_registry_registration_borrows_field_without_result_map_closure() {
    let source = include_str!("../../reflect/type_registry.rs");
    let registration = source
        .split("pub fn registration(&self, type_path: &str)")
        .nth(1)
        .and_then(|text| text.split("pub fn runtime_registration(").next())
        .expect("read TypeRegistry::registration body");

    assert!(
        registration.contains("Ok(&self.runtime_registration(type_path)?.registration)")
            && !registration.contains(".map(|registration| &registration.registration)"),
        "TypeRegistry::registration must borrow the reflected registration field directly without a Result::map closure"
    );
}

#[test]
fn type_registry_short_path_rebuild_streams_borrowed_registrations() {
    let source = include_str!("../../reflect/type_registry.rs");
    let rebuild = source
        .split("fn rebuild_short_path_lookup(&mut self)")
        .nth(1)
        .and_then(|text| text.split("}\n}").next())
        .expect("read TypeRegistry::rebuild_short_path_lookup body");

    assert!(
        rebuild.contains("for (type_path, registration) in &self.registrations")
            && rebuild.contains("Self::update_short_path_maps(")
            && !rebuild.contains(".collect::<Vec<_>>()")
            && !rebuild.contains("type_path.clone(),")
            && !rebuild.contains("short_type_path.clone(),"),
        "short-path rebuilds must stream borrowed registrations into replacement indexes without a cloned staging vector"
    );
}

#[test]
fn reflection_conversion_vector_finite_checks_use_direct_loops() {
    let source = include_str!("../../reflect/conversion.rs");
    let vector = source
        .split("fn ensure_finite_vector(")
        .nth(1)
        .expect("read ensure_finite_vector body");

    assert!(
        vector.contains("for value in values {")
            && vector.contains("if !value.is_finite()")
            && vector.contains("return Err(ReflectError::UnsupportedConversion")
            && !vector.contains(".iter()")
            && !vector.contains(".all(")
            && !vector.contains(".then_some(())"),
        "reflection finite-value validation must use direct loops instead of iterator adapters on nested DTO traversal"
    );
}

#[test]
fn reflection_conversion_scalar_finite_check_uses_direct_branch() {
    let source = include_str!("../../reflect/conversion.rs");
    let scalar = source
        .split("fn ensure_finite_scalar(")
        .nth(1)
        .and_then(|text| text.split("fn ensure_finite_vector(").next())
        .expect("read ensure_finite_scalar body");

    assert!(
        scalar.contains("if value.is_finite()")
            && scalar.contains("return Ok(());")
            && scalar.contains("Err(ReflectError::UnsupportedConversion")
            && !scalar.contains(".then_some(())")
            && !scalar.contains(".ok_or_else("),
        "reflection scalar finite validation must use a direct branch instead of Option/error closure setup"
    );
}

#[test]
fn reflection_json_persistence_stays_folder_backed_and_versioned() {
    let mod_source = include_str!("../../reflect/json_document/mod.rs");
    let read_source = include_str!("../../reflect/json_document/read.rs");
    let write_source = include_str!("../../reflect/json_document/write.rs");
    let migration_source = include_str!("../../reflect/json_document/migration.rs");
    assert!(
        mod_source.contains("mod migration;")
            && mod_source.contains("mod schema;")
            && read_source.contains("load_versioned::<ReflectedJsonDocument>")
            && write_source.contains("write_versioned_text")
            && migration_source.contains(
                "use zircon_runtime_interface::project::migrate_retired_asset_references;"
            )
            && migration_source.contains("migrate_retired_asset_references"),
        "reflected JSON persistence must remain folder-backed and delegate retired AssetRef value migration to the Runtime Interface contract"
    );
}

#[test]
fn reflection_hard_cut_removes_the_manual_fixed_adapter_tree() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixed_module = manifest_dir.join("src/scene/reflect/fixed/mod.rs");
    let reflect_root = include_str!("../../reflect/mod.rs");

    assert!(
        !fixed_module.exists() && !reflect_root.contains("mod fixed;"),
        "unified derived reflection must hard-cut the former scene/reflect/fixed adapter tree"
    );
}

#[test]
fn builtin_reflection_registration_uses_derived_adapters_for_plain_components() {
    let registration = include_str!("../../reflect/builtin_reflection/registration.rs");
    for component in [
        "Name",
        "LocalTransform",
        "ActiveSelf",
        "RenderLayerMask",
        "Mobility",
        "CameraComponent",
        "MeshRenderer",
        "AmbientLight",
        "DirectionalLight",
        "PointLight",
        "RectLight",
        "SpotLight",
        "RigidBodyComponent",
    ] {
        assert!(
            registration.contains(&format!("derived_component_registration::<{component}>()")),
            "plain reflected component {component} must use the unified derived adapter"
        );
    }
    assert!(
        registration.contains("hierarchy::registration()")
            && registration.contains("active_in_hierarchy::registration()"),
        "only components with world-owned invariants should use explicit builtin adapters"
    );
}

#[test]
fn builtin_reflection_special_adapters_only_enforce_world_owned_invariants() {
    let hierarchy = include_str!("../../reflect/builtin_reflection/hierarchy.rs");
    let active_in_hierarchy =
        include_str!("../../reflect/builtin_reflection/active_in_hierarchy.rs");

    assert!(
        hierarchy.contains("derived_component_registration_with_adapter::<Hierarchy>")
            && hierarchy.contains("world.set_parent_checked(entity, parent)")
            && active_in_hierarchy
                .contains("derived_component_registration_with_adapter::<ActiveInHierarchy>")
            && active_in_hierarchy.contains("world.active_in_hierarchy(entity)"),
        "special adapters must be limited to hierarchy-cycle validation and computed active state"
    );
}

#[test]
fn builtin_component_metadata_is_owned_by_zr_reflect_derives() {
    fn assert_component_owner(type_path: &str, owner: &str) {
        let type_path_offset = owner
            .find(type_path)
            .unwrap_or_else(|| panic!("component source must own {type_path}"));
        assert_eq!(
            owner.matches(type_path).count(),
            1,
            "component source must own exactly one canonical metadata declaration for {type_path}"
        );

        let declaration_prefix = &owner[..type_path_offset];
        let derive_offset = declaration_prefix.rfind("#[derive(").unwrap_or_else(|| {
            panic!("component declaration must derive ZrReflect for {type_path}")
        });
        let declaration_attributes = &declaration_prefix[derive_offset..];
        assert!(
            declaration_attributes.contains("zircon_reflect_derive::ZrReflect")
                && !declaration_attributes.contains("pub struct")
                && !declaration_attributes.contains("pub enum"),
            "ZrReflect must be bound to the declaration for {type_path}"
        );
    }

    let component_owners = [
        (
            "zircon_runtime::scene::components::Name",
            include_str!("../../components/scene/identity.rs"),
        ),
        (
            "zircon_runtime::scene::components::Hierarchy",
            include_str!("../../components/scene/hierarchy.rs"),
        ),
        (
            "zircon_runtime::scene::components::LocalTransform",
            include_str!("../../components/scene/transform.rs"),
        ),
        (
            "zircon_runtime::scene::components::ActiveSelf",
            include_str!("../../components/scene/activation.rs"),
        ),
        (
            "zircon_runtime::scene::components::ActiveInHierarchy",
            include_str!("../../components/scene/activation.rs"),
        ),
        (
            "zircon_runtime::scene::components::RenderLayerMask",
            include_str!("../../components/scene/activation.rs"),
        ),
        (
            "zircon_runtime::scene::components::CameraComponent",
            include_str!("../../components/scene/camera.rs"),
        ),
        (
            "zircon_runtime::scene::components::MeshRenderer",
            include_str!("../../components/scene/mesh_renderer.rs"),
        ),
        (
            "zircon_runtime::scene::components::RigidBodyComponent",
            include_str!("../../components/scene/physics.rs"),
        ),
    ];
    let lighting = include_str!("../../components/scene/lighting.rs");
    let mobility = include_str!("../../../core/framework/scene/mobility.rs");

    for &(type_path, owner) in &component_owners {
        assert_component_owner(type_path, owner);
    }
    for type_path in [
        "zircon_runtime::scene::components::AmbientLight",
        "zircon_runtime::scene::components::DirectionalLight",
        "zircon_runtime::scene::components::PointLight",
        "zircon_runtime::scene::components::RectLight",
        "zircon_runtime::scene::components::SpotLight",
    ] {
        assert!(
            lighting.contains(type_path),
            "lighting component source must own canonical reflected metadata for {type_path}"
        );
    }
    assert!(
        lighting.contains("zircon_reflect_derive::ZrReflect")
            && mobility.contains("zircon_reflect_derive::ZrReflect")
            && mobility.contains("zircon_runtime::core::framework::scene::Mobility"),
        "builtin metadata must be generated from component-owned ZrReflect derives"
    );
}

#[test]
fn derived_reflection_helpers_have_owner_scoped_visibility_and_world_lifetimes() {
    let registration = include_str!("../../reflect/builtin_reflection/registration.rs");
    let component_support = include_str!("../../reflect/builtin_reflection/component_support.rs");
    let local_transform = include_str!("../../components/scene/reflection/local_transform.rs");
    let mesh_renderer = include_str!("../../components/scene/reflection/mesh_renderer.rs");
    let rigid_body = include_str!("../../components/scene/reflection/rigid_body.rs");

    assert!(
        registration.contains("pub(in crate::scene::reflect) fn register(")
            && component_support.contains("fn get<'world, T>(")
            && component_support.contains("world: &'world World,")
            && component_support.contains("Result<&'world T, ReflectError>"),
        "builtin registration and borrowed component helpers must expose only the required reflection owner and bind returned references to World"
    );
    for source in [local_transform, mesh_renderer, rigid_body] {
        assert!(
            source.contains("pub(in crate::scene::components::scene) fn")
                && !source.contains("pub fn"),
            "derive accessors must be visible to their generated component owner without becoming crate-public"
        );
    }
}

#[test]
fn derived_component_write_reinserts_through_world_with_direct_error_branches() {
    let source = include_str!("../../reflect/derived/component_adapter.rs");
    let write_field = source
        .split("fn write_field<T>(")
        .nth(1)
        .and_then(|text| text.split("fn remove<T>").next())
        .expect("read derived component write_field body");

    assert!(
        write_field.contains("match world.insert(entity, next) {")
            && write_field.contains("Ok(_) => Ok(true),")
            && write_field.contains("Err(error) => Err(ReflectError::UnsupportedConversion")
            && !write_field.contains(".map(")
            && !write_field.contains(".map_err("),
        "derived component writes must re-enter World invariants with direct success/error branches"
    );
}

#[test]
fn derived_and_dynamic_component_adapters_expose_dense_field_slots() {
    let reflect_contract =
        include_str!("../../../../../zircon_runtime_interface/src/reflect/zr_reflect.rs");
    let reflect_component = include_str!("../../reflect/reflect_component.rs");
    let derived = include_str!("../../reflect/derived/component_adapter.rs");
    let dynamic = include_str!("../../reflect/dynamic_component.rs");
    let hierarchy = include_str!("../../reflect/builtin_reflection/hierarchy.rs");
    let active_in_hierarchy =
        include_str!("../../reflect/builtin_reflection/active_in_hierarchy.rs");

    assert!(
        reflect_contract.contains("fn read_reflected_field_by_slot(")
            && reflect_contract.contains("fn write_reflected_field_by_slot(")
            && reflect_component.contains("pub fn with_dense_field_slots(")
            && reflect_component.contains("pub fn read_field_by_slot(")
            && reflect_component.contains("pub fn write_field_by_slot(")
            && derived.contains(".with_dense_field_slots(")
            && derived.contains("read_reflected_field_by_slot(field_slot)")
            && derived.contains("write_reflected_field_by_slot(field_slot, value)")
            && dynamic.contains(".with_dense_field_slots(read_dense_slot, write_dense_slot)")
            && hierarchy
                .contains(".with_dense_field_slots(read_field_by_slot, write_field_by_slot)")
            && active_in_hierarchy
                .contains(".with_dense_field_slots(read_field_by_slot, write_field_by_slot)"),
        "all component reflection paths consumed by VM call sites must provide numeric field-slot adapters"
    );
}

#[test]
fn derived_component_registration_constructs_reflection_metadata_once() {
    let source = include_str!("../../reflect/derived/component_adapter.rs");
    let derived_registration = source
        .split("pub fn derived_component_registration<T>()")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn derived_component_registration_with_adapter<T>(")
                .next()
        })
        .expect("read default derived component registration body");
    let custom_adapter_registration = source
        .split("pub fn derived_component_registration_with_adapter<T>(")
        .nth(1)
        .and_then(|text| text.split("fn finish_component_registration(").next())
        .expect("read custom-adapter derived component registration body");

    assert!(
        derived_registration
            .matches("T::reflect_type_registration()?")
            .count()
            == 1
            && custom_adapter_registration
                .matches("T::reflect_type_registration()?")
                .count()
                == 1
            && derived_registration.contains("finish_component_registration(registration,")
            && custom_adapter_registration
                .contains("finish_component_registration(registration, component)"),
        "each derived adapter constructor must build reflected registration metadata exactly once"
    );
}

#[test]
fn dynamic_component_bulk_read_reuses_the_resolved_field_descriptor() {
    let source = include_str!("../../reflect/dynamic_component.rs");
    let read_fields = source
        .split("fn read_fields(")
        .nth(1)
        .and_then(|text| text.split("fn write_field(").next())
        .expect("read dynamic component read_fields body");

    assert!(
        read_fields.contains("read_declared_field(world, entity, type_path, field)?")
            && !read_fields.contains("read_field(world, entity, type_path, &field.name)?"),
        "bulk reflection reads must reuse each already-resolved field instead of re-querying the registry and rescanning the schema"
    );
}
