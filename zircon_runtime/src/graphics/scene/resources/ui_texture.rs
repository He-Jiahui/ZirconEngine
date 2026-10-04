use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Weak};

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    RenderImageDescriptor, RenderImageDimension, UiRenderSubmission,
};
use crate::core::resource::{ResourceId, ResourceLocator, ResourceScheme};
use crate::text::{resolve_compiled_rich_text_artifact, RichTextDependency};
use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiVisualAssetRef};

use super::{GpuTextureResource, ResourceStreamer};

mod prepare_receipt;

pub(in crate::graphics::scene) use prepare_receipt::UiTexturePrepareReceipt;
use prepare_receipt::{resolve_ui_texture_candidate, UiTexturePrepareOutcome, UiTexturePrepareRow};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::resources) struct UiTextureDependencyChangeJournal {
    base_generation: Option<u64>,
    added_ids: Arc<[ResourceId]>,
    removed_ids: Arc<[ResourceId]>,
    full_rebuild: bool,
}

impl UiTextureDependencyChangeJournal {
    pub(in crate::graphics::scene::resources) fn base_generation(&self) -> Option<u64> {
        self.base_generation
    }

    pub(in crate::graphics::scene::resources) fn added_ids(&self) -> &[ResourceId] {
        &self.added_ids
    }

    pub(in crate::graphics::scene::resources) fn removed_ids(&self) -> &[ResourceId] {
        &self.removed_ids
    }

    pub(in crate::graphics::scene::resources) fn is_full_rebuild(&self) -> bool {
        self.full_rebuild
    }
}

#[derive(Debug)]
pub(in crate::graphics::scene::resources) struct UiTextureDependencies {
    ids: Arc<[ResourceId]>,
    generation: u64,
    change_journal: UiTextureDependencyChangeJournal,
}

impl UiTextureDependencies {
    fn as_slice(&self) -> &[ResourceId] {
        &self.ids
    }

    pub(in crate::graphics::scene::resources) const fn generation(&self) -> u64 {
        self.generation
    }

    pub(in crate::graphics::scene::resources) fn change_journal(
        &self,
    ) -> &UiTextureDependencyChangeJournal {
        &self.change_journal
    }
}

#[derive(Default)]
pub(in crate::graphics::scene::resources) struct UiTextureDependencyCache {
    submission: Option<Weak<UiRenderSubmission>>,
    segment_entries: Vec<UiTextureDependencySegmentEntry>,
    active_ref_counts: BTreeMap<ResourceId, usize>,
    product: Option<Arc<UiTextureDependencies>>,
    next_generation: u64,
}

struct UiTextureDependencySegmentEntry {
    commands: Weak<[UiRenderCommand]>,
    ids: Arc<[ResourceId]>,
}

impl UiTextureDependencyCache {
    pub(in crate::graphics::scene::resources) fn prepare(
        &mut self,
        submission: &Arc<UiRenderSubmission>,
    ) -> Arc<UiTextureDependencies> {
        if self.submission.as_ref().is_some_and(|current| {
            current
                .upgrade()
                .is_some_and(|current| Arc::ptr_eq(&current, submission))
        }) {
            record_ui_texture_dependency_profile(0, 0, 0, self.active_ref_counts.len());
            return Arc::clone(
                self.product
                    .as_ref()
                    .expect("a retained UI submission must retain its texture dependencies"),
            );
        }

        let previous_generation = self.product.as_ref().map(|product| product.generation());
        let previous_entries = std::mem::take(&mut self.segment_entries);
        let mut previous_entries = previous_entries.into_iter();
        let mut next_entries = Vec::new();
        let mut touched_initial_presence = HashMap::new();
        let mut segment_identity_visit_count = 0_usize;
        let mut command_visit_count = 0_usize;
        let mut changed_segment_count = 0_usize;

        for commands in submission
            .segments()
            .iter()
            .flat_map(|segment| segment.extract().command_segments())
        {
            segment_identity_visit_count = segment_identity_visit_count.saturating_add(1);
            let mut previous = previous_entries.next();
            if previous.as_ref().is_some_and(|entry| {
                entry
                    .commands
                    .upgrade()
                    .is_some_and(|current| Arc::ptr_eq(&current, commands))
            }) {
                next_entries.push(
                    previous
                        .take()
                        .expect("a matching UI texture dependency entry must remain available"),
                );
                continue;
            }

            changed_segment_count = changed_segment_count.saturating_add(1);
            if let Some(previous) = previous {
                remove_ui_texture_dependency_ids(
                    &mut self.active_ref_counts,
                    &mut touched_initial_presence,
                    &previous.ids,
                );
            }
            let ids = collect_ui_texture_command_segment_dependencies(commands);
            command_visit_count = command_visit_count.saturating_add(commands.len());
            add_ui_texture_dependency_ids(
                &mut self.active_ref_counts,
                &mut touched_initial_presence,
                &ids,
            );
            next_entries.push(UiTextureDependencySegmentEntry {
                commands: Arc::downgrade(commands),
                ids,
            });
        }
        for previous in previous_entries {
            changed_segment_count = changed_segment_count.saturating_add(1);
            remove_ui_texture_dependency_ids(
                &mut self.active_ref_counts,
                &mut touched_initial_presence,
                &previous.ids,
            );
        }
        self.segment_entries = next_entries;
        self.submission = Some(Arc::downgrade(submission));

        let mut added_ids = Vec::new();
        let mut removed_ids = Vec::new();
        for (id, was_present) in touched_initial_presence {
            match (was_present, self.active_ref_counts.contains_key(&id)) {
                (false, true) => added_ids.push(id),
                (true, false) => removed_ids.push(id),
                _ => {}
            }
        }
        added_ids.sort_unstable();
        removed_ids.sort_unstable();
        let dependency_set_changed =
            previous_generation.is_none() || !added_ids.is_empty() || !removed_ids.is_empty();
        if !dependency_set_changed {
            record_ui_texture_dependency_profile(
                segment_identity_visit_count,
                command_visit_count,
                changed_segment_count,
                self.active_ref_counts.len(),
            );
            return Arc::clone(
                self.product
                    .as_ref()
                    .expect("an unchanged dependency set must retain its prior product"),
            );
        }

        self.next_generation = self.next_generation.wrapping_add(1).max(1);
        let full_rebuild = previous_generation.is_none();
        let product = Arc::new(UiTextureDependencies {
            ids: Arc::from(self.active_ref_counts.keys().copied().collect::<Vec<_>>()),
            generation: self.next_generation,
            change_journal: UiTextureDependencyChangeJournal {
                // `then_some` evaluates its argument eagerly, which would
                // unwrap the absent generation on the first (full-rebuild)
                // publication.  Keep the delta base absent for that initial
                // product and only unwrap after the non-full-rebuild branch
                // has been selected.
                base_generation: (!full_rebuild).then(|| {
                    previous_generation
                        .expect("a local UI texture delta must retain its base generation")
                }),
                added_ids: Arc::from(added_ids),
                removed_ids: Arc::from(removed_ids),
                full_rebuild,
            },
        });
        self.product = Some(Arc::clone(&product));
        record_ui_texture_dependency_profile(
            segment_identity_visit_count,
            command_visit_count,
            changed_segment_count,
            self.active_ref_counts.len(),
        );
        product
    }

    pub(in crate::graphics::scene::resources) fn clear(&mut self) {
        self.submission = None;
        self.segment_entries.clear();
        self.active_ref_counts.clear();
        self.product = None;
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

fn collect_ui_texture_command_segment_dependencies(
    commands: &[UiRenderCommand],
) -> Arc<[ResourceId]> {
    let mut ids = HashSet::new();
    for command in commands {
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
    Arc::from(ids)
}

fn add_ui_texture_dependency_ids(
    active_ref_counts: &mut BTreeMap<ResourceId, usize>,
    touched_initial_presence: &mut HashMap<ResourceId, bool>,
    ids: &[ResourceId],
) {
    for &id in ids {
        touched_initial_presence
            .entry(id)
            .or_insert_with(|| active_ref_counts.contains_key(&id));
        let count = active_ref_counts.entry(id).or_default();
        *count = count.saturating_add(1);
    }
}

fn remove_ui_texture_dependency_ids(
    active_ref_counts: &mut BTreeMap<ResourceId, usize>,
    touched_initial_presence: &mut HashMap<ResourceId, bool>,
    ids: &[ResourceId],
) {
    for &id in ids {
        touched_initial_presence
            .entry(id)
            .or_insert_with(|| active_ref_counts.contains_key(&id));
        let Some(count) = active_ref_counts.get_mut(&id) else {
            continue;
        };
        *count = count.saturating_sub(1);
        if *count == 0 {
            active_ref_counts.remove(&id);
        }
    }
}

fn record_ui_texture_dependency_profile(
    segment_identity_visit_count: usize,
    command_visit_count: usize,
    changed_segment_count: usize,
    dependency_count: usize,
) {
    crate::core::diagnostics::profiling::record_counter_batch(
        "runtime",
        &[
            (
                "ui.ui_texture_dependencies.segment_identity_visit_count",
                segment_identity_visit_count as f64,
            ),
            (
                "ui.ui_texture_dependencies.command_visit_count",
                command_visit_count as f64,
            ),
            (
                "ui.ui_texture_dependencies.changed_segment_count",
                changed_segment_count as f64,
            ),
            (
                "ui.ui_texture_dependencies.active_dependency_count",
                dependency_count as f64,
            ),
        ],
    );
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
#[path = "tests/ui_texture.rs"]
mod tests;
