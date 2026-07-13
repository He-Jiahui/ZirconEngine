use std::collections::BTreeMap;

use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

use super::super::{MeshRenderer, MeshRendererPrimitiveBinding};

pub(in crate::scene::components::scene) fn read_model(
    component: &MeshRenderer,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Resource(component.model.id().to_string()))
}

pub(in crate::scene::components::scene) fn read_mesh(
    component: &MeshRenderer,
) -> Result<ReflectedValue, ReflectError> {
    Ok(component.mesh.map_or(ReflectedValue::Null, |mesh| {
        ReflectedValue::Resource(mesh.id().to_string())
    }))
}

pub(in crate::scene::components::scene) fn read_material(
    component: &MeshRenderer,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Resource(
        component.material.id().to_string(),
    ))
}

pub(in crate::scene::components::scene) fn read_morph_weights(
    component: &MeshRenderer,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::List(
        component
            .morph_weights
            .iter()
            .copied()
            .map(ReflectedValue::Scalar)
            .collect(),
    ))
}

pub(in crate::scene::components::scene) fn read_primitives(
    component: &MeshRenderer,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::List(
        component
            .primitives
            .iter()
            .map(reflected_primitive_binding)
            .collect(),
    ))
}

pub(in crate::scene::components::scene) fn read_lods(
    component: &MeshRenderer,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::List(
        component
            .lods
            .iter()
            .map(|lod| {
                ReflectedValue::Map(BTreeMap::from([
                    (
                        "min_distance".to_string(),
                        ReflectedValue::Scalar(lod.min_distance),
                    ),
                    (
                        "model".to_string(),
                        ReflectedValue::Resource(lod.model.id().to_string()),
                    ),
                    (
                        "mesh".to_string(),
                        lod.mesh.map_or(ReflectedValue::Null, |mesh| {
                            ReflectedValue::Resource(mesh.id().to_string())
                        }),
                    ),
                    (
                        "material".to_string(),
                        ReflectedValue::Resource(lod.material.id().to_string()),
                    ),
                    (
                        "primitives".to_string(),
                        ReflectedValue::List(
                            lod.primitives
                                .iter()
                                .map(reflected_primitive_binding)
                                .collect(),
                        ),
                    ),
                ]))
            })
            .collect(),
    ))
}

fn reflected_primitive_binding(primitive: &MeshRendererPrimitiveBinding) -> ReflectedValue {
    ReflectedValue::Map(BTreeMap::from([
        (
            "mesh".to_string(),
            ReflectedValue::Resource(primitive.mesh.id().to_string()),
        ),
        (
            "material".to_string(),
            ReflectedValue::Resource(primitive.material.id().to_string()),
        ),
    ]))
}
