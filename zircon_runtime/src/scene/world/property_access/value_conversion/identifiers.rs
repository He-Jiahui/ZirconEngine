use crate::core::framework::scene::ComponentPropertyPath;
use crate::scene::{SceneError, SceneResult, World};

pub(in crate::scene::world::property_access) fn normalized_identifier(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
        }
    }
    normalized
}

impl World {
    /// Canonicalizes a component-property DTO once before Runtime interns it.
    pub(in crate::scene::world) fn canonical_component_field_key(
        property_path: &ComponentPropertyPath,
    ) -> String {
        let Some(component) = canonical_runtime_property_component(property_path.component())
        else {
            // Dynamic type identifiers are schema keys and can be case-sensitive.
            // Keep them exact so different runtime schemas never intern to one field ID.
            return property_path.as_str().to_string();
        };
        let segments = property_path.property_segments();
        let mut key = String::with_capacity(
            component.len() + segments.iter().map(String::len).sum::<usize>() + segments.len(),
        );
        key.push_str(&component);
        for segment in segments {
            key.push('.');
            key.push_str(&normalized_identifier(segment));
        }
        key
    }

    pub(in crate::scene::world) fn is_runtime_property_component(
        property_path: &ComponentPropertyPath,
    ) -> bool {
        canonical_runtime_property_component(property_path.component()).is_some()
    }
}

fn canonical_runtime_property_component(component: &str) -> Option<&'static str> {
    match normalized_identifier(component).as_str() {
        "light" | "directionallight" => Some("directionallight"),
        "mesh" | "meshrenderer" => Some("meshrenderer"),
        "renderlayermask" | "renderlayer" => Some("renderlayer"),
        "name" => Some("name"),
        "hierarchy" => Some("hierarchy"),
        "transform" => Some("transform"),
        "camera" => Some("camera"),
        "ambientlight" => Some("ambientlight"),
        "pointlight" => Some("pointlight"),
        "rectlight" => Some("rectlight"),
        "spotlight" => Some("spotlight"),
        "rigidbody" => Some("rigidbody"),
        "collider" => Some("collider"),
        "joint" => Some("joint"),
        "animationskeleton" => Some("animationskeleton"),
        "animationplayer" => Some("animationplayer"),
        "animationsequenceplayer" => Some("animationsequenceplayer"),
        "animationgraphplayer" => Some("animationgraphplayer"),
        "animationstatemachineplayer" => Some("animationstatemachineplayer"),
        _ => None,
    }
}

pub(in crate::scene::world::property_access) fn normalized_identifier_matches(
    value: &str,
    target: &str,
) -> bool {
    let mut value_chars = value.chars();
    let mut target_chars = target.chars();

    loop {
        match (
            next_normalized_identifier_char(&mut value_chars),
            next_normalized_identifier_char(&mut target_chars),
        ) {
            (Some(value), Some(target)) if value == target => {}
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn next_normalized_identifier_char(characters: &mut impl Iterator<Item = char>) -> Option<char> {
    for character in characters {
        if character.is_ascii_alphanumeric() {
            return Some(character.to_ascii_lowercase());
        }
    }

    None
}

pub(in crate::scene::world::property_access) fn axis_index(
    axis: &str,
    property_path: &ComponentPropertyPath,
) -> SceneResult<usize> {
    match axis {
        "x" | "0" => Ok(0),
        "y" | "1" => Ok(1),
        "z" | "2" => Ok(2),
        _ => Err(SceneError::UnknownPropertyAxis {
            property_path: property_path.to_string(),
            axis_kind: "axis",
        }),
    }
}

pub(in crate::scene::world::property_access) fn quat_axis_index(
    axis: &str,
    property_path: &ComponentPropertyPath,
) -> SceneResult<usize> {
    match axis {
        "x" | "0" => Ok(0),
        "y" | "1" => Ok(1),
        "z" | "2" => Ok(2),
        "w" | "3" => Ok(3),
        _ => Err(SceneError::UnknownPropertyAxis {
            property_path: property_path.to_string(),
            axis_kind: "quaternion axis",
        }),
    }
}
