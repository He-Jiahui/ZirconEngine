use super::*;

#[test]
fn world_property_writes_use_direct_optional_state_branches() {
    let write_source = include_str!("../../world/property_access/write.rs");
    let animation_write_source = include_str!("../../world/property_access/write/animation.rs");
    let physics_write_source = include_str!("../../world/property_access/write/physics.rs");

    assert!(animation_write_source.contains("let next = if next.is_empty() {"));
    assert!(animation_write_source.contains("None"));
    assert!(animation_write_source.contains("} else {"));
    assert!(animation_write_source.contains("Some(next)"));
    assert!(write_source.contains(
        "\"animationstatemachineplayer\" => self.set_animation_state_machine_player_property("
    ));
    assert!(write_source.contains("\"collider\" => self.set_collider_property"));
    assert!(physics_write_source.contains("pub(super) fn set_collider_property"));
    assert!(physics_write_source.contains("if let Some(material) = collider.material.as_ref()"));
    assert!(physics_write_source.contains("if material.id() == next"));
    assert!(physics_write_source.contains("return Ok(false);"));
    assert!(!animation_write_source.contains("(!next.is_empty()).then_some(next)"));
    assert!(!physics_write_source.contains(".is_some_and(|handle| handle.id() == next)"));
}

#[test]
fn world_property_writes_pre_size_normalized_segment_vector() {
    let write_source = include_str!("../../world/property_access/write.rs");
    let set_property_source = write_source
        .split("fn set_property_impl(")
        .nth(1)
        .and_then(|text| text.split("match component.as_str()").next())
        .expect("read set_property_impl setup");

    assert!(set_property_source.contains("let raw_segments = property_path.property_segments();"));
    assert!(set_property_source.contains("Vec::with_capacity(raw_segments.len())"));
    assert!(set_property_source.contains("for segment in raw_segments"));
    assert!(set_property_source.contains("segments.push(normalized_identifier(segment));"));
    assert!(!set_property_source.contains(".map(|segment| normalized_identifier(segment))"));
    assert!(!set_property_source.contains(".collect::<Vec<_>>()"));
}

#[test]
fn world_collider_shape_kind_write_matches_normalized_values_without_allocation() {
    let physics_write_source = include_str!("../../world/property_access/write/physics.rs");
    let shape_kind_source = physics_write_source
        .split("(shape, \"kind\") => {")
        .nth(1)
        .and_then(|text| text.split("if *shape == replacement").next())
        .expect("read collider shape kind write branch");

    assert!(shape_kind_source.contains("normalized_identifier_matches(&next_kind, \"box\")"));
    assert!(shape_kind_source.contains("normalized_identifier_matches(&next_kind, \"sphere\")"));
    assert!(shape_kind_source.contains("normalized_identifier_matches(&next_kind, \"capsule\")"));
    assert!(!shape_kind_source.contains("normalized_identifier(&next_kind).as_str()"));
}

#[test]
fn world_property_write_segment_expectation_uses_direct_candidate_loop() {
    let errors_source = include_str!("../../world/property_access/value_conversion/errors.rs");
    let expect_segment_source = errors_source
        .split("fn expect_segment(")
        .nth(1)
        .and_then(|text| text.split("fn unknown_property<T>").next())
        .expect("read expect_segment helper body");

    assert!(expect_segment_source.contains("for candidate in expected"));
    assert!(expect_segment_source.contains("if *candidate == actual"));
    assert!(expect_segment_source.contains("return Ok(());"));
    assert!(!expect_segment_source.contains("expected.iter().any("));
}

#[test]
fn world_transform_rotation_validation_sums_quaternion_length_directly() {
    let values_source = include_str!("../../world/property_access/value_conversion/values.rs");
    let validate_quat_source = values_source
        .split("fn validate_quat_array(")
        .nth(1)
        .and_then(|text| text.split("fn validate_finite_scalar").next())
        .expect("read validate_quat_array helper body");

    assert!(validate_quat_source.contains("let mut length_squared = 0.0;"));
    assert!(validate_quat_source.contains("for component in value"));
    assert!(validate_quat_source.contains("length_squared += component * component;"));
    assert!(validate_quat_source.contains("if length_squared <= Real::EPSILON"));
    assert!(!validate_quat_source.contains(".iter()"));
    assert!(!validate_quat_source.contains(".sum::<Real>()"));
}

#[test]
fn world_property_numeric_array_validation_uses_direct_finite_loop() {
    let values_source = include_str!("../../world/property_access/value_conversion/values.rs");
    let validate_finite_array_source = values_source
        .split("fn validate_finite_array(")
        .nth(1)
        .expect("read validate_finite_array helper body");

    assert!(validate_finite_array_source.contains("for component in value"));
    assert!(validate_finite_array_source.contains("if !component.is_finite()"));
    assert!(validate_finite_array_source.contains("return Err(SceneError::NonFinitePropertyValue"));
    assert!(validate_finite_array_source.contains("property_path: property_path.to_string()"));
    assert!(validate_finite_array_source.contains("expected,"));
    assert!(validate_finite_array_source.contains("Ok(())"));
    assert!(!validate_finite_array_source.contains(".iter().all("));
    assert!(!validate_finite_array_source.contains("Err(format!("));
}

#[test]
fn world_property_enum_parsers_match_normalized_values_without_allocation() {
    let domain_source = include_str!("../../world/property_access/value_conversion/domain.rs");
    let parse_mobility_source = domain_source
        .split("fn parse_mobility")
        .nth(1)
        .and_then(|text| text.split("fn parse_rigid_body_type").next())
        .expect("read parse_mobility helper body");
    let parse_rigid_body_source = domain_source
        .split("fn parse_rigid_body_type")
        .nth(1)
        .and_then(|text| text.split("fn parse_joint_kind").next())
        .expect("read parse_rigid_body_type helper body");
    let parse_joint_source = domain_source
        .split("fn parse_joint_kind")
        .nth(1)
        .and_then(|text| text.split("fn parse_combine_rule").next())
        .expect("read parse_joint_kind helper body");
    let parse_combine_source = domain_source
        .split("fn parse_combine_rule")
        .nth(1)
        .and_then(|text| text.split("fn combine_rule_label").next())
        .expect("read parse_combine_rule helper body");

    assert!(parse_mobility_source.contains("normalized_identifier_matches(value, \"dynamic\")"));
    assert!(parse_mobility_source.contains("normalized_identifier_matches(value, \"static\")"));
    assert!(
        parse_rigid_body_source.contains("normalized_identifier_matches(value, \"kinematic\")")
    );
    assert!(parse_joint_source.contains("normalized_identifier_matches(value, \"generic6dof\")"));
    assert!(parse_joint_source.contains("normalized_identifier_matches(value, \"sixdof\")"));
    assert!(parse_combine_source.contains("normalized_identifier_matches(value, \"multiply\")"));
    assert!(!parse_mobility_source.contains("normalized_identifier(value).as_str()"));
    assert!(!parse_rigid_body_source.contains("normalized_identifier(value).as_str()"));
    assert!(!parse_joint_source.contains("normalized_identifier(value).as_str()"));
    assert!(!parse_combine_source.contains("normalized_identifier(value).as_str()"));
}

#[test]
fn world_property_write_normalizer_pushes_identifier_characters_directly() {
    let identifiers_source =
        include_str!("../../world/property_access/value_conversion/identifiers.rs");
    let normalized_identifier_source = identifiers_source
        .split("fn normalized_identifier(value: &str) -> String")
        .nth(1)
        .and_then(|text| text.split("fn normalized_identifier_matches").next())
        .expect("read normalized_identifier helper body");

    assert!(normalized_identifier_source.contains("String::with_capacity(value.len())"));
    assert!(normalized_identifier_source.contains("for character in value.chars()"));
    assert!(normalized_identifier_source.contains("if character.is_ascii_alphanumeric()"));
    assert!(
        normalized_identifier_source.contains("normalized.push(character.to_ascii_lowercase());")
    );
    assert!(!normalized_identifier_source.contains(".filter(|ch|"));
    assert!(!normalized_identifier_source.contains(".map(|ch|"));
    assert!(!normalized_identifier_source.contains(".collect()"));
}

#[test]
fn world_property_value_conversion_errors_use_direct_result_branches() {
    let values_source = include_str!("../../world/property_access/value_conversion/values.rs");
    let domain_source = include_str!("../../world/property_access/value_conversion/domain.rs");
    let expect_i32_source = values_source
        .split("fn expect_i32")
        .nth(1)
        .and_then(|text| text.split("fn expect_vec3").next())
        .expect("read expect_i32 body");
    let resource_id_source = domain_source
        .split("fn expect_resource_id")
        .nth(1)
        .and_then(|text| text.split("fn expect_animation_parameter").next())
        .expect("read expect_resource_id body");

    assert!(
        expect_i32_source
            .contains("ScenePropertyValue::Integer(value) => match i32::try_from(value)")
    );
    assert!(
        expect_i32_source
            .contains("ScenePropertyValue::Unsigned(value) => match i32::try_from(value)")
    );
    assert!(expect_i32_source.contains("Ok(value) => Ok(value)"));
    assert!(expect_i32_source.contains("Err(_) => Err(SceneError::PropertyTypeMismatch"));
    assert!(expect_i32_source.contains("expected: \"i32 integer\".to_string()"));
    assert!(resource_id_source.contains("match ResourceId::from_str(&value)"));
    assert!(resource_id_source.contains("Ok(resource_id) => Ok(resource_id)"));
    assert!(resource_id_source.contains("Err(error) => Err(SceneError::InvalidPropertyResourceId"));
    assert!(resource_id_source.contains("source_message: error.to_string()"));
    assert!(!expect_i32_source.contains("Err(format!("));
    assert!(!resource_id_source.contains("Err(format!("));
    assert!(!expect_i32_source.contains(".map_err("));
    assert!(!resource_id_source.contains(".map_err("));
}

#[test]
fn world_property_value_conversion_facade_keeps_policy_owners_separate() {
    let facade = include_str!("../../world/property_access/value_conversion.rs");
    let compiled = include_str!("../../world/property_access/value_conversion/compiled.rs");
    let domain = include_str!("../../world/property_access/value_conversion/domain.rs");
    let errors = include_str!("../../world/property_access/value_conversion/errors.rs");
    let identifiers = include_str!("../../world/property_access/value_conversion/identifiers.rs");
    let values = include_str!("../../world/property_access/value_conversion/values.rs");

    for module in [
        "mod compiled;",
        "mod domain;",
        "mod errors;",
        "mod identifiers;",
        "mod values;",
    ] {
        assert!(facade.contains(module));
    }
    assert!(!facade.contains(" fn "));
    assert!(compiled.contains("fn compiled_property_expect_scalar("));
    assert!(domain.contains("fn set_animation_player_like_property"));
    assert!(errors.contains("fn property_type_error<T>("));
    assert!(identifiers.contains("fn canonical_component_field_key("));
    assert!(values.contains("fn validate_quat_array("));

    for (path, source) in [
        ("value_conversion.rs", facade),
        ("value_conversion/compiled.rs", compiled),
        ("value_conversion/domain.rs", domain),
        ("value_conversion/errors.rs", errors),
        ("value_conversion/identifiers.rs", identifiers),
        ("value_conversion/values.rs", values),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 300,
            "{path} should stay below the production owner budget; got {line_count} lines"
        );
    }
}
