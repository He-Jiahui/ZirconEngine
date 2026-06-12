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
        )
            && register.contains("self.update_short_path_lookup(&type_path, short_type_path);")
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
        runtime_registration.contains("if let Some(registration) = self.registrations.get(type_path)")
            && runtime_registration.contains("return Ok(registration);")
            && runtime_registration.contains("if let Some(resolved) = self.short_paths.get(type_path)")
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
fn reflection_conversion_finite_checks_use_direct_loops() {
    let source = include_str!("../../reflect/conversion.rs");
    let reflected_value = source
        .split("fn ensure_finite_reflected_value(")
        .nth(1)
        .and_then(|text| text.split("fn ensure_finite_scalar(").next())
        .expect("read ensure_finite_reflected_value body");
    let vector = source
        .split("fn ensure_finite_vector(")
        .nth(1)
        .expect("read ensure_finite_vector body");

    assert!(
        reflected_value.contains("for value in values {")
            && reflected_value.contains("ensure_finite_reflected_value(value, target)?;")
            && reflected_value.contains("for value in values.values() {")
            && !reflected_value.contains(".try_for_each(")
            && vector.contains("for value in values {")
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
fn reflection_json_conversion_uses_direct_error_branch() {
    let source = include_str!("../../reflect/conversion.rs");
    let json_from_reflected = source
        .split("pub fn json_from_reflected(")
        .nth(1)
        .and_then(|text| text.split("fn unsupported_reflected_to_scene").next())
        .expect("read json_from_reflected body");
    let type_source = source
        .split("fn reflected_value_type_source(")
        .nth(1)
        .and_then(|text| text.split("fn ensure_finite_reflected_value").next())
        .expect("read reflected value type source helper");

    assert!(
        json_from_reflected.contains("match serde_json::to_value(&value)")
            && json_from_reflected.contains("Ok(value) => Ok(value)")
            && json_from_reflected.contains("Err(_) => Err(ReflectError::UnsupportedConversion")
            && json_from_reflected.contains("source: reflected_value_type_source(value.type_name())")
            && !json_from_reflected.contains(".map_err(")
            && !json_from_reflected.contains("format!(\"ReflectedValue::{}\"")
            && type_source.contains("const PREFIX: &str = \"ReflectedValue::\";")
            && type_source.contains("String::with_capacity(PREFIX.len() + type_name.len())")
            && type_source.contains("source.push_str(PREFIX);")
            && type_source.contains("source.push_str(type_name);")
            && !type_source.contains("format!("),
        "reflection JSON conversion must use a direct serde error branch and pre-sized source string helper"
    );
}

#[test]
fn fixed_reflection_shared_component_helpers_use_direct_branches() {
    let source = include_str!("../../reflect/fixed/shared.rs");
    let ensure_entity = source
        .split("pub(super) fn ensure_entity")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn get_component").next())
        .expect("read fixed shared ensure_entity helper");
    let get_component = source
        .split("pub(super) fn get_component<'a, T>(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn get_component_mut").next())
        .expect("read fixed shared get_component helper");
    let get_component_mut = source
        .split("pub(super) fn get_component_mut<'a, T>(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn ensure_component").next())
        .expect("read fixed shared get_component_mut helper");
    let ensure_component = source
        .split("pub(super) fn ensure_component<T>")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn missing_component").next())
        .expect("read fixed shared ensure_component helper");
    let expect_i32 = source
        .split("pub(super) fn expect_i32(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn expect_scalar(").next())
        .expect("read fixed shared expect_i32 helper");
    let remove_component = source
        .split("pub(super) fn remove_component<T>(")
        .nth(1)
        .expect("read fixed shared remove_component helper");

    assert!(
        ensure_entity.contains("if world.contains_entity(entity) {")
            && ensure_entity.contains("return Ok(());")
            && ensure_entity.contains("Err(ReflectError::MissingEntity { entity })")
            && !ensure_entity.contains(".then_some(())")
            && !ensure_entity.contains(".ok_or(")
            && get_component.contains("let Some(component) = world.get::<T>(entity) else")
            && get_component.contains("return Err(missing_component(entity, type_path));")
            && get_component.contains("Ok(component)")
            && !get_component.contains(".ok_or_else(")
            && get_component_mut.contains("let Some(component) = world.get_mut::<T>(entity) else")
            && get_component_mut.contains("return Err(missing_component(entity, type_path));")
            && get_component_mut.contains("Ok(component)")
            && !get_component_mut.contains("world.get::<T>(entity).is_none()")
            && !get_component_mut.contains(".ok_or_else(")
            && ensure_component.contains("if world.get::<T>(entity).is_none() {")
            && ensure_component.contains("return Err(missing_component(entity, type_path));")
            && ensure_component.contains("Ok(())")
            && !ensure_component.contains(".then_some(())")
            && !ensure_component.contains(".ok_or_else(")
            && expect_i32.contains("match i32::try_from(value)")
            && expect_i32.contains("Ok(value) => Ok(value)")
            && expect_i32.contains("Err(_) => Err(invalid_value(")
            && !expect_i32.contains(".map_err(")
            && remove_component.contains("match world.remove::<T>(entity) {")
            && remove_component.contains("Ok(Some(_)) => Ok(true),")
            && remove_component.contains("Err(_) => Err(missing_component(entity, type_path)),")
            && !remove_component.contains(".map_err(|_| missing_component(entity, type_path))"),
        "fixed reflection shared helpers must use direct success/error branches instead of Option/Result adapter projections"
    );
}

#[test]
fn fixed_reflection_vector_expectations_use_direct_finite_checks() {
    let source = include_str!("../../reflect/fixed/shared.rs");
    let expect_vec3 = source
        .split("pub(super) fn expect_vec3(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn expect_vec2(").next())
        .expect("read fixed shared expect_vec3 helper");
    let expect_vec2 = source
        .split("pub(super) fn expect_vec2(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn expect_vec4(").next())
        .expect("read fixed shared expect_vec2 helper");
    let expect_vec4 = source
        .split("pub(super) fn expect_vec4(")
        .nth(1)
        .and_then(|text| text.split("fn vec2_components_are_finite").next())
        .expect("read fixed shared expect_vec4 helper");
    let finite_helpers = source
        .split("fn vec2_components_are_finite")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn remove_component").next())
        .expect("read fixed shared vector finite helpers");

    assert!(
        expect_vec2.contains("ReflectedValue::Vec2(value) if vec2_components_are_finite(&value)")
            && expect_vec3
                .contains("ReflectedValue::Vec3(value) if vec3_components_are_finite(&value)")
            && expect_vec4
                .contains("ReflectedValue::Vec4(value) if vec4_components_are_finite(&value)")
            && !expect_vec2.contains(".iter().all(")
            && !expect_vec3.contains(".iter().all(")
            && !expect_vec4.contains(".iter().all(")
            && finite_helpers.contains("value[0].is_finite() && value[1].is_finite()")
            && finite_helpers.contains(
                "value[0].is_finite() && value[1].is_finite() && value[2].is_finite()"
            )
            && finite_helpers.contains("value[3].is_finite()")
            && !finite_helpers.contains(".iter()")
            && !finite_helpers.contains(".all("),
        "fixed reflection vector expectations must use direct fixed-array finite checks instead of iterator all adapters"
    );
}

#[test]
fn fixed_reflection_simple_adapters_use_direct_error_branches() {
    let active_in_hierarchy = include_str!("../../reflect/fixed/active_in_hierarchy.rs");
    let active_self = include_str!("../../reflect/fixed/active_self.rs");
    let name = include_str!("../../reflect/fixed/name.rs");
    let render_layer_mask = include_str!("../../reflect/fixed/render_layer_mask.rs");

    let active_in_hierarchy_read = active_in_hierarchy
        .split("fn read_field(")
        .nth(1)
        .and_then(|text| text.split("fn read_fields(").next())
        .expect("read ActiveInHierarchy read_field body");
    let active_self_write = active_self
        .split("fn write_field(")
        .nth(1)
        .and_then(|text| text.split("fn remove(").next())
        .expect("read ActiveSelf write_field body");
    let name_write = name
        .split("fn write_field(")
        .nth(1)
        .and_then(|text| text.split("fn remove(").next())
        .expect("read Name write_field body");
    let render_layer_mask_write = render_layer_mask
        .split("fn write_field(")
        .nth(1)
        .and_then(|text| text.split("fn remove(").next())
        .expect("read RenderLayerMask write_field body");

    assert!(
        active_in_hierarchy_read
            .contains("let Some(value) = world.active_in_hierarchy(entity) else")
            && active_in_hierarchy_read
                .contains("return Err(shared::missing_component(entity, TYPE_PATH));")
            && active_in_hierarchy_read.contains("Ok(ReflectedValue::Bool(value))")
            && !active_in_hierarchy_read.contains(".map(ReflectedValue::Bool)")
            && !active_in_hierarchy_read.contains(".ok_or_else(")
            && active_self_write.contains("match world.insert(entity, ActiveSelf(next))")
            && active_self_write.contains("Ok(_) => Ok(true)")
            && active_self_write.contains("Err(_) => Err(shared::missing_component(entity, TYPE_PATH))")
            && !active_self_write.contains(".map_err(")
            && name_write.contains("match world.insert(entity, Name(next))")
            && name_write.contains("Ok(_) => Ok(true)")
            && name_write.contains("Err(_) => Err(shared::missing_component(entity, TYPE_PATH))")
            && !name_write.contains(".map_err(")
            && render_layer_mask_write
                .contains("match world.insert(entity, RenderLayerMask(next))")
            && render_layer_mask_write.contains("Ok(_) => Ok(true)")
            && render_layer_mask_write
                .contains("Err(_) => Err(shared::missing_component(entity, TYPE_PATH))")
            && !render_layer_mask_write.contains(".map_err("),
        "simple fixed reflection adapters must use direct read/write error branches instead of map/ok_or/map_err adapters"
    );
}

#[test]
fn fixed_reflection_world_conversion_writes_use_direct_error_branches() {
    let shared = include_str!("../../reflect/fixed/shared.rs");
    let hierarchy = include_str!("../../reflect/fixed/hierarchy.rs");
    let mobility = include_str!("../../reflect/fixed/mobility.rs");

    let field_target = shared
        .split("pub(super) fn field_target")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn type_mismatch").next())
        .expect("read fixed shared field_target helper");
    let hierarchy_write = hierarchy
        .split("fn write_field(")
        .nth(1)
        .and_then(|text| text.split("fn remove(").next())
        .expect("read Hierarchy write_field body");
    let mobility_write = mobility
        .split("fn write_field(")
        .nth(1)
        .and_then(|text| text.split("fn remove(").next())
        .expect("read Mobility write_field body");

    assert!(
        field_target.contains("String::with_capacity(type_path.len() + 1 + field_name.len())")
            && field_target.contains("target.push_str(type_path);")
            && field_target.contains("target.push('.');")
            && field_target.contains("target.push_str(field_name);")
            && !field_target.contains("format!(")
            && hierarchy_write.contains("match world.set_parent_checked(entity, parent)")
            && hierarchy_write.contains("Ok(changed) => Ok(changed)")
            && hierarchy_write.contains("Err(error) => Err(ReflectError::UnsupportedConversion")
            && hierarchy_write.contains("target: shared::field_target(TYPE_PATH, field_name)")
            && !hierarchy_write.contains(".map_err(")
            && !hierarchy_write.contains("format!(\"{TYPE_PATH}.{field_name}\")")
            && mobility_write.contains("match world.set_mobility(entity, parse_mobility(&kind)?)")
            && mobility_write.contains("Ok(changed) => Ok(changed)")
            && mobility_write.contains("Err(error) => Err(ReflectError::UnsupportedConversion")
            && mobility_write.contains("target: shared::field_target(TYPE_PATH, field_name)")
            && !mobility_write.contains(".map_err(")
            && !mobility_write.contains("format!(\"{TYPE_PATH}.{field_name}\")"),
        "Hierarchy and Mobility reflection writes must use direct conversion-error branches and the shared pre-sized field target"
    );
}
