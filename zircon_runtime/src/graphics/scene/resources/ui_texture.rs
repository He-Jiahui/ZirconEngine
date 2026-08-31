use std::collections::HashSet;
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    RenderImageDescriptor, RenderImageDimension, UiRenderSubmission,
};
use crate::core::resource::{ResourceId, ResourceLocator, ResourceScheme};
use crate::text::{RichTextDependency, resolve_compiled_rich_text_artifact};
use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

use super::{GpuTextureResource, ResourceStreamer};

mod prepare_receipt;

pub(in crate::graphics::scene) use prepare_receipt::UiTexturePrepareReceipt;
use prepare_receipt::{UiTexturePrepareOutcome, UiTexturePrepareRow, resolve_ui_texture_candidate};

#[derive(Debug)]
pub(in crate::graphics::scene::resources) struct UiTextureDependencies {
    ids: Vec<ResourceId>,
}

impl UiTextureDependencies {
    fn as_slice(&self) -> &[ResourceId] {
        &self.ids
    }
}

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
    submission: &UiRenderSubmission,
) -> UiTextureDependencies {
    let mut ids = HashSet::new();
    for command in submission.commands() {
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
            ids.extend(
                rich.dependencies()
                    .iter()
                    .map(|dependency| match dependency {
                        RichTextDependency::ImageTexture(texture) => *texture,
                        RichTextDependency::IconAsset(asset) => asset.resource_id(),
                    }),
            );
        }
    }
    let mut ids = ids.into_iter().collect::<Vec<_>>();
    ids.sort_unstable();
    UiTextureDependencies { ids }
}

pub(crate) fn resolve_ui_texture_id(
    asset_manager: &ProjectAssetManager,
    requested: ResourceId,
) -> ResourceId {
    let generation = asset_manager
        .resource_manager()
        .projection_snapshot()
        .management()
        .clone();
    resolve_ui_texture_candidate(&generation, requested)
        .map(|row| row.id)
        .unwrap_or(requested)
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

    pub(in crate::graphics::scene) fn last_ui_texture_prepare_receipt(
        &self,
    ) -> Option<&UiTexturePrepareReceipt> {
        self.last_ui_texture_prepare_receipt.as_ref()
    }

    pub(in crate::graphics::scene) fn prepared_ui_texture_id(
        &self,
        requested: ResourceId,
    ) -> Option<ResourceId> {
        let (resolved, expected_revision) = self
            .last_ui_texture_prepare_receipt()?
            .ready_texture_binding(requested)?;
        self.texture_with_revision(resolved)
            .filter(|(prepared_revision, texture)| {
                *prepared_revision == expected_revision
                    && is_ui_texture_descriptor(&texture.descriptor)
            })
            .map(|_| resolved)
    }
}

fn is_ui_texture_descriptor(descriptor: &RenderImageDescriptor) -> bool {
    descriptor.dimension == RenderImageDimension::D2 && descriptor.depth_or_array_layers == 1
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{
        UiTexturePrepareOutcome, UiTexturePrepareReceipt, UiTexturePrepareRow,
        resolve_ui_texture_candidate, resolve_ui_texture_id, ui_image_resource_id, ui_texture_ids,
    };
    use crate::asset::ProjectAssetManager;
    use crate::core::framework::render::UiRenderSubmission;
    use crate::core::resource::{
        AssetUuid, ResourceId, ResourceKind, ResourceLocator, ResourceRecord,
    };
    use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
    use zircon_runtime_interface::ui::layout::UiFrame;
    use zircon_runtime_interface::ui::surface::{
        UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
        UiVisualAssetRef,
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
    fn ui_texture_discovery_walks_all_submission_segments() {
        let first_id = ui_image_resource_id("res://ui/first.png").unwrap();
        let second_id = ui_image_resource_id("res://ui/second.png").unwrap();
        let submission = UiRenderSubmission::from_segments(vec![
            image_extract("first", &["res://ui/first.png"]),
            image_extract("second", &["res://ui/second.png", "res://ui/first.png"]),
        ]);

        let ids = ui_texture_ids(submission.as_ref());

        let mut expected = vec![first_id, second_id];
        expected.sort_unstable();
        assert_eq!(ids.as_slice(), expected);
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

    #[test]
    fn ui_texture_candidate_rejects_unresolved_and_wrong_kind_resources() {
        let manager = ProjectAssetManager::default();
        let missing = ResourceId::from_stable_label("missing-ui-texture");
        let projection = manager.resource_manager().projection_snapshot();
        assert_eq!(
            resolve_ui_texture_candidate(projection.management(), missing),
            Err(UiTexturePrepareOutcome::UnresolvedIdentity)
        );

        let locator = ResourceLocator::parse("res://ui/not-a-texture.asset").unwrap();
        let wrong_kind = ResourceId::from_locator(&locator);
        manager
            .resource_manager()
            .register_record(ResourceRecord::new(
                wrong_kind,
                ResourceKind::Material,
                locator,
            ))
            .unwrap();
        let projection = manager.resource_manager().projection_snapshot();
        assert_eq!(
            resolve_ui_texture_candidate(projection.management(), wrong_kind),
            Err(UiTexturePrepareOutcome::InvalidResourceKind)
        );
    }

    #[test]
    fn ui_texture_receipt_only_exposes_exact_ready_rows_to_binding() {
        let manager = ProjectAssetManager::default();
        let projection = manager.resource_manager().projection_snapshot();
        let ready = ResourceId::from_stable_label("ready-ui-texture");
        let failed = ResourceId::from_stable_label("failed-ui-texture");
        let unqualified = ResourceId::from_stable_label("unqualified-ui-texture");
        let receipt = UiTexturePrepareReceipt::new(
            1,
            projection.management_identity(),
            projection.readiness_identity(),
            vec![
                UiTexturePrepareRow {
                    requested: ready,
                    resolved: Some(ready),
                    outcome: UiTexturePrepareOutcome::Ready,
                    prepared_revision: Some(7),
                },
                UiTexturePrepareRow {
                    requested: failed,
                    resolved: Some(failed),
                    outcome: UiTexturePrepareOutcome::UploadFailed,
                    prepared_revision: None,
                },
                UiTexturePrepareRow {
                    requested: unqualified,
                    resolved: Some(unqualified),
                    outcome: UiTexturePrepareOutcome::Ready,
                    prepared_revision: None,
                },
            ],
        );

        assert_eq!(receipt.ready_texture_id(ready), Some(ready));
        assert_eq!(receipt.ready_texture_id(failed), None);
        assert_eq!(receipt.ready_texture_id(unqualified), None);
    }

    fn image_extract(tree_id: &str, sources: &[&str]) -> Arc<UiRenderExtract> {
        Arc::new(UiRenderExtract {
            tree_id: UiTreeId::new(tree_id),
            list: UiRenderList {
                commands: sources
                    .iter()
                    .enumerate()
                    .map(|(index, source)| UiRenderCommand {
                        node_id: UiNodeId::new(index as u64 + 1),
                        kind: UiRenderCommandKind::Image,
                        frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
                        clip_frame: None,
                        z_index: index as i32,
                        style: UiResolvedStyle::default(),
                        text_layout: None,
                        text: None,
                        image: Some(UiVisualAssetRef::Image((*source).to_string())),
                        opacity: 1.0,
                    })
                    .collect(),
            },
            raster_scale: 1.0,
        })
    }
}
