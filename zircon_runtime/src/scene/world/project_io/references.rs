use crate::asset::AssetReference;
use crate::asset::project::ProjectManager;
use crate::core::resource::{
    MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId, ResourceLocator,
    ResourceMarker, ResourceScheme,
};
use std::sync::OnceLock;

use super::{
    BUILTIN_CUBE, BUILTIN_DEFAULT_MATERIAL, BUILTIN_MISSING_MATERIAL, BUILTIN_MISSING_MODEL,
    SceneProjectError,
};
pub(super) fn model_handle_for_reference(
    project: &ProjectManager,
    reference: &AssetReference,
) -> Result<ResourceHandle<ModelMarker>, SceneProjectError> {
    handle_for_reference(project, reference)
}

pub(super) fn material_handle_for_reference(
    project: &ProjectManager,
    reference: &AssetReference,
) -> Result<ResourceHandle<MaterialMarker>, SceneProjectError> {
    handle_for_reference(project, reference)
}

pub(super) fn handle_for_reference<T: ResourceMarker>(
    project: &ProjectManager,
    reference: &AssetReference,
) -> Result<ResourceHandle<T>, SceneProjectError> {
    let locator = &reference.locator;
    if locator.scheme() == ResourceScheme::Builtin {
        return Ok(ResourceHandle::new(ResourceId::from_locator(locator)));
    }

    project
        .asset_registry()
        .resolve_asset_id_for_reference(reference.uuid, locator)
        .map(ResourceHandle::new)
        .map_err(|_| SceneProjectError::DanglingAssetReference {
            uuid: reference.uuid,
            locator: locator.clone(),
        })
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
    if let Ok(reference) = project.asset_registry().resolve_reference_by_asset_id(id) {
        return Ok(reference);
    }
    if let Some(locator) = builtin_locator_for_id(id) {
        return Ok(AssetReference::from_locator(locator));
    }
    Err(SceneProjectError::SceneAsset(format!(
        "missing persistent locator for {label} resource {id}"
    )))
}

fn builtin_locators() -> &'static [(ResourceId, ResourceLocator)] {
    static BUILTIN_LOCATORS: OnceLock<Vec<(ResourceId, ResourceLocator)>> = OnceLock::new();
    BUILTIN_LOCATORS.get_or_init(|| {
        let mut locators = Vec::with_capacity(4);
        for locator_text in [
            BUILTIN_CUBE,
            BUILTIN_DEFAULT_MATERIAL,
            BUILTIN_MISSING_MODEL,
            BUILTIN_MISSING_MATERIAL,
        ] {
            let locator = ResourceLocator::parse(locator_text).expect("builtin locator");
            locators.push((ResourceId::from_locator(&locator), locator));
        }
        locators
    })
}

fn builtin_locator_for_id(id: ResourceId) -> Option<ResourceLocator> {
    for (candidate_id, locator) in builtin_locators() {
        if *candidate_id == id {
            return Some(locator.clone());
        }
    }
    None
}
