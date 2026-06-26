use super::*;

#[test]
fn world_property_writes_use_direct_optional_state_branches() {
    let write_source = include_str!("../../world/property_access/write.rs");

    assert!(write_source.contains("let next = if next.is_empty() {"));
    assert!(write_source.contains("None"));
    assert!(write_source.contains("} else {"));
    assert!(write_source.contains("Some(next)"));
    assert!(write_source.contains("if let Some(material) = collider.material.as_ref()"));
    assert!(write_source.contains("if material.id() == next"));
    assert!(write_source.contains("return Ok(false);"));
    assert!(!write_source.contains("(!next.is_empty()).then_some(next)"));
    assert!(!write_source.contains(".is_some_and(|handle| handle.id() == next)"));
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
    let write_source = include_str!("../../world/property_access/write.rs");
    let shape_kind_source = write_source
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
    let value_conversion_source = include_str!("../../world/property_access/value_conversion.rs");
    let expect_segment_source = value_conversion_source
        .split("pub(super) fn expect_segment(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn unknown_property_error").next())
        .expect("read expect_segment helper body");

    assert!(expect_segment_source.contains("for candidate in expected"));
    assert!(expect_segment_source.contains("if *candidate == actual"));
    assert!(expect_segment_source.contains("return Ok(());"));
    assert!(!expect_segment_source.contains("expected.iter().any("));
}

#[test]
fn world_transform_rotation_validation_sums_quaternion_length_directly() {
    let value_conversion_source = include_str!("../../world/property_access/value_conversion.rs");
    let validate_quat_source = value_conversion_source
        .split("pub(super) fn validate_quat_array(")
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
    let value_conversion_source = include_str!("../../world/property_access/value_conversion.rs");
    let validate_finite_array_source = value_conversion_source
        .split("fn validate_finite_array(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn expect_resource_id").next())
        .expect("read validate_finite_array helper body");

    assert!(validate_finite_array_source.contains("for component in value"));
    assert!(validate_finite_array_source.contains("if !component.is_finite()"));
    assert!(validate_finite_array_source.contains("return Err(format!("));
    assert!(validate_finite_array_source.contains("Ok(())"));
    assert!(!validate_finite_array_source.contains(".iter().all("));
}

#[test]
fn world_property_enum_parsers_match_normalized_values_without_allocation() {
    let value_conversion_source = include_str!("../../world/property_access/value_conversion.rs");
    let parse_mobility_source = value_conversion_source
        .split("pub(super) fn parse_mobility")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn parse_rigid_body_type").next())
        .expect("read parse_mobility helper body");
    let parse_rigid_body_source = value_conversion_source
        .split("pub(super) fn parse_rigid_body_type")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn parse_joint_kind").next())
        .expect("read parse_rigid_body_type helper body");
    let parse_joint_source = value_conversion_source
        .split("pub(super) fn parse_joint_kind")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn parse_combine_rule").next())
        .expect("read parse_joint_kind helper body");
    let parse_combine_source = value_conversion_source
        .split("pub(super) fn parse_combine_rule")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn combine_rule_label").next())
        .expect("read parse_combine_rule helper body");

    assert!(parse_mobility_source.contains("normalized_identifier_matches(value, \"dynamic\")"));
    assert!(parse_mobility_source.contains("normalized_identifier_matches(value, \"static\")"));
    assert!(parse_rigid_body_source.contains("normalized_identifier_matches(value, \"kinematic\")"));
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
    let value_conversion_source = include_str!("../../world/property_access/value_conversion.rs");
    let normalized_identifier_source = value_conversion_source
        .split("pub(super) fn normalized_identifier(value: &str) -> String")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn normalized_identifier_matches")
                .next()
        })
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
    let value_conversion_source = include_str!("../../world/property_access/value_conversion.rs");
    let expect_i32_source = value_conversion_source
        .split("pub(super) fn expect_i32")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn expect_vec3").next())
        .expect("read expect_i32 body");
    let resource_id_source = value_conversion_source
        .split("pub(super) fn expect_resource_id")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn expect_animation_parameter")
                .next()
        })
        .expect("read expect_resource_id body");

    assert!(expect_i32_source
        .contains("ScenePropertyValue::Integer(value) => match i32::try_from(value)"));
    assert!(expect_i32_source
        .contains("ScenePropertyValue::Unsigned(value) => match i32::try_from(value)"));
    assert!(expect_i32_source.contains("Ok(value) => Ok(value)"));
    assert!(expect_i32_source
        .contains("Err(_) => Err(format!(\"property `{property_path}` expected i32 integer\"))"));
    assert!(resource_id_source.contains("match ResourceId::from_str(&value)"));
    assert!(resource_id_source.contains("Ok(resource_id) => Ok(resource_id)"));
    assert!(resource_id_source
        .contains("\"property `{property_path}` has invalid resource id: {error}\""));
    assert!(!expect_i32_source.contains(".map_err("));
    assert!(!resource_id_source.contains(".map_err("));
}
