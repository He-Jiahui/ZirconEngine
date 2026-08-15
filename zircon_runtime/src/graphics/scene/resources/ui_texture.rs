use std::collections::HashSet;
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{RenderImageDescriptor, RenderImageDimension};
use crate::core::resource::{ResourceId, ResourceLocator, ResourceScheme};
use crate::text::resolve_compiled_rich_text_artifact;
use zircon_runtime_interface::ui::surface::{UiRenderExtract, UiVisualAssetRef};

use super::{GpuTextureResource, ResourceStreamer};

pub(crate) fn ui_image_resource_id(source: &str) -> Option<ResourceId> {
    let locator = ResourceLocator::parse(source.trim()).ok()?;
    matches!(
        locator.scheme(),
        ResourceScheme::Res
            | ResourceScheme::Library
            | ResourceScheme::Package
            | ResourceScheme::Builtin
    )
    .then(|| ResourceId::from_locator(&locator))
}

pub(in crate::graphics::scene::resources) fn ui_texture_ids(
    extract: &UiRenderExtract,
) -> Vec<ResourceId> {
    let mut ids = HashSet::new();
    for command in &extract.list.commands {
        if let Some(UiVisualAssetRef::Image(source)) = command.image.as_ref() {
            if let Some(id) = ui_image_resource_id(source) {
                ids.insert(id);
            }
        }
        if let Some(rich) = command
            .text_layout
            .as_ref()
            .and_then(|layout| layout.rich_text_artifact.as_ref())
            .and_then(resolve_compiled_rich_text_artifact)
        {
            ids.extend(rich.resource_ids().iter().copied());
        }
    }
    let mut ids = ids.into_iter().collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

pub(crate) fn resolve_ui_texture_id(
    asset_manager: &ProjectAssetManager,
    requested: ResourceId,
) -> ResourceId {
    let resource_manager = asset_manager.resource_manager();
    let registry = resource_manager.registry();
    if registry.get(requested).is_some() {
        return requested;
    }
    let resolved = registry
        .values()
        .find(|record| ResourceId::from_locator(record.primary_locator()) == requested)
        .map(|record| record.id())
        .unwrap_or(requested);
    resolved
}

pub(in crate::graphics::scene::resources) fn ui_texture_id_for_upload(
    asset_manager: &ProjectAssetManager,
    requested: ResourceId,
) -> Option<ResourceId> {
    let resolved = resolve_ui_texture_id(asset_manager, requested);
    let asset = asset_manager.load_texture_asset(resolved).ok()?;
    let descriptor = asset.render_image_descriptor();
    is_ui_texture_descriptor(&descriptor).then_some(resolved)
}

impl ResourceStreamer {
    pub(crate) fn resolve_ui_texture_id(&self, requested: ResourceId) -> Option<ResourceId> {
        self.asset_manager()
            .ok()
            .map(|asset_manager| resolve_ui_texture_id(asset_manager.as_ref(), requested))
    }

    pub(crate) fn ui_texture_ref(
        &self,
        resolved_texture_id: Option<ResourceId>,
    ) -> &Arc<GpuTextureResource> {
        let texture = self.texture_ref(resolved_texture_id);
        if is_ui_texture_descriptor(&texture.descriptor) {
            texture
        } else {
            self.texture_ref(None)
        }
    }
}

fn is_ui_texture_descriptor(descriptor: &RenderImageDescriptor) -> bool {
    descriptor.dimension == RenderImageDimension::D2 && descriptor.depth_or_array_layers == 1
}

#[cfg(test)]
mod tests {
    use super::{resolve_ui_texture_id, ui_image_resource_id};
    use crate::asset::ProjectAssetManager;
    use crate::core::resource::{
        AssetUuid, ResourceId, ResourceKind, ResourceLocator, ResourceRecord,
    };

    #[test]
    fn ui_image_resource_id_accepts_engine_assets_and_rejects_network_urls() {
        assert_eq!(
            ui_image_resource_id("res://ui/checker.png"),
            Some(ResourceId::from_stable_label("res://ui/checker.png"))
        );
        assert_eq!(
            ui_image_resource_id("https://example.com/checker.png"),
            None
        );
    }

    #[test]
    fn ui_texture_resolution_maps_locator_identity_to_imported_asset_identity() {
        let manager = ProjectAssetManager::default();
        let locator = ResourceLocator::parse("res://ui/checker.png").unwrap();
        let requested = ResourceId::from_locator(&locator);
        let imported = ResourceId::from_asset_uuid(AssetUuid::from_stable_label("ui/checker"));
        manager
            .resource_manager()
            .register_record(ResourceRecord::new(
                imported,
                ResourceKind::Texture,
                locator,
            ))
            .unwrap();

        assert_ne!(requested, imported);
        assert_eq!(resolve_ui_texture_id(&manager, requested), imported);
    }
}
