//! Resolves the geometry actually bound to a World mesh renderer for a navigation bake.
//!
//! Navigation has no camera-distance input, so the renderer's base source is the stable bake
//! source. This matches the render-scene selector's base fallback for an invalid/unspecified
//! distance. Geometry is copied while typed asset leases are held; no procedural stand-in is
//! emitted when an explicit model or mesh handle is unresolved.

use zircon_runtime::asset::{MeshAsset, ModelPrimitiveAsset, ProjectAssetManager};
use zircon_runtime::core::framework::navigation::{NavigationError, NavigationErrorKind};
use zircon_runtime::core::framework::render::RenderMeshTopology;
use zircon_runtime::core::math::{is_finite_vec3, Mat4, Vec3};
use zircon_runtime::core::resource::ResourceId;
use zircon_runtime::scene::components::{NodeKind, SceneNode};

use super::geometry::BakeGeometry;

pub(super) fn collect_render_node_geometry(
    asset_manager: &ProjectAssetManager,
    node: &SceneNode,
    matrix: Mat4,
    geometry: &mut BakeGeometry,
    area: u8,
) -> Result<(), NavigationError> {
    let Some(renderer) = node.mesh.as_ref() else {
        if matches!(&node.kind, NodeKind::Cube | NodeKind::Mesh) {
            geometry.unbound_render_nodes += 1;
        }
        return Ok(());
    };

    // Render-scene extraction emits primitive bindings as the authoritative mesh list and
    // returns before considering the renderer's direct mesh or model fallback.
    if !renderer.primitives.is_empty() {
        for binding in &renderer.primitives {
            let id = binding.mesh.id();
            let lease = asset_manager
                .acquire_mesh_asset(id)
                .map_err(|error| unresolved_mesh(id, error.to_string()))?;
            let primitive = primitive_from_mesh(&lease, id)?;
            append_primitive(&primitive, matrix, geometry, area, node.id)?;
        }
        if !renderer.lods.is_empty() {
            geometry.render_lod_levels_not_selected += renderer.lods.len();
        }
        return Ok(());
    }

    // A direct mesh is the renderer's primary source when no primitive bindings are present.
    if let Some(mesh) = renderer.mesh.as_ref() {
        let lease = asset_manager
            .acquire_mesh_asset(mesh.id())
            .map_err(|error| unresolved_mesh(mesh.id(), error.to_string()))?;
        let primitive = primitive_from_mesh(&lease, mesh.id())?;
        append_primitive(&primitive, matrix, geometry, area, node.id)?;
        return Ok(());
    }

    let model_id = renderer.model.id();
    let model = asset_manager
        .acquire_model_asset(model_id)
        .map_err(|error| unresolved_model(model_id, error.to_string()))?;
    for embedded in &model.primitives {
        let primitive = resolve_embedded_mesh(asset_manager, embedded)?;
        append_primitive(&primitive, matrix, geometry, area, node.id)?;
    }

    if !renderer.lods.is_empty() {
        geometry.render_lod_levels_not_selected += renderer.lods.len();
    }
    Ok(())
}

fn resolve_embedded_mesh(
    asset_manager: &ProjectAssetManager,
    embedded: &ModelPrimitiveAsset,
) -> Result<ModelPrimitiveAsset, NavigationError> {
    let Some(reference) = embedded.mesh.as_ref() else {
        return Ok(embedded.clone());
    };
    let Some(id) = asset_manager.resolve_asset_id(&reference.locator) else {
        return Err(navigation_error(format!(
            "render model primitive mesh {} could not be resolved",
            reference.locator
        )));
    };
    let mesh = asset_manager
        .acquire_mesh_asset(id)
        .map_err(|error| unresolved_mesh(id, error.to_string()))?;
    primitive_from_mesh(&mesh, id)
}

fn primitive_from_mesh(
    mesh: &MeshAsset,
    id: ResourceId,
) -> Result<ModelPrimitiveAsset, NavigationError> {
    validate_triangle_mesh_topology(mesh.topology, id)?;
    mesh.to_model_primitive()
        .map_err(|error| invalid_mesh(id, format!("mesh payload cannot become triangles: {error}")))
}

fn append_primitive(
    primitive: &ModelPrimitiveAsset,
    matrix: Mat4,
    geometry: &mut BakeGeometry,
    area: u8,
    entity: u64,
) -> Result<(), NavigationError> {
    if primitive.indices.is_empty() {
        return Ok(());
    }
    if primitive.indices.len() % 3 != 0 {
        return Err(navigation_error(format!(
            "entity {entity} has a model primitive index count not divisible by three"
        )));
    }
    let base = u32::try_from(geometry.vertices.len()).map_err(|_| {
        navigation_error(format!(
            "entity {entity} render geometry exceeds the Recast vertex index range"
        ))
    })?;
    let vertex_count = primitive.vertices.len();
    if primitive
        .indices
        .iter()
        .any(|index| *index as usize >= vertex_count)
    {
        return Err(navigation_error(format!(
            "entity {entity} render geometry contains an index outside its vertex range"
        )));
    }

    let mut transformed = Vec::with_capacity(vertex_count);
    for vertex in &primitive.vertices {
        let position = matrix.transform_point3(Vec3::from_array(vertex.position));
        if !is_finite_vec3(position) {
            return Err(navigation_error(format!(
                "entity {entity} render geometry contains a non-finite transformed vertex"
            )));
        }
        transformed.push(position.to_array());
    }
    let mut indices = Vec::with_capacity(primitive.indices.len());
    for index in &primitive.indices {
        indices.push(base.checked_add(*index).ok_or_else(|| {
            navigation_error(format!(
                "entity {entity} render geometry exceeds the Recast vertex index range"
            ))
        })?);
    }

    geometry.vertices.extend(transformed);
    geometry.indices.extend(indices);
    geometry
        .triangle_areas
        .extend(std::iter::repeat(area).take(primitive.indices.len() / 3));
    Ok(())
}

pub(super) fn validate_triangle_mesh_topology(
    topology: RenderMeshTopology,
    id: ResourceId,
) -> Result<(), NavigationError> {
    if topology == RenderMeshTopology::TriangleList {
        return Ok(());
    }
    Err(invalid_mesh(
        id,
        format!("navigation requires triangle-list geometry; found {topology:?}"),
    ))
}

fn unresolved_model(id: ResourceId, reason: String) -> NavigationError {
    navigation_error(format!("render model {id} could not be resolved: {reason}"))
}

fn unresolved_mesh(id: ResourceId, reason: String) -> NavigationError {
    navigation_error(format!("render mesh {id} could not be resolved: {reason}"))
}

fn invalid_mesh(id: ResourceId, reason: String) -> NavigationError {
    navigation_error(format!("render geometry {id} is invalid: {reason}"))
}

fn navigation_error(message: String) -> NavigationError {
    NavigationError::new(NavigationErrorKind::InvalidConfiguration, message)
}
