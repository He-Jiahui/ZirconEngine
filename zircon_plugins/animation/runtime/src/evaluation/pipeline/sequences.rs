use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use zircon_runtime::animation::CompiledAnimationSequence;
use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::core::framework::animation::{
    compiler::sequence::{compile_animation_sequence, AnimationCompiledSequence},
    AnimationSequenceAsset,
};
use zircon_runtime::core::resource::{
    ResourceActiveRow, ResourceActiveVersionKey, ResourceKind, ResourceManager,
    ResourceProjectionSnapshot, ResourceReadinessRowIdentity, ResourceRecord, ResourceSnapshot,
    ResourceState,
};
use zircon_runtime::scene::components::AnimationSequencePlayerComponent;
use zircon_runtime::scene::ecs::{ChangeTick, ComponentTicks};
use zircon_runtime::scene::{EntityId, LevelSystem, World};

use super::{
    requests::{playback_requires_continuous_frame, PendingSequenceSample},
    AnimationEvaluationPipeline,
};

#[cfg(test)]
#[path = "sequences/tests/perf.rs"]
mod optimization_batch_20260830cq_tests;

type Loaded = (
    ResourceActiveVersionKey,
    ResourceSnapshot<AnimationSequenceAsset>,
);
type Slots = Arc<[(usize, usize)]>;
pub(super) type SequenceClockUpdate = (EntityId, AnimationSequencePlayerComponent, SequenceOwner);

#[derive(Clone, Debug)]
pub(super) struct SequenceOwner {
    ticks: ComponentTicks,
    player: AnimationSequencePlayerComponent,
}

impl SequenceOwner {
    pub(super) fn capture(
        world: &World,
        entity: EntityId,
        player: &AnimationSequencePlayerComponent,
    ) -> Option<Self> {
        let ticks = world.component_change_ticks::<AnimationSequencePlayerComponent>(entity)?;
        Some(Self {
            ticks,
            player: player.clone(),
        })
    }
    pub(super) fn added(&self) -> ChangeTick {
        self.ticks.added()
    }
    fn same(&self, ticks: ComponentTicks, player: &AnimationSequencePlayerComponent) -> bool {
        self.ticks == ticks
            && self.player.sequence.id() == player.sequence.id()
            && self.player.playing == player.playing
            && self.player.looping == player.looping
            && self.player.time_seconds.to_bits() == player.time_seconds.to_bits()
            && self.player.playback_speed.to_bits() == player.playback_speed.to_bits()
    }
    pub(super) fn is_current(&self, world: &World, entity: EntityId) -> bool {
        world
            .component_change_ticks::<AnimationSequencePlayerComponent>(entity)
            .is_some_and(|ticks| {
                world
                    .animation_sequence_player(entity)
                    .is_some_and(|player| self.same(ticks, player))
            })
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct SequenceObservation {
    active: Option<Arc<ResourceActiveRow>>,
    catalog: Option<ResourceReadinessRowIdentity>,
}

impl SequenceObservation {
    pub(super) fn read(view: &ResourceProjectionSnapshot, id: AssetId) -> Self {
        Self {
            active: view
                .active()
                .row(id)
                .filter(|row| row.record.kind == ResourceKind::AnimationSequence)
                .cloned(),
            catalog: view
                .readiness()
                .row_identity(id)
                .filter(|row| row.row().record.kind == ResourceKind::AnimationSequence),
        }
    }
    fn key(&self) -> Option<&ResourceActiveVersionKey> {
        self.active.as_ref().map(|row| &row.key)
    }
    fn source(&self) -> Option<&ResourceRecord> {
        self.catalog
            .as_ref()
            .map(|row| row.row().record.as_ref())
            .filter(|row| row.state == ResourceState::Ready)
            .or_else(|| self.active.as_ref().map(|row| row.record.as_ref()))
    }
    fn matches(&self, (key, payload): &Loaded) -> bool {
        self.active
            .as_ref()
            .is_none_or(|row| &row.key == key && version_matches(&row.record, payload.record()))
            && self
                .source()
                .is_some_and(|row| version_matches(row, payload.record()))
    }
    fn same_observation(&self, other: &Self) -> bool {
        self.key() == other.key() && self.catalog == other.catalog
    }
}

fn version_matches(a: &ResourceRecord, b: &ResourceRecord) -> bool {
    std::ptr::eq(a, b)
        || (a.id == b.id
            && a.kind == b.kind
            && a.revision == b.revision
            && a.artifact_locator == b.artifact_locator
            && a.source_hash == b.source_hash
            && a.importer_id == b.importer_id
            && a.importer_version == b.importer_version
            && a.config_hash == b.config_hash)
}

#[derive(Clone, Debug)]
pub(super) struct SequenceRetry {
    key: ResourceActiveVersionKey,
    validated: Arc<AnimationCompiledSequence>,
    slots: Slots,
    catalog: Option<u64>,
    refused_key: Option<ResourceActiveVersionKey>,
}

#[derive(Debug)]
struct Receipt {
    owner: SequenceOwner,
    actual: Arc<ResourceRecord>,
    retry: Option<SequenceRetry>,
}

#[derive(Debug, Default)]
pub(super) struct SequencePlayback {
    receipts: BTreeMap<EntityId, Receipt>,
    // Unknown cold/backend failures have no reliable completion event. Retry on a new
    // observation, or at most every fourth actual evaluation frame, never demand a frame.
    waits: BTreeMap<AssetId, (SequenceObservation, u64, bool)>,
    frame: u64,
}

impl SequencePlayback {
    pub(super) fn begin_frame(&mut self) {
        self.frame = self.frame.saturating_add(1);
    }

    pub(super) fn select(
        &self,
        entity: EntityId,
        owner: &SequenceOwner,
        source: &SequenceObservation,
        catalog: u64,
    ) -> Option<Option<SequenceRetry>> {
        if self
            .waiting(owner.player.sequence.id(), source)
            .is_some_and(|has_duration| !owner.player.playing || !has_duration)
        {
            return None;
        }
        let receipt = self.receipts.get(&entity);
        if owner.player.playing
            || receipt.is_none_or(|r| {
                !r.owner.same(owner.ticks, &owner.player)
                    || source
                        .source()
                        .is_some_and(|seen| !version_matches(seen, &r.actual))
            })
        {
            return Some(None);
        }
        let retry = receipt?.retry.as_ref()?;
        (Some(catalog) != retry.catalog
            && (retry.refused_key.is_none() || retry.refused_key.as_ref() != source.key()))
        .then(|| Some(retry.clone()))
    }

    pub(super) fn prune(
        &mut self,
        mut live: Vec<(EntityId, AssetId, ChangeTick)>,
        observations: &BTreeMap<AssetId, SequenceObservation>,
    ) {
        live.sort_unstable_by_key(|entry| entry.0);
        self.receipts.retain(|entity, r| {
            live.binary_search_by_key(entity, |entry| entry.0)
                .ok()
                .is_some_and(|index| {
                    live[index].1 == r.actual.id && live[index].2 == r.owner.added()
                })
        });
        self.waits.retain(|id, _| observations.contains_key(id));
    }

    fn waiting(&self, id: AssetId, source: &SequenceObservation) -> Option<bool> {
        self.waits
            .get(&id)
            .filter(|(seen, until, _)| self.frame < *until && seen.same_observation(source))
            .map(|(_, _, has_duration)| *has_duration)
    }
}

#[derive(Debug)]
pub(super) struct CachedCompiledSequence {
    key: ResourceActiveVersionKey,
    revision: u64,
    validated: Arc<AnimationCompiledSequence>,
    compiled: CompiledAnimationSequence,
}

fn compiled_for<'a>(
    cache: &'a mut BTreeMap<AssetId, CachedCompiledSequence>,
    world: &mut World,
    loaded: &Loaded,
    retry: Option<&SequenceRetry>,
) -> Option<&'a CachedCompiledSequence> {
    let (key, payload) = loaded;
    let id = key.id();
    let reusable = cache
        .get(&id)
        .filter(|c| &c.key == key && c.revision == payload.revision());
    if reusable.is_none_or(|c| !c.compiled.is_current_for(world)) {
        let validated = reusable
            .map(|c| Arc::clone(&c.validated))
            .or_else(|| {
                retry
                    .filter(|r| &r.key == key)
                    .map(|r| Arc::clone(&r.validated))
            })
            .or_else(|| {
                compile_animation_sequence(payload.resource())
                    .into_artifact()
                    .map(Arc::new)
            })?;
        let compiled = CompiledAnimationSequence::from_validated_playback_source(
            world,
            Arc::clone(&validated),
        )
        .ok()?;
        cache.insert(
            id,
            CachedCompiledSequence {
                key: key.clone(),
                revision: payload.revision(),
                validated,
                compiled,
            },
        );
    }
    cache.get(&id)
}

pub(super) fn prepare_sequences(
    manager: &ProjectAssetManager,
    samples: &[PendingSequenceSample],
    demand: &mut bool,
) -> BTreeMap<AssetId, Option<Loaded>> {
    let mut loaded = BTreeMap::new();
    for sample in samples {
        let entry = loaded.entry(sample.sequence_id).or_insert_with(|| {
            manager
                .load_animation_sequence_playback_snapshot(sample.sequence_id)
                .ok()
        });
        if let Some((_, actual)) = entry {
            *demand |= playback_requires_continuous_frame(
                sample.playing,
                sample.playback_speed,
                sample.time_seconds,
                actual.duration_seconds,
                sample.looping,
            );
        }
    }
    loaded
}

fn batch_matches(
    expected: &SequenceObservation,
    batch: &SequenceObservation,
    loaded: &Loaded,
) -> bool {
    expected.matches(loaded) && batch.key() == Some(&loaded.0) && batch.matches(loaded)
}

pub(super) fn apply_loaded_sequences(
    level: &LevelSystem,
    epoch: u64,
    resources: &ResourceManager,
    samples: &[PendingSequenceSample],
    loaded: &BTreeMap<AssetId, Option<Loaded>>,
    deferred: &BTreeSet<EntityId>,
) -> bool {
    level
        .with_world_mut_if_replacement_epoch(epoch, |world| {
            // This is a read cut for the whole admitted batch; callbacks may publish the next frame's
            // resources without either mixing sources or holding an authority lock across writes.
            let view = resources.projection_snapshot();
            let batch = loaded
                .keys()
                .map(|&id| (id, SequenceObservation::read(&view, id)))
                .collect::<BTreeMap<_, _>>();
            let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
            let mut projection = pipeline.take_projection();
            let mut cache = pipeline.take_sequence_cache();
            let mut requested = Vec::new();
            let mut comparisons = BTreeMap::new();
            for sample in samples {
                let entity = sample.entity;
                if deferred.contains(&entity)
                    || !projection.sequence_world_is_current(world)
                    || !sample.owner.is_current(world, entity)
                {
                    continue;
                }
                let id = sample.sequence_id;
                let playback = &mut projection.sequence_playback;
                if playback.waiting(id, &sample.source).is_some() {
                    continue;
                }
                let current = &batch[&id];
                let entry = loaded[&id].as_ref();
                if entry.is_some() {
                    requested.push(id);
                }
                let load = entry.filter(|load| batch_matches(&sample.source, current, load));
                let compiled = if let Some(load) = load {
                    compiled_for(&mut cache, world, load, sample.retry.as_ref())
                } else {
                    None
                };
                let (Some(load), Some(compiled)) = (load, compiled) else {
                    playback.waits.insert(
                        id,
                        (
                            sample.source.clone(),
                            playback.frame.saturating_add(4),
                            entry.is_some(),
                        ),
                    );
                    continue;
                };
                playback.waits.remove(&id);
                if let Some(retry) = &sample.retry {
                    // Pointer is only a memo index; samples keep that source alive across this batch.
                    let equal = retry.key == load.0
                        || *comparisons
                            .entry((id, Arc::as_ptr(&retry.validated) as usize))
                            .or_insert_with(|| {
                                compiled.compiled.has_playback_source(&retry.validated)
                            });
                    if !equal {
                        if let Some(retry) = playback
                            .receipts
                            .get_mut(&entity)
                            .and_then(|r| r.retry.as_mut())
                        {
                            retry.refused_key = Some(load.0.clone());
                        }
                        continue;
                    }
                }
                let Some((catalog, slots)) = compiled.compiled.apply_playback_slots(
                    world,
                    sample.time_seconds,
                    sample.looping,
                    sample.retry.as_ref().map(|r| &r.slots),
                    |world| {
                        projection.sequence_world_is_current(world)
                            && sample.owner.is_current(world, entity)
                    },
                ) else {
                    continue;
                };
                let playback = &mut projection.sequence_playback;
                if sample.playing {
                    playback.receipts.remove(&entity);
                } else {
                    let retry = (!slots.is_empty()).then(|| SequenceRetry {
                        key: load.0.clone(),
                        validated: Arc::clone(&compiled.validated),
                        slots,
                        catalog,
                        refused_key: None,
                    });
                    let actual = Arc::clone(
                        &current
                            .active
                            .as_ref()
                            .expect("matched batch has active row")
                            .record,
                    );
                    playback.receipts.insert(
                        entity,
                        Receipt {
                            owner: sample.owner.clone(),
                            actual,
                            retry,
                        },
                    );
                }
            }
            if !projection.sequence_world_is_current(world) {
                return false;
            }
            let Some(pipeline) = world.get_resource_mut::<AnimationEvaluationPipeline>() else {
                return false;
            };
            if !requested.is_empty() {
                let requested = sorted_requested_assets(requested);
                cache.retain(|id, _| requested_asset_is_current(&requested, id));
            }
            pipeline.restore_sequence_cache(cache);
            pipeline.restore_projection(projection);
            true
        })
        .unwrap_or(false)
}

fn sorted_requested_assets(assets: impl IntoIterator<Item = AssetId>) -> Vec<AssetId> {
    let mut assets = assets.into_iter().collect::<Vec<_>>();
    assets.sort_unstable();
    assets.dedup();
    assets
}

fn requested_asset_is_current(requested_assets: &[AssetId], asset_id: &AssetId) -> bool {
    requested_assets.binary_search(asset_id).is_ok()
}

#[cfg(test)]
mod playback_cache_tests {
    use super::*;
    use zircon_runtime::core::framework::animation::{
        AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
        AnimationInterpolationAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
    };
    use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};
    use zircon_runtime::core::resource::{
        AnimationSequenceMarker, ResourceHandle, ResourceLocator,
    };
    use zircon_runtime::scene::components::NodeKind;

    #[test]
    fn property_sequence_pipeline_reuses_only_paired_current_world_projections() {
        let manager = ResourceManager::new();
        let uri = ResourceLocator::parse("mem://cache/sequence").unwrap();
        let id = AssetId::from_locator(&uri);
        let record = ResourceRecord::new(id, ResourceKind::AnimationSequence, uri)
            .with_source_hash("p1")
            .with_state(ResourceState::Ready);
        manager.register_lazy_record(record.clone()).unwrap();
        let observe = || SequenceObservation::read(&manager.projection_snapshot(), id);
        let read = || {
            let key = manager.projection_snapshot().active().key(id).unwrap();
            let actual = manager
                .snapshot_expected(ResourceHandle::<AnimationSequenceMarker>::new(id), &key)
                .unwrap();
            (key, actual)
        };
        let cold = observe();
        assert!(cold.key().is_none());
        let mut asset = AnimationSequenceAsset {
            name: None,
            duration_seconds: 1.0,
            frames_per_second: 30.0,
            bindings: vec![AnimationSequenceBindingAsset {
                entity_path: EntityPath::parse("CacheTarget").unwrap(),
                target_id: None,
                tracks: vec![AnimationSequenceTrackAsset {
                    property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                    channel: AnimationChannelAsset {
                        interpolation: AnimationInterpolationAsset::Step,
                        keys: vec![AnimationChannelKeyAsset {
                            time_seconds: 0.0,
                            value: AnimationChannelValueAsset::Vec3([2.0, 0.0, 0.0]),
                            in_tangent: None,
                            out_tangent: None,
                        }],
                    },
                }],
            }],
        };
        manager
            .register_ready(record.clone(), asset.clone())
            .unwrap();
        let p1 = read();
        let warm = observe();
        assert!(batch_matches(&cold, &warm, &p1));
        assert!(!batch_matches(&SequenceObservation::default(), &warm, &p1));
        let mut world = World::empty();
        let target = world.spawn_node(NodeKind::Empty).unwrap();
        world.rename_node(target, "CacheTarget").unwrap();
        let mut cache = BTreeMap::new();
        let source = Arc::clone(
            &compiled_for(&mut cache, &mut world, &p1, None)
                .unwrap()
                .validated,
        );
        let after_compile = world.compiled_scene_property_access_stats();
        assert_eq!(after_compile.target_compilations, 1);
        let compiled = compiled_for(&mut cache, &mut world, &p1, None).unwrap();
        assert!(Arc::ptr_eq(&source, &compiled.validated));
        assert_eq!(world.compiled_scene_property_access_stats(), after_compile);
        let (_, pending) = compiled
            .compiled
            .apply_playback_slots(&mut world, 0.5, false, None, |_| true)
            .unwrap();
        assert!(pending.is_empty());
        assert_eq!(
            world
                .get::<zircon_runtime::scene::components::LocalTransform>(target)
                .unwrap()
                .transform
                .translation
                .x,
            2.0
        );
        world.rename_node(target, "Away").unwrap();
        assert!(!compiled.compiled.is_current_for(&world));
        let recompiled = compiled_for(&mut cache, &mut world, &p1, None).unwrap();
        assert!(Arc::ptr_eq(&source, &recompiled.validated));
        assert_eq!(recompiled.compiled.missing_tracks().len(), 1);
        asset.bindings[0].tracks[0].channel.keys[0].value =
            AnimationChannelValueAsset::Vec3([7.0, 0.0, 0.0]);
        manager
            .register_ready(record.with_source_hash("p2"), asset)
            .unwrap();
        let p2 = read();
        let newer = observe();
        assert!(p2.1.revision() > p1.1.revision());
        assert!(!batch_matches(&warm, &newer, &p2));
        assert!(!batch_matches(&warm, &newer, &p1));
        assert!(batch_matches(&newer, &newer, &p2));
        world.rename_node(target, "CacheTarget").unwrap();
        let compiled = compiled_for(&mut cache, &mut world, &p2, None).unwrap();
        assert_eq!(compiled.key, p2.0);
        assert_eq!(compiled.revision, p2.1.revision());
        assert!(!Arc::ptr_eq(&source, &compiled.validated));
        assert!(compiled
            .compiled
            .apply_playback_slots(&mut world, 0.5, false, None, |_| false)
            .is_none());
        assert_eq!(
            world
                .get::<zircon_runtime::scene::components::LocalTransform>(target)
                .unwrap()
                .transform
                .translation
                .x,
            2.0,
            "a retired playback owner must preserve the last accepted value"
        );
        assert!(compiled
            .compiled
            .apply_playback_slots(&mut world, 0.5, false, None, |_| true)
            .unwrap()
            .1
            .is_empty());
        assert_eq!(
            world
                .get::<zircon_runtime::scene::components::LocalTransform>(target)
                .unwrap()
                .transform
                .translation
                .x,
            7.0
        );
    }
}
