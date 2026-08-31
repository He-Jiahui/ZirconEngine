fn dynamic_components_source() -> &'static str {
    include_str!("../world/dynamic_components.rs")
}

fn component_type_registry_source() -> &'static str {
    include_str!("../world/component_type_registry.rs")
}

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .unwrap_or_else(|| panic!("read section from {start} to {end}"))
}

#[test]
fn dynamic_component_descriptor_projections_use_pre_sized_vectors() {
    let source = dynamic_components_source();
    let entity_projection = section_between(
        source,
        "pub fn dynamic_components_for_entity",
        "pub fn remove_dynamic_component",
    );
    let descriptors = section_between(
        source,
        "pub fn component_type_descriptors",
        "pub fn set_dynamic_component",
    );

    assert!(
        entity_projection.contains("let mut instances = Vec::with_capacity(components.len());")
            && entity_projection.contains("for (component_id, value) in components")
            && entity_projection.contains("instances.push(DynamicComponentInstance")
            && entity_projection
                .contains("descriptor: self.component_types.descriptor(component_id).cloned()")
            && entity_projection.contains(
                "instances.sort_by(|left, right| left.component_id.cmp(&right.component_id));"
            )
            && !entity_projection.contains(".map(|(component_id, value)|")
            && !entity_projection.contains(".collect::<Vec<_>>()"),
        "dynamic_components_for_entity must pre-size and push instance snapshots directly while retaining descriptor projection and final component-id ordering"
    );
    assert!(
        descriptors.contains("let descriptors = self.component_types.descriptors();")
            && descriptors
                .contains("let mut result = Vec::with_capacity(descriptors.size_hint().0);")
            && descriptors.contains("for descriptor in descriptors")
            && descriptors.contains("result.push(descriptor);")
            && descriptors.contains("result")
            && !descriptors.contains(".collect()")
            && !descriptors.contains(".collect::<Vec<_>>()"),
        "dynamic component descriptor-list projection must pre-size the returned Vec and push descriptors directly instead of relying on collect growth"
    );
}

#[test]
fn dynamic_component_registration_moves_descriptor_after_reflection_setup() {
    let source = dynamic_components_source();
    let register = section_between(
        source,
        "pub fn register_component_type",
        "pub fn component_type_descriptor",
    );

    assert!(
        register.contains(
            "crate::scene::reflect::registration_from_component_descriptor(&descriptor)?"
        ) && register.contains("return Err(ReflectError::DuplicateTypePath")
            && register.contains(".into());")
            && register.contains("let component =")
            && register.contains(
                "crate::scene::reflect::reflect_component_for_dynamic_descriptor(&descriptor);"
            )
            && register.contains("self.component_types.register(descriptor)?;")
            && register.contains("self.type_registry.register(RuntimeTypeRegistration")
            && register.contains("})?;")
            && register.contains("component: Some(component),")
            && !register.contains("self.component_types.register(descriptor.clone())")
            && !register.contains("reflect_component_for_dynamic_descriptor(&descriptor),")
            && !register.contains("error.to_string()")
            && !register.contains("Result<(), String>"),
        "dynamic component registration must build reflection state and preserve typed SceneError sources before moving the descriptor into the registry"
    );
}

#[test]
fn dynamic_component_single_value_and_property_reads_use_direct_branches() {
    let source = dynamic_components_source();
    let dynamic_component = section_between(
        source,
        "pub fn dynamic_component",
        "pub fn dynamic_components_for_entity",
    );
    let property_read = section_between(
        source,
        "pub(crate) fn dynamic_component_property",
        "pub(crate) fn set_dynamic_component_property",
    );

    assert!(
        dynamic_component.contains("let components = self.dynamic_components.get(&entity)?;")
            && dynamic_component.contains("components.get(component_id)")
            && !dynamic_component.contains(".and_then(|components| components.get(component_id))"),
        "dynamic component reads must fetch the entity component map through a direct branch"
    );
    assert!(
        property_read.contains("let value = self.dynamic_component(entity, component_id)?;")
            && property_read.contains("let value = json_property(value, property)?;")
            && property_read.contains("scene_value_from_json(value)")
            && !property_read
                .contains("json_property(value, property).and_then(scene_value_from_json)"),
        "dynamic component property reads must project JSON values through direct branches"
    );
}

#[test]
fn dynamic_component_property_writes_split_and_insert_only_at_map_boundaries() {
    let source = dynamic_components_source();
    let write_property = section_between(
        source,
        "pub(crate) fn set_dynamic_component_property",
        "fn validate_dynamic_component_type",
    );
    let split_helper = section_between(
        source,
        "fn split_dynamic_property_path",
        "fn dynamic_component_belongs_to_plugin",
    );

    assert!(
        write_property.contains(
            "let Some((component_id, property)) = split_dynamic_property_path(property_path) else"
        ) && write_property.contains("SceneError::UnknownDynamicComponentProperty")
            && write_property.contains("self.validate_dynamic_component_type(component_id)?;")
            && write_property.contains(
                "self.validate_dynamic_component_property_write(component_id, property)?;"
            )
            && write_property.contains(".entry(component_id.to_string())")
            && write_property.contains("if object.get(property) == Some(&json_value)")
            && write_property.contains("object.insert(property.to_string(), json_value);")
            && write_property
                .contains("self.insert_dynamic_component_presence(entity, component_id)?;")
            && !write_property.contains("Err(format!")
            && !write_property.contains(".ok_or_else"),
        "dynamic component property writes should allocate owned strings only at the map insertion points"
    );
    assert!(
        split_helper.contains("Option<(&str, &str)>")
            && split_helper.contains("property_path.as_str().rsplit_once('.')")
            && !split_helper.contains("Option<(String, String)>")
            && !split_helper.contains("component_id.to_string()")
            && !split_helper.contains("property.to_string()"),
        "dynamic component property split helper must return borrowed component/property segments"
    );
}

#[test]
fn dynamic_component_json_numbers_use_f32_short_decimal_projection() {
    let source = dynamic_components_source();
    let number_helper = source
        .split("fn finite_json_number")
        .nth(1)
        .expect("read dynamic component JSON number helper");

    assert!(
        number_helper.contains("if !value.is_finite()")
            && number_helper.contains("let text = value.to_string();")
            && number_helper.contains("text.parse::<f64>()")
            && number_helper.contains("Number::from_f64(value)")
            && !number_helper.contains("value as f64")
            && !number_helper.contains(".then(||"),
        "dynamic component JSON number writes must project f32 values through their shortest decimal form instead of expanding binary precision"
    );
}

#[test]
fn dynamic_component_plugin_match_and_property_descriptor_scan_use_direct_branches() {
    let source = dynamic_components_source();
    let plugin_matcher = section_between(
        source,
        "fn dynamic_component_belongs_to_plugin",
        "fn json_property",
    );
    let write_validation = section_between(
        source,
        "fn validate_dynamic_component_property_write",
        "fn split_dynamic_property_path",
    );

    assert!(
        plugin_matcher.contains("let Some(suffix) = component_id.strip_prefix(plugin_id) else")
            && plugin_matcher.contains("return false;")
            && plugin_matcher.contains("suffix.starts_with('.')")
            && !plugin_matcher.contains(".is_some_and(")
            && !source.contains("format!(\"{plugin_id}.\")")
            && !source.contains("prefix.clone()")
            && !source.contains(".starts_with(&prefix)"),
        "dynamic component plugin matching must branch directly from the borrowed plugin prefix"
    );
    assert!(
        write_validation.contains("let mut property_descriptor = None;")
            && write_validation.contains("for descriptor in &descriptor.properties")
            && write_validation.contains("if descriptor.name == property")
            && write_validation.contains("property_descriptor = Some(descriptor);")
            && write_validation.contains("break;")
            && write_validation
                .contains("let Some(property_descriptor) = property_descriptor else")
            && !write_validation.contains(".iter().find("),
        "dynamic component property write validation must scan descriptors directly"
    );
}

#[test]
fn dynamic_component_plugin_count_and_unload_scan_maps_directly() {
    let source = dynamic_components_source();
    let count_body = section_between(
        source,
        "pub fn dynamic_component_count_for_plugin",
        "pub fn ensure_plugin_components_can_unload",
    );
    let unload_body = section_between(
        source,
        "pub fn ensure_plugin_components_can_unload",
        "pub(crate) fn dynamic_component_property",
    );

    assert!(
        count_body.contains("let mut count = 0_usize;")
            && count_body.contains("for components in self.dynamic_components.values()")
            && count_body.contains("for component_id in components.keys()")
            && count_body.contains("dynamic_component_belongs_to_plugin(component_id, plugin_id)")
            && count_body.contains("count += 1;")
            && count_body.contains("count")
            && !count_body.contains("dynamic_component_refs_for_plugin(plugin_id).count()"),
        "dynamic component plugin counts must scan component maps directly"
    );
    assert!(
        source.contains("use std::fmt::Write as _;")
            && unload_body.contains("let mut active_components = String::new();")
            && unload_body.contains("let mut has_active_components = false;")
            && unload_body.contains("for (entity, components) in &self.dynamic_components")
            && unload_body.contains("for component_id in components.keys()")
            && unload_body
                .contains("if !dynamic_component_belongs_to_plugin(component_id, plugin_id)")
            && unload_body.contains("continue;")
            && unload_body.contains("active_components.push_str(\", \");")
            && unload_body.contains("has_active_components = true;")
            && unload_body.contains("let _ = write!")
            && unload_body.contains("\"{component_id} on entity {entity}\"")
            && unload_body.contains("if !has_active_components")
            && !unload_body.contains("self.dynamic_component_refs_for_plugin(plugin_id)")
            && !unload_body.contains(".map(|(entity, component_id)| format!")
            && !unload_body.contains(".collect::<Vec<_>>()")
            && !unload_body.contains(".join(\", \")")
            && !source.contains("fn dynamic_component_refs_for_plugin"),
        "dynamic component unload diagnostics must stream matching plugin components from direct nested scans"
    );
}

#[test]
fn dynamic_component_type_registry_validates_borrowed_plugin_prefix_without_formatting() {
    let source = component_type_registry_source();
    let register = section_between(source, "pub fn register", "pub fn descriptor");
    let plugin_matcher = source
        .split("fn component_type_belongs_to_plugin")
        .nth(1)
        .expect("read component type registry plugin matcher helper");

    assert!(
        register.contains(
            "if !component_type_belongs_to_plugin(&descriptor.type_id, &descriptor.plugin_id)"
        ) && register.contains("SceneError::ComponentTypePluginPrefixMismatch")
            && register.contains("SceneError::DuplicateComponentType")
            && plugin_matcher.contains(".strip_prefix(plugin_id)")
            && plugin_matcher.contains("let Some(suffix) = type_id.strip_prefix(plugin_id) else")
            && plugin_matcher.contains("return false;")
            && plugin_matcher.contains("suffix.starts_with('.')")
            && !plugin_matcher.contains(".is_some_and(")
            && !register.contains("Result<(), String>")
            && !register.contains("Err(format!")
            && !register.contains("format!(\"{}.\", descriptor.plugin_id)")
            && !register.contains("expected_prefix")
            && !register.contains("starts_with(&expected_prefix)"),
        "dynamic component type registration must validate the borrowed plugin id plus dot suffix without formatting an expected prefix String or building a predicate adapter"
    );
}

#[test]
fn dynamic_component_json_vectors_project_without_temporary_vec() {
    let source = dynamic_components_source();
    let vector_projection = section_between(
        source,
        "fn scene_vector_from_json",
        "fn scene_object_from_json",
    );

    assert!(
        vector_projection.contains("match values")
            && vector_projection.contains("[x, y] => Some(ScenePropertyValue::Vec2([")
            && vector_projection.contains("[x, y, z] => Some(ScenePropertyValue::Vec3([")
            && vector_projection.contains("[x, y, z, w] => Some(ScenePropertyValue::Vec4([")
            && vector_projection.contains("json_number_as_f32(x)?")
            && vector_projection.contains("fn json_number_as_f32(value: &Value) -> Option<f32>")
            && !vector_projection.contains(".collect::<Option<Vec<_>>>()")
            && !vector_projection.contains("values.as_slice()"),
        "dynamic component JSON vector projection must match array slices directly without building a temporary numeric Vec"
    );
}

#[test]
fn dynamic_component_json_vector_writes_pre_size_output_array() {
    let source = dynamic_components_source();
    let vector_write = section_between(source, "fn vector_to_json", "fn finite_json_number");

    assert!(
        vector_write.contains("let mut array = Vec::with_capacity(N);")
            && vector_write.contains("for value in values")
            && vector_write.contains("array.push(Value::Number(finite_json_number(value)?));")
            && vector_write.contains("Some(Value::Array(array))")
            && source.contains("fn finite_json_number(value: f32) -> Option<Number>")
            && !vector_write.contains(".collect::<Option<Vec<_>>>()")
            && !vector_write.contains(".map(Value::Array)"),
        "dynamic component JSON vector writes must pre-size the JSON array and push finite numbers directly"
    );
}

#[test]
fn dynamic_component_json_property_conversions_use_direct_branches() {
    let source = dynamic_components_source();
    let json_read = section_between(
        source,
        "fn scene_value_from_json",
        "fn json_from_scene_value",
    );
    let json_write = section_between(
        source,
        "fn json_from_scene_value",
        "fn scene_vector_from_json",
    );
    let number_helper =
        section_between(source, "fn json_number_as_f32", "fn scene_object_from_json");
    let object_read = section_between(
        source,
        "fn scene_object_from_json",
        "fn single_property_object",
    );

    assert!(
        json_read.contains("if let Some(value) = value.as_i64()")
            && json_read.contains("return Some(ScenePropertyValue::Integer(value));")
            && json_read.contains("if let Some(value) = value.as_u64()")
            && json_read.contains("return Some(ScenePropertyValue::Unsigned(value));")
            && json_read.contains("match value.as_f64()")
            && json_read.contains("Some(value) => Some(ScenePropertyValue::Scalar(value as _))")
            && !json_read.contains(".map(ScenePropertyValue::Integer)")
            && !json_read.contains(".map(ScenePropertyValue::Unsigned)")
            && !json_read.contains(".or_else("),
        "dynamic component JSON number reads must project signed, unsigned, and scalar values through direct branches"
    );
    assert!(
        json_write.contains("let entity = match value")
            && json_write.contains("Some(entity) => Value::Number(Number::from(entity))")
            && json_write.contains("None => Value::Null")
            && !json_write.contains(".unwrap_or(Value::Null)")
            && !json_write.contains(".map(|entity| Value::Number"),
        "dynamic component nullable entity JSON writes must use a direct option branch"
    );
    assert!(
        number_helper.contains("match value.as_f64()")
            && number_helper.contains("Some(value) => Some(value as _)")
            && number_helper.contains("None => None")
            && !number_helper.contains(".map(|value| value as _)"),
        "dynamic component JSON number helper must avoid a projection closure"
    );
    assert!(
        object_read.contains("if let Some(value) = object.get(\"resource\")")
            && object_read.contains("if let Some(value) = value.as_str()")
            && object_read.contains("Value::Number(number) => match number.as_u64()")
            && object_read
                .contains("Some(entity) => Some(ScenePropertyValue::Entity(Some(entity)))")
            && !object_read.contains(".and_then(Value::as_str)")
            && !object_read.contains(".map(|entity| ScenePropertyValue::Entity"),
        "dynamic component JSON object reads must branch directly for resource and entity wrapper objects"
    );
}

#[test]
fn dynamic_component_json_scalar_writes_use_finite_number_helper() {
    let source = dynamic_components_source();
    let scalar_write = section_between(
        source,
        "ScenePropertyValue::Scalar(value)",
        "ScenePropertyValue::String",
    );
    let number_helper = source
        .split("fn finite_json_number")
        .nth(1)
        .expect("read dynamic component JSON number helper");

    assert!(
        scalar_write.contains("=> match finite_json_number(value)")
            && scalar_write.contains("Some(number) => Some(Value::Number(number))")
            && scalar_write.contains("None => None")
            && source.contains("fn finite_json_number(value: f32) -> Option<Number>")
            && !scalar_write.contains(".map(Value::Number)")
            && !scalar_write.contains(".to_string()"),
        "dynamic component scalar JSON writes must use the finite-number helper through a direct branch instead of closure adapters"
    );
    assert!(
        number_helper.contains("let text = value.to_string();")
            && number_helper.contains("text.parse::<f64>()")
            && !number_helper.contains("value as f64"),
        "dynamic component finite-number helper must keep f32 JSON values on their shortest decimal projection"
    );
}

#[test]
fn dynamic_component_json_single_property_objects_are_pre_sized() {
    let source = dynamic_components_source();
    let json_write = section_between(
        source,
        "fn json_from_scene_value",
        "fn scene_vector_from_json",
    );
    let object_helper = section_between(source, "fn single_property_object", "fn vector_to_json");

    assert!(
        json_write.contains("Some(single_property_object(\"entity\", entity))")
            && json_write
                .contains("Some(single_property_object(\"resource\", Value::String(value)))")
            && !json_write.contains("Map::from_iter")
            && object_helper.contains("let mut object = Map::with_capacity(1);")
            && object_helper.contains("object.insert(key.to_string(), value);")
            && object_helper.contains("Value::Object(object)"),
        "dynamic component Entity/Resource JSON writes must pre-size one-property objects instead of routing through Map::from_iter"
    );
}

#[test]
fn vm_payload_validation_indexes_retained_components_before_registration_checks() {
    let source = dynamic_components_source();
    let validation = section_between(
        source,
        "fn validate_retained_vm_payloads(",
        "fn validate_dynamic_component_value_against_registration(",
    );

    assert!(
        validation.contains("let mut payloads_by_type = HashMap::with_capacity(")
            && validation.contains("for components in self.dynamic_components.values()")
            && validation.contains("payloads_by_type.entry(type_path.as_str())")
            && validation.contains("for registration in registrations")
            && validation.contains("payloads_by_type.get(type_path)")
            && !validation.contains(
                "for registration in registrations {\n            let type_path = registration.type_path.type_path();\n            for components in self.dynamic_components.values()"
            ),
        "VM catalog validation must index retained payloads once instead of rescanning every entity map for every registration"
    );
}
