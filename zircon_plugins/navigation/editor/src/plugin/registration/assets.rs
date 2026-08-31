use zircon_editor::core::asset::{AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId};
use zircon_editor::core::commands::EditorCommandDescriptor;
use zircon_editor::core::editor_event::{EditorAssetEvent, EditorEvent};
use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError,
};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_runtime_interface::resource::ResourceKind;

use crate::extension_ids::{
    NAVIGATION_ASSET_VIEW_ID, NAVIGATION_OPEN_NAVMESH_ASSET_OPERATION,
    NAVIGATION_OPEN_SETTINGS_ASSET_OPERATION, NAVIGATION_SETTINGS_ASSET_VIEW_ID,
};

pub(super) fn register(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    register_asset_type(
        registry,
        NAVIGATION_OPEN_NAVMESH_ASSET_OPERATION,
        ResourceKind::NavMesh,
        NAVIGATION_ASSET_VIEW_ID,
    )?;
    register_asset_type(
        registry,
        NAVIGATION_OPEN_SETTINGS_ASSET_OPERATION,
        ResourceKind::NavigationSettings,
        NAVIGATION_SETTINGS_ASSET_VIEW_ID,
    )
}

fn register_asset_type(
    registry: &mut EditorExtensionRegistry,
    operation_path: &str,
    resource_kind: ResourceKind,
    view_id: &str,
) -> Result<(), EditorExtensionRegistryError> {
    let operation = EditorOperationPath::parse(operation_path)
        .map_err(EditorExtensionRegistryError::OperationPath)?;
    registry.register_command(
        EditorCommandDescriptor::operation(operation.clone())
            .with_callable_from_remote(false)
            .with_event(EditorEvent::Asset(EditorAssetEvent::OpenAssetBrowser)),
    )?;
    registry.register_asset_type_contribution(
        AssetTypeContribution::augment(AssetTypeId::from_resource_kind(resource_kind))
            .with_toolkit(AssetToolkitDescriptor::new(view_id, operation)),
    )
}
