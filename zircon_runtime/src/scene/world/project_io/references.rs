use crate::asset::project::ProjectManager;
use crate::asset::AssetReference;
use crate::core::resource::{
    MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId, ResourceLocator,
    ResourceMarker, ResourceScheme,
};

use super::{
    SceneProjectError, BUILTIN_CUBE, BUILTIN_DEFAULT_MATERIAL, BUILTIN_MISSING_MATERIAL,
    BUILTIN_MISSING_MODEL,
};
pub(super) fn model_handle_for_reference(
    project: &ProjectManager,
    reference: &AssetReference,
) -> ResourceHandle<ModelMarker> {
    let locator = &reference.locator;
    if locator.scheme() == ResourceScheme::Builtin {
        return ResourceHandle::new(ResourceId::from_locator(locator));
    }

    project
        .asset_id_for_reference(reference.uuid, locator)
        .map(ResourceHandle::new)
        .unwrap_or_else(|| {
            ResourceHandle::new(ResourceId::from_stable_label(BUILTIN_MISSING_MODEL))
        })
}

pub(super) fn material_handle_for_reference(
    project: &ProjectManager,
    reference: &AssetReference,
) -> ResourceHandle<MaterialMarker> {
    let locator = &reference.locator;
    if locator.scheme() == ResourceScheme::Builtin {
        return ResourceHandle::new(ResourceId::from_locator(locator));
    }

    project
        .asset_id_for_reference(reference.uuid, locator)
        .map(ResourceHandle::new)
        .unwrap_or_else(|| {
            ResourceHandle::new(ResourceId::from_stable_label(BUILTIN_MISSING_MATERIAL))
        })
}

pub(super) fn handle_for_reference<T: ResourceMarker>(
    project: &ProjectManager,
    reference: &AssetReference,
) -> ResourceHandle<T> {
    let locator = &reference.locator;
    if locator.scheme() == ResourceScheme::Builtin {
        return ResourceHandle::new(ResourceId::from_locator(locator));
    }

    project
        .asset_id_for_reference(reference.uuid, locator)
        .map(ResourceHandle::new)
        .unwrap_or_else(|| ResourceHandle::new(ResourceId::from_locator(locator)))
}

pub(super) fn reference_for_model_handle(
    project: &ProjectManager,
    handle: ResourceHandle<ModelMarker>,
) -> Result<AssetReference, SceneProjectError> {
    reference_for_handle(project, handle.id(), "model")
}

pub(super) fn reference_for_mesh_handle(
    project: &ProjectManager,
    handle: ResourceHandle<MeshMarker>,
) -> Result<AssetReference, SceneProjectError> {
    reference_for_handle(project, handle.id(), "mesh")
}

pub(super) fn reference_for_material_handle(
    project: &ProjectManager,
    handle: ResourceHandle<MaterialMarker>,
) -> Result<AssetReference, SceneProjectError> {
    reference_for_handle(project, handle.id(), "material")
}

pub(super) fn reference_for_handle(
    project: &ProjectManager,
    id: ResourceId,
    label: &str,
) -> Result<AssetReference, SceneProjectError> {
    if let Some(reference) = project.asset_reference_for_id(id) {
        return Ok(reference);
    }
    if let Some(locator) = builtin_locator_for_id(id) {
        return Ok(AssetReference::from_locator(locator));
    }
    Err(SceneProjectError::SceneAsset(format!(
        "missing persistent locator for {label} resource {id}"
    )))
}

fn builtin_locator_for_id(id: ResourceId) -> Option<ResourceLocator> {
    for locator_text in [
        BUILTIN_CUBE,
        BUILTIN_DEFAULT_MATERIAL,
        BUILTIN_MISSING_MODEL,
        BUILTIN_MISSING_MATERIAL,
    ] {
        let locator = ResourceLocator::parse(locator_text).expect("builtin locator");
        if ResourceId::from_locator(&locator) == id {
            return Some(locator);
        }
    }
    None
}
