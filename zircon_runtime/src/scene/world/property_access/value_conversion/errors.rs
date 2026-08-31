use crate::core::framework::scene::ComponentPropertyPath;
use crate::scene::{EntityId, SceneError, SceneResult};

pub(in crate::scene::world::property_access) fn expect_segment_count(
    segments: &[String],
    expected: usize,
    property_path: &ComponentPropertyPath,
) -> SceneResult<()> {
    if segments.len() == expected {
        Ok(())
    } else {
        Err(SceneError::PropertySegmentCount {
            property_path: property_path.to_string(),
            expected,
            actual: segments.len(),
        })
    }
}

pub(in crate::scene::world::property_access) fn expect_segment(
    actual: &str,
    expected: &[&str],
    property_path: &ComponentPropertyPath,
) -> SceneResult<()> {
    for candidate in expected {
        if *candidate == actual {
            return Ok(());
        }
    }

    unknown_property(property_path)
}

fn unknown_property<T>(property_path: &ComponentPropertyPath) -> SceneResult<T> {
    Err(SceneError::UnknownProperty {
        property_path: property_path.to_string(),
    })
}

pub(in crate::scene::world::property_access) fn unknown_property_error(
    property_path: &ComponentPropertyPath,
) -> SceneResult<bool> {
    unknown_property(property_path)
}

pub(in crate::scene::world::property_access) fn missing_component_error(
    entity: EntityId,
    property_path: &ComponentPropertyPath,
) -> SceneResult<bool> {
    Err(SceneError::MissingPropertyComponent {
        entity,
        property_path: property_path.to_string(),
    })
}

pub(in crate::scene::world::property_access) fn property_type_error<T>(
    property_path: &ComponentPropertyPath,
    expected: &str,
) -> SceneResult<T> {
    Err(SceneError::PropertyTypeMismatch {
        property_path: property_path.to_string(),
        expected: format!("value of type {expected}"),
    })
}
