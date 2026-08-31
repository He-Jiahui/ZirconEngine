use std::fmt::Write as _;

use crate::core::framework::scene::ScenePropertyValue;
use crate::scene::components::ColliderShape;

use super::super::super::value_conversion::normalized_identifier_matches;

pub(super) fn collider_shape_property_value(
    shape: &ColliderShape,
    segments: &[String],
) -> Option<ScenePropertyValue> {
    match shape {
        ColliderShape::Box { half_extents } => match segments {
            [field] if normalized_identifier_matches(field, "kind") => {
                Some(ScenePropertyValue::Enum("box".to_string()))
            }
            [field] if normalized_identifier_matches(field, "half_extents") => {
                Some(ScenePropertyValue::Vec3(half_extents.to_array()))
            }
            _ => None,
        },
        ColliderShape::Sphere { radius } => match segments {
            [field] if normalized_identifier_matches(field, "kind") => {
                Some(ScenePropertyValue::Enum("sphere".to_string()))
            }
            [field] if normalized_identifier_matches(field, "radius") => {
                Some(ScenePropertyValue::Scalar(*radius))
            }
            _ => None,
        },
        ColliderShape::Capsule {
            radius,
            half_height,
        } => match segments {
            [field] if normalized_identifier_matches(field, "kind") => {
                Some(ScenePropertyValue::Enum("capsule".to_string()))
            }
            [field] if normalized_identifier_matches(field, "radius") => {
                Some(ScenePropertyValue::Scalar(*radius))
            }
            [field] if normalized_identifier_matches(field, "half_height") => {
                Some(ScenePropertyValue::Scalar(*half_height))
            }
            _ => None,
        },
        ColliderShape::Cylinder {
            radius,
            half_height,
        } => match segments {
            [field] if normalized_identifier_matches(field, "kind") => {
                Some(ScenePropertyValue::Enum("cylinder".to_string()))
            }
            [field] if normalized_identifier_matches(field, "radius") => {
                Some(ScenePropertyValue::Scalar(*radius))
            }
            [field] if normalized_identifier_matches(field, "half_height") => {
                Some(ScenePropertyValue::Scalar(*half_height))
            }
            _ => None,
        },
        ColliderShape::ConvexHull { points } => match segments {
            [field] if normalized_identifier_matches(field, "kind") => {
                Some(ScenePropertyValue::Enum("convex_hull".to_string()))
            }
            [field] if normalized_identifier_matches(field, "point_count") => {
                Some(ScenePropertyValue::Unsigned(points.len() as u64))
            }
            [group, index] if normalized_identifier_matches(group, "points") => points
                .get(index.parse::<usize>().ok()?)
                .map(|point| ScenePropertyValue::Vec3(point.to_array())),
            _ => None,
        },
        ColliderShape::TriangleMesh { mesh } => match segments {
            [field] if normalized_identifier_matches(field, "kind") => {
                Some(ScenePropertyValue::Enum("triangle_mesh".to_string()))
            }
            [field] if normalized_identifier_matches(field, "mesh") => {
                Some(ScenePropertyValue::Resource(mesh.uuid.to_string()))
            }
            _ => None,
        },
        ColliderShape::HeightField {
            resolution,
            heights,
        } => match segments {
            [field] if normalized_identifier_matches(field, "kind") => {
                Some(ScenePropertyValue::Enum("height_field".to_string()))
            }
            [group, axis]
                if normalized_identifier_matches(group, "resolution")
                    && normalized_identifier_matches(axis, "x") =>
            {
                Some(ScenePropertyValue::Unsigned(u64::from(resolution[0])))
            }
            [group, axis]
                if normalized_identifier_matches(group, "resolution")
                    && normalized_identifier_matches(axis, "y") =>
            {
                Some(ScenePropertyValue::Unsigned(u64::from(resolution[1])))
            }
            [field] if normalized_identifier_matches(field, "heights") => {
                Some(ScenePropertyValue::Resource(heights.uuid.to_string()))
            }
            _ => None,
        },
        ColliderShape::Compound { children } => match segments {
            [field] if normalized_identifier_matches(field, "kind") => {
                Some(ScenePropertyValue::Enum("compound".to_string()))
            }
            [field] if normalized_identifier_matches(field, "child_count") => {
                Some(ScenePropertyValue::Unsigned(children.len() as u64))
            }
            [group, index, transform, field]
                if normalized_identifier_matches(group, "children")
                    && normalized_identifier_matches(transform, "transform") =>
            {
                let (child_transform, _) = children.get(index.parse::<usize>().ok()?)?;
                if normalized_identifier_matches(field, "translation") {
                    Some(ScenePropertyValue::Vec3(
                        child_transform.translation.to_array(),
                    ))
                } else if normalized_identifier_matches(field, "rotation") {
                    Some(ScenePropertyValue::Quaternion(
                        child_transform.rotation.to_array(),
                    ))
                } else if normalized_identifier_matches(field, "scale") {
                    Some(ScenePropertyValue::Vec3(child_transform.scale.to_array()))
                } else {
                    None
                }
            }
            [group, index, shape, remaining @ ..]
                if normalized_identifier_matches(group, "children")
                    && normalized_identifier_matches(shape, "shape") =>
            {
                let (_, child_shape) = children.get(index.parse::<usize>().ok()?)?;
                collider_shape_property_value(child_shape.as_ref(), remaining)
            }
            _ => None,
        },
    }
}

pub(super) fn visit_collider_shape_property_entries<F>(
    shape: &ColliderShape,
    prefix: &str,
    visitor: &mut F,
) -> bool
where
    F: FnMut(&str, &mut dyn FnMut() -> ScenePropertyValue, bool) -> bool,
{
    let mut path = String::with_capacity(
        prefix
            .len()
            .saturating_add(collider_shape_property_path_suffix_capacity(shape)),
    );
    path.push_str(prefix);
    visit_collider_shape_property_entries_with_path(shape, &mut path, visitor)
}

fn visit_collider_shape_property_entries_with_path<F>(
    shape: &ColliderShape,
    path: &mut String,
    visitor: &mut F,
) -> bool
where
    F: FnMut(&str, &mut dyn FnMut() -> ScenePropertyValue, bool) -> bool,
{
    macro_rules! push_shape_entry {
        ($suffix:expr, $value:expr, $animatable:expr $(,)?) => {{
            let checkpoint = path.len();
            path.push('.');
            path.push_str($suffix);
            let mut build_value = || $value;
            let keep_visiting = visitor(path.as_str(), &mut build_value, $animatable);
            path.truncate(checkpoint);
            if !keep_visiting {
                return false;
            }
        }};
    }

    match shape {
        ColliderShape::Box { half_extents } => {
            push_shape_entry!("kind", ScenePropertyValue::Enum("box".to_string()), false);
            push_shape_entry!(
                "half_extents",
                ScenePropertyValue::Vec3(half_extents.to_array()),
                true,
            );
        }
        ColliderShape::Sphere { radius } => {
            push_shape_entry!(
                "kind",
                ScenePropertyValue::Enum("sphere".to_string()),
                false,
            );
            push_shape_entry!("radius", ScenePropertyValue::Scalar(*radius), true);
        }
        ColliderShape::Capsule {
            radius,
            half_height,
        } => {
            push_shape_entry!(
                "kind",
                ScenePropertyValue::Enum("capsule".to_string()),
                false,
            );
            push_shape_entry!("radius", ScenePropertyValue::Scalar(*radius), true);
            push_shape_entry!(
                "half_height",
                ScenePropertyValue::Scalar(*half_height),
                true,
            );
        }
        ColliderShape::Cylinder {
            radius,
            half_height,
        } => {
            push_shape_entry!(
                "kind",
                ScenePropertyValue::Enum("cylinder".to_string()),
                false,
            );
            push_shape_entry!("radius", ScenePropertyValue::Scalar(*radius), true);
            push_shape_entry!(
                "half_height",
                ScenePropertyValue::Scalar(*half_height),
                true,
            );
        }
        ColliderShape::ConvexHull { points } => {
            push_shape_entry!(
                "kind",
                ScenePropertyValue::Enum("convex_hull".to_string()),
                false,
            );
            push_shape_entry!(
                "point_count",
                ScenePropertyValue::Unsigned(points.len() as u64),
                false,
            );
            for (index, point) in points.iter().enumerate() {
                let checkpoint = path.len();
                write!(path, ".points.{index}").expect("writing to a String cannot fail");
                let mut build_value = || ScenePropertyValue::Vec3(point.to_array());
                let keep_visiting = visitor(path.as_str(), &mut build_value, false);
                path.truncate(checkpoint);
                if !keep_visiting {
                    return false;
                }
            }
        }
        ColliderShape::TriangleMesh { mesh } => {
            push_shape_entry!(
                "kind",
                ScenePropertyValue::Enum("triangle_mesh".to_string()),
                false,
            );
            push_shape_entry!(
                "mesh",
                ScenePropertyValue::Resource(mesh.uuid.to_string()),
                false,
            );
        }
        ColliderShape::HeightField {
            resolution,
            heights,
        } => {
            push_shape_entry!(
                "kind",
                ScenePropertyValue::Enum("height_field".to_string()),
                false,
            );
            push_shape_entry!(
                "resolution.x",
                ScenePropertyValue::Unsigned(u64::from(resolution[0])),
                false,
            );
            push_shape_entry!(
                "resolution.y",
                ScenePropertyValue::Unsigned(u64::from(resolution[1])),
                false,
            );
            push_shape_entry!(
                "heights",
                ScenePropertyValue::Resource(heights.uuid.to_string()),
                false,
            );
        }
        ColliderShape::Compound { children } => {
            push_shape_entry!(
                "kind",
                ScenePropertyValue::Enum("compound".to_string()),
                false,
            );
            push_shape_entry!(
                "child_count",
                ScenePropertyValue::Unsigned(children.len() as u64),
                false,
            );
            for (index, (transform, child_shape)) in children.iter().enumerate() {
                let checkpoint = path.len();
                write!(path, ".children.{index}").expect("writing to a String cannot fail");
                push_shape_entry!(
                    "transform.translation",
                    ScenePropertyValue::Vec3(transform.translation.to_array()),
                    false,
                );
                push_shape_entry!(
                    "transform.rotation",
                    ScenePropertyValue::Quaternion(transform.rotation.to_array()),
                    false,
                );
                push_shape_entry!(
                    "transform.scale",
                    ScenePropertyValue::Vec3(transform.scale.to_array()),
                    false,
                );
                path.push_str(".shape");
                let keep_visiting = visit_collider_shape_property_entries_with_path(
                    child_shape.as_ref(),
                    path,
                    visitor,
                );
                path.truncate(checkpoint);
                if !keep_visiting {
                    return false;
                }
            }
        }
    }

    true
}

fn collider_shape_property_path_suffix_capacity(shape: &ColliderShape) -> usize {
    match shape {
        ColliderShape::Box { .. } => ".half_extents".len(),
        ColliderShape::Sphere { .. } => ".radius".len(),
        ColliderShape::Capsule { .. } | ColliderShape::Cylinder { .. } => ".half_height".len(),
        ColliderShape::ConvexHull { points } => ".point_count"
            .len()
            .max(".points.".len() + decimal_digit_count(points.len().saturating_sub(1))),
        ColliderShape::TriangleMesh { .. } => ".mesh".len(),
        ColliderShape::HeightField { .. } => ".resolution.x".len(),
        ColliderShape::Compound { children } => {
            let indexed_child_prefix =
                ".children.".len() + decimal_digit_count(children.len().saturating_sub(1));
            let child_suffix = children
                .iter()
                .map(|(_, child_shape)| {
                    ".transform.translation".len().max(
                        ".shape".len()
                            + collider_shape_property_path_suffix_capacity(child_shape.as_ref()),
                    )
                })
                .max()
                .unwrap_or_default();
            ".child_count"
                .len()
                .max(indexed_child_prefix.saturating_add(child_suffix))
        }
    }
}

fn decimal_digit_count(mut value: usize) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

pub(super) fn collider_shape_property_entry_capacity(shape: &ColliderShape) -> usize {
    match shape {
        ColliderShape::Box { .. }
        | ColliderShape::Sphere { .. }
        | ColliderShape::TriangleMesh { .. } => 2,
        ColliderShape::Capsule { .. } | ColliderShape::Cylinder { .. } => 3,
        ColliderShape::ConvexHull { points } => 2 + points.len(),
        ColliderShape::HeightField { .. } => 4,
        ColliderShape::Compound { children } => {
            2 + children
                .iter()
                .map(|(_, child_shape)| {
                    3 + collider_shape_property_entry_capacity(child_shape.as_ref())
                })
                .sum::<usize>()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::math::{Transform, Vec3};
    use crate::core::resource::{AssetReference, ResourceLocator};

    use super::*;

    #[test]
    fn extended_collider_shape_projection_matches_capacity_and_preserves_shape_data() {
        let mesh = AssetReference::from_locator(
            ResourceLocator::parse("res://physics/property_projection.physics_mesh").unwrap(),
        );
        let shapes = [
            ColliderShape::Cylinder {
                radius: 0.75,
                half_height: 1.25,
            },
            ColliderShape::ConvexHull {
                points: vec![Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z],
            },
            ColliderShape::TriangleMesh { mesh: mesh.clone() },
            ColliderShape::HeightField {
                resolution: [8, 4],
                heights: mesh,
            },
            ColliderShape::Compound {
                children: vec![(
                    Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
                    Box::new(ColliderShape::Sphere { radius: 0.5 }),
                )],
            },
        ];

        for shape in shapes {
            let mut entries = Vec::new();
            assert!(visit_collider_shape_property_entries(
                &shape,
                "Collider.shape",
                &mut |path, value, animatable| {
                    entries.push((path.to_string(), value(), animatable));
                    true
                },
            ));
            assert_eq!(
                entries.len(),
                collider_shape_property_entry_capacity(&shape)
            );
            assert!(
                entries
                    .iter()
                    .any(|(path, _, _)| path == "Collider.shape.kind")
            );
        }
    }

    #[test]
    fn compound_projection_keeps_child_transform_and_shape_fields() {
        let compound = ColliderShape::Compound {
            children: vec![(
                Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
                Box::new(ColliderShape::Sphere { radius: 0.5 }),
            )],
        };
        let mut entries = Vec::new();
        assert!(visit_collider_shape_property_entries(
            &compound,
            "Collider.shape",
            &mut |path, value, _| {
                entries.push((path.to_string(), value()));
                true
            },
        ));
        assert!(entries.iter().any(|(path, value)| {
            path == "Collider.shape.children.0.transform.translation"
                && *value == ScenePropertyValue::Vec3([1.0, 2.0, 3.0])
        }));
        assert!(entries.iter().any(|(path, value)| {
            path == "Collider.shape.children.0.shape.radius"
                && *value == ScenePropertyValue::Scalar(0.5)
        }));
    }
}

#[cfg(test)]
#[path = "collider_shape/reused_path_buffer_tests.rs"]
mod reused_path_buffer_tests;
