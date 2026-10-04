use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap, HashMap, HashSet};
use std::convert::Infallible;
use std::ops::Bound::{Excluded, Unbounded};

use crate::core::framework::render::{
    RenderComponentChangeArtifact, RenderComponentMeshPrimitiveBinding,
    RenderComponentProjectionMode, RenderComponentSourceWorldId, RenderComponentValue,
    RenderFrameExtract, RenderWorldSnapshotHandle,
};
use crate::core::resource::{
    MeshMarker, ModelMarker, ResourceHandle, ResourceKind, UntypedResourceHandle,
};

use super::super::{
    RenderScene, RenderSceneChangeJournal, RenderSceneDeltaTransactionError, RenderSceneReadView,
    RenderSceneResourceReferenceDelta,
};
use super::projection::{build_delta, build_delta_with_geometry_replay, reproject_geometry};
use super::{
    RenderSceneComponentProjectionError, RenderSceneComponentProjectionTransactionError,
    RenderSceneGeometryResolver,
};

#[derive(Clone, Debug)]
pub(crate) enum RenderSceneComponentProjectionCommit<StageOutput> {
    Applied {
        journal: RenderSceneChangeJournal,
        staging: StageOutput,
    },
    Replayed,
}

pub(crate) struct RenderSceneGeometryReplaySelection {
    targets: Vec<u64>,
    required_resources: Vec<UntypedResourceHandle>,
    // Snapshot used to invalidate a partial continuation when an asset advances mid-replay.
    resource_revisions: HashMap<UntypedResourceHandle, u64>,
    resync: bool,
    truncated: bool,
    component_geometry_changed: bool,
    resource_scope: HashSet<UntypedResourceHandle>,
}

impl RenderSceneGeometryReplaySelection {
    pub(crate) fn required_resources(&self) -> &[UntypedResourceHandle] {
        &self.required_resources
    }

    pub(crate) fn truncated(&self) -> bool {
        self.truncated
    }

    #[cfg(test)]
    pub(crate) fn selected_primitive_count(&self) -> usize {
        self.targets.len()
    }

    #[cfg(test)]
    pub(crate) fn target_keys(&self) -> &[u64] {
        &self.targets
    }
}

pub(crate) struct RenderSceneComponentProjector {
    bound_world: Option<RenderComponentSourceWorldId>,
    applied_journal_generation: u64,
    scene: RenderScene,
    geometry_dependents: HashMap<UntypedResourceHandle, BTreeSet<u64>>,
    geometry_primitive_dependencies: HashMap<u64, HashSet<UntypedResourceHandle>>,
    geometry_primitive_keys: BTreeSet<u64>,
    geometry_resync_after: Option<u64>,
    geometry_replay_after: Option<u64>,
    geometry_replay_scope: HashSet<UntypedResourceHandle>,
    // A stable-key cursor is valid only while every pending resource keeps the same revision.
    geometry_replay_revisions: HashMap<UntypedResourceHandle, u64>,
    #[cfg(test)]
    geometry_selection_visits: std::cell::Cell<usize>,
    #[cfg(test)]
    geometry_selection_peak_keys: std::cell::Cell<usize>,
}

impl RenderSceneComponentProjector {
    pub(crate) fn new(world: RenderWorldSnapshotHandle) -> Self {
        Self {
            bound_world: None,
            applied_journal_generation: 0,
            scene: RenderScene::new(world.with_generation(0)),
            geometry_dependents: HashMap::new(),
            geometry_primitive_dependencies: HashMap::new(),
            geometry_primitive_keys: BTreeSet::new(),
            geometry_resync_after: None,
            geometry_replay_after: None,
            geometry_replay_scope: HashSet::new(),
            geometry_replay_revisions: HashMap::new(),
            #[cfg(test)]
            geometry_selection_visits: std::cell::Cell::new(0),
            #[cfg(test)]
            geometry_selection_peak_keys: std::cell::Cell::new(0),
        }
    }

    pub(crate) fn read(&self) -> RenderSceneReadView<'_> {
        self.scene.read()
    }

    pub(in crate::graphics::scene::render_scene) fn resource_release_deltas(
        &self,
    ) -> Vec<RenderSceneResourceReferenceDelta> {
        self.scene.resource_release_deltas()
    }

    pub(crate) fn project_frame(
        &mut self,
        frame: &RenderFrameExtract,
        resolver: &mut impl RenderSceneGeometryResolver,
    ) -> Result<Option<RenderSceneChangeJournal>, RenderSceneComponentProjectionError> {
        match self.project_frame_with_staging(frame, resolver, |_| Ok::<(), Infallible>(())) {
            Ok(Some(RenderSceneComponentProjectionCommit::Applied { journal, .. })) => {
                Ok(Some(journal))
            }
            Ok(Some(RenderSceneComponentProjectionCommit::Replayed)) | Ok(None) => Ok(None),
            Err(RenderSceneComponentProjectionTransactionError::Projection(error)) => Err(error),
            Err(RenderSceneComponentProjectionTransactionError::Staging(never)) => match never {},
        }
    }

    /// Identifies a multi-view replay without resolving geometry or touching residency state.
    pub(crate) fn is_exact_frame_replay(
        &self,
        frame: &RenderFrameExtract,
    ) -> Result<bool, RenderSceneComponentProjectionError> {
        Ok(self
            .validate_frame(frame)?
            .is_some_and(|artifact| self.is_exact_artifact_replay(artifact)))
    }

    pub(crate) fn project_frame_with_staging<StageOutput, StageError>(
        &mut self,
        frame: &RenderFrameExtract,
        resolver: &mut impl RenderSceneGeometryResolver,
        stage: impl FnOnce(&[RenderSceneResourceReferenceDelta]) -> Result<StageOutput, StageError>,
    ) -> Result<
        Option<RenderSceneComponentProjectionCommit<StageOutput>>,
        RenderSceneComponentProjectionTransactionError<StageError>,
    > {
        let Some(artifact) = self
            .validate_frame(frame)
            .map_err(RenderSceneComponentProjectionTransactionError::Projection)?
        else {
            return Ok(None);
        };
        self.project_with_staging(artifact, resolver, stage)
            .map(Some)
    }

    pub(crate) fn project_frame_with_resource_geometry_staging<StageOutput, StageError>(
        &mut self,
        frame: &RenderFrameExtract,
        selection: &RenderSceneGeometryReplaySelection,
        resolver: &mut impl RenderSceneGeometryResolver,
        stage: impl FnOnce(&[RenderSceneResourceReferenceDelta]) -> Result<StageOutput, StageError>,
    ) -> Result<
        Option<RenderSceneComponentProjectionCommit<StageOutput>>,
        RenderSceneComponentProjectionTransactionError<StageError>,
    > {
        let artifact = self
            .validate_frame(frame)
            .map_err(RenderSceneComponentProjectionTransactionError::Projection)?;
        let targets = &selection.targets;
        let Some(artifact) = artifact else {
            if targets.is_empty() {
                if selection.resync {
                    self.geometry_resync_after = None;
                }
                return Ok(None);
            }
            return self
                .apply_geometry_replay_targets_with_staging(
                    targets,
                    selection.resync,
                    resolver,
                    stage,
                )
                .map(Some);
        };
        if self.is_exact_artifact_replay(artifact) {
            if targets.is_empty() {
                if selection.resync {
                    self.geometry_resync_after = None;
                }
                return Ok(Some(RenderSceneComponentProjectionCommit::Replayed));
            }
            return self
                .apply_geometry_replay_targets_with_staging(
                    targets,
                    selection.resync,
                    resolver,
                    stage,
                )
                .map(Some);
        }
        self.project_with_geometry_targets_staging(
            artifact,
            targets,
            selection.resync,
            resolver,
            stage,
        )
        .map(Some)
    }

    pub(crate) fn project_with_resource_geometry_staging<StageOutput, StageError>(
        &mut self,
        artifact: &RenderComponentChangeArtifact,
        resources: &[UntypedResourceHandle],
        resync: bool,
        limit: usize,
        resolver: &mut impl RenderSceneGeometryResolver,
        stage: impl FnOnce(&[RenderSceneResourceReferenceDelta]) -> Result<StageOutput, StageError>,
    ) -> Result<
        RenderSceneComponentProjectionCommit<StageOutput>,
        RenderSceneComponentProjectionTransactionError<StageError>,
    > {
        let (targets, _, _) =
            self.geometry_replay_targets(resources, &HashMap::new(), resync, limit);
        if self.is_exact_artifact_replay(artifact) {
            if targets.is_empty() {
                return Ok(RenderSceneComponentProjectionCommit::Replayed);
            }
            return self
                .apply_geometry_replay_targets_with_staging(&targets, resync, resolver, stage);
        }
        self.project_with_geometry_targets_staging(artifact, &targets, resync, resolver, stage)
    }

    pub(crate) fn project(
        &mut self,
        artifact: &RenderComponentChangeArtifact,
        resolver: &mut impl RenderSceneGeometryResolver,
    ) -> Result<Option<RenderSceneChangeJournal>, RenderSceneComponentProjectionError> {
        match self.project_with_staging(artifact, resolver, |_| Ok::<(), Infallible>(())) {
            Ok(RenderSceneComponentProjectionCommit::Applied { journal, .. }) => Ok(Some(journal)),
            Ok(RenderSceneComponentProjectionCommit::Replayed) => Ok(None),
            Err(RenderSceneComponentProjectionTransactionError::Projection(error)) => Err(error),
            Err(RenderSceneComponentProjectionTransactionError::Staging(never)) => match never {},
        }
    }

    pub(crate) fn project_with_staging<StageOutput, StageError>(
        &mut self,
        artifact: &RenderComponentChangeArtifact,
        resolver: &mut impl RenderSceneGeometryResolver,
        stage: impl FnOnce(&[RenderSceneResourceReferenceDelta]) -> Result<StageOutput, StageError>,
    ) -> Result<
        RenderSceneComponentProjectionCommit<StageOutput>,
        RenderSceneComponentProjectionTransactionError<StageError>,
    > {
        if self.is_exact_artifact_replay(artifact) {
            return Ok(RenderSceneComponentProjectionCommit::Replayed);
        }
        self.validate_artifact(artifact)
            .map_err(RenderSceneComponentProjectionTransactionError::Projection)?;
        let delta = build_delta(&self.scene, artifact, resolver)
            .map_err(RenderSceneComponentProjectionTransactionError::Projection)?;
        let (journal, staging) =
            self.scene
                .apply_delta_with_staging(delta, stage)
                .map_err(|error| match error {
                    RenderSceneDeltaTransactionError::Preflight(error) => {
                        RenderSceneComponentProjectionTransactionError::Projection(error.into())
                    }
                    RenderSceneDeltaTransactionError::Staging(error) => {
                        RenderSceneComponentProjectionTransactionError::Staging(error)
                    }
                })?;
        self.bound_world = Some(artifact.world());
        self.applied_journal_generation = artifact.journal_generation();
        self.geometry_resync_after = None;
        if Self::artifact_changes_geometry_dependencies(artifact) {
            self.geometry_replay_after = None;
            self.geometry_replay_scope.clear();
            self.geometry_replay_revisions.clear();
        }
        self.apply_geometry_dependency_journal(&journal, resolver);
        Ok(RenderSceneComponentProjectionCommit::Applied { journal, staging })
    }

    pub(crate) fn replay_resource_geometry_with_staging<StageOutput, StageError>(
        &mut self,
        resources: &[UntypedResourceHandle],
        resync: bool,
        limit: usize,
        resolver: &mut impl RenderSceneGeometryResolver,
        stage: impl FnOnce(&[RenderSceneResourceReferenceDelta]) -> Result<StageOutput, StageError>,
    ) -> Result<
        Option<RenderSceneComponentProjectionCommit<StageOutput>>,
        RenderSceneComponentProjectionTransactionError<StageError>,
    > {
        let (targets, _, _) =
            self.geometry_replay_targets(resources, &HashMap::new(), resync, limit);
        if targets.is_empty() {
            if resync {
                self.geometry_resync_after = None;
            }
            return Ok(None);
        }
        self.apply_geometry_replay_targets_with_staging(&targets, resync, resolver, stage)
            .map(Some)
    }

    fn apply_geometry_replay_targets_with_staging<StageOutput, StageError>(
        &mut self,
        targets: &[u64],
        resync: bool,
        resolver: &mut impl RenderSceneGeometryResolver,
        stage: impl FnOnce(&[RenderSceneResourceReferenceDelta]) -> Result<StageOutput, StageError>,
    ) -> Result<
        RenderSceneComponentProjectionCommit<StageOutput>,
        RenderSceneComponentProjectionTransactionError<StageError>,
    > {
        let upserts = {
            let read = self.scene.read();
            targets
                .iter()
                .filter_map(|key| {
                    read.handle_for_stable_key(*key)
                        .and_then(|handle| read.get(handle).cloned())
                })
                .map(|primitive| reproject_geometry(&primitive, resolver))
                .collect::<Result<Vec<_>, _>>()
                .map_err(RenderSceneComponentProjectionTransactionError::Projection)?
        };
        let (journal, staging) = self
            .scene
            .apply_delta_with_staging(
                super::super::RenderSceneDelta::new(upserts, Vec::new()),
                stage,
            )
            .map_err(|error| match error {
                super::super::RenderSceneDeltaTransactionError::Preflight(error) => {
                    RenderSceneComponentProjectionTransactionError::Projection(error.into())
                }
                super::super::RenderSceneDeltaTransactionError::Staging(error) => {
                    RenderSceneComponentProjectionTransactionError::Staging(error)
                }
            })?;
        self.apply_geometry_dependency_journal(&journal, resolver);
        self.advance_geometry_resync(targets, resync);
        Ok(RenderSceneComponentProjectionCommit::Applied { journal, staging })
    }

    fn project_with_geometry_targets_staging<StageOutput, StageError>(
        &mut self,
        artifact: &RenderComponentChangeArtifact,
        targets: &[u64],
        resync: bool,
        resolver: &mut impl RenderSceneGeometryResolver,
        stage: impl FnOnce(&[RenderSceneResourceReferenceDelta]) -> Result<StageOutput, StageError>,
    ) -> Result<
        RenderSceneComponentProjectionCommit<StageOutput>,
        RenderSceneComponentProjectionTransactionError<StageError>,
    > {
        self.validate_artifact(artifact)
            .map_err(RenderSceneComponentProjectionTransactionError::Projection)?;
        let delta = build_delta_with_geometry_replay(&self.scene, artifact, targets, resolver)
            .map_err(RenderSceneComponentProjectionTransactionError::Projection)?;
        let (journal, staging) =
            self.scene
                .apply_delta_with_staging(delta, stage)
                .map_err(|error| match error {
                    RenderSceneDeltaTransactionError::Preflight(error) => {
                        RenderSceneComponentProjectionTransactionError::Projection(error.into())
                    }
                    RenderSceneDeltaTransactionError::Staging(error) => {
                        RenderSceneComponentProjectionTransactionError::Staging(error)
                    }
                })?;
        self.bound_world = Some(artifact.world());
        self.applied_journal_generation = artifact.journal_generation();
        if Self::artifact_changes_geometry_dependencies(artifact) {
            self.geometry_replay_after = None;
            self.geometry_replay_scope.clear();
            self.geometry_replay_revisions.clear();
        }
        self.apply_geometry_dependency_journal(&journal, resolver);
        self.advance_geometry_resync(targets, resync);
        Ok(RenderSceneComponentProjectionCommit::Applied { journal, staging })
    }

    fn advance_geometry_resync(&mut self, targets: &[u64], resync: bool) {
        if !resync {
            return;
        }
        let Some(last) = targets.last().copied() else {
            self.geometry_resync_after = None;
            return;
        };
        let has_more = self
            .geometry_primitive_keys
            .range((Excluded(last), Unbounded))
            .next()
            .is_some();
        self.geometry_resync_after = has_more.then_some(last);
    }

    pub(crate) fn geometry_resync_complete(&self) -> bool {
        self.geometry_resync_after.is_none()
    }

    pub(crate) fn finish_geometry_replay_selection(
        &mut self,
        selection: &RenderSceneGeometryReplaySelection,
    ) {
        if selection.resync {
            return;
        }
        if selection.truncated && !selection.component_geometry_changed {
            self.geometry_replay_after = selection.targets.last().copied();
            self.geometry_replay_scope = selection.resource_scope.clone();
            self.geometry_replay_revisions = selection.resource_revisions.clone();
        } else {
            self.geometry_replay_after = None;
            self.geometry_replay_scope.clear();
            self.geometry_replay_revisions.clear();
        }
    }

    pub(crate) fn has_geometry_dependency(&self, resource: UntypedResourceHandle) -> bool {
        self.geometry_dependents.contains_key(&resource)
    }

    pub(crate) fn artifact_geometry_resources(
        artifact: &RenderComponentChangeArtifact,
    ) -> HashSet<UntypedResourceHandle> {
        let mut resources = HashSet::new();
        for patch in artifact.upserts() {
            let RenderComponentValue::Present(mesh) = patch.mesh_renderer() else {
                continue;
            };
            add_artifact_geometry_level(
                &mut resources,
                mesh.model(),
                mesh.mesh(),
                mesh.primitives(),
            );
            for lod in mesh.lods() {
                add_artifact_geometry_level(
                    &mut resources,
                    lod.model(),
                    lod.mesh(),
                    lod.primitives(),
                );
            }
        }
        resources
    }

    /// A model-backed source can resolve additional mesh resources from the prepared model
    /// payload. Those IDs are intentionally not copied into the component artifact, so the
    /// residency bridge must retain every geometry event while this source is first admitted.
    pub(crate) fn artifact_may_reference_supplemental_geometry(
        artifact: &RenderComponentChangeArtifact,
    ) -> bool {
        artifact.upserts().iter().any(|patch| {
            let RenderComponentValue::Present(mesh) = patch.mesh_renderer() else {
                return false;
            };
            mesh.mesh().is_none() || mesh.lods().iter().any(|lod| lod.mesh().is_none())
        })
    }

    #[cfg(test)]
    pub(crate) fn take_geometry_selection_visits(&self) -> usize {
        self.geometry_selection_visits.replace(0)
    }

    #[cfg(test)]
    pub(crate) fn take_geometry_selection_peak_keys(&self) -> usize {
        self.geometry_selection_peak_keys.replace(0)
    }

    fn record_geometry_selection_visit(&self) {
        #[cfg(test)]
        self.geometry_selection_visits
            .set(self.geometry_selection_visits.get().saturating_add(1));
    }

    fn record_geometry_selection_peak_keys(&self, _key_count: usize) {
        #[cfg(test)]
        self.geometry_selection_peak_keys
            .set(self.geometry_selection_peak_keys.get().max(_key_count));
    }

    pub(crate) fn prepare_geometry_replay(
        &self,
        resources: &[UntypedResourceHandle],
        resync: bool,
        limit: usize,
        artifact: Option<&RenderComponentChangeArtifact>,
    ) -> RenderSceneGeometryReplaySelection {
        self.prepare_geometry_replay_with_revision_map(
            resources,
            HashMap::new(),
            resync,
            limit,
            artifact,
        )
    }

    pub(crate) fn prepare_geometry_replay_with_revisions(
        &self,
        resources: &[(UntypedResourceHandle, u64)],
        resync: bool,
        limit: usize,
        artifact: Option<&RenderComponentChangeArtifact>,
    ) -> RenderSceneGeometryReplaySelection {
        let handles = resources
            .iter()
            .map(|(resource, _)| *resource)
            .collect::<Vec<_>>();
        let revisions = resources.iter().copied().collect::<HashMap<_, _>>();
        self.prepare_geometry_replay_with_revision_map(&handles, revisions, resync, limit, artifact)
    }

    fn prepare_geometry_replay_with_revision_map(
        &self,
        resources: &[UntypedResourceHandle],
        resource_revisions: HashMap<UntypedResourceHandle, u64>,
        resync: bool,
        limit: usize,
        artifact: Option<&RenderComponentChangeArtifact>,
    ) -> RenderSceneGeometryReplaySelection {
        let (targets, truncated, resource_scope) =
            self.geometry_replay_targets(resources, &resource_revisions, resync, limit);
        let read = self.scene.read();
        let full_component_projection = artifact.is_some_and(|artifact| {
            matches!(artifact.mode(), RenderComponentProjectionMode::Full(_))
        });
        let component_geometry_changed =
            artifact.is_some_and(Self::artifact_changes_geometry_dependencies);
        let component_geometry = artifact
            .into_iter()
            .flat_map(|artifact| {
                artifact
                    .upserts()
                    .iter()
                    .filter(|patch| {
                        !matches!(
                            patch.mesh_renderer(),
                            crate::core::framework::render::RenderComponentValue::Unchanged
                        )
                    })
                    .map(|patch| patch.entity())
                    .chain(artifact.removals().iter().copied())
            })
            .collect::<HashSet<_>>();
        let mut required = resources.iter().copied().collect::<HashSet<_>>();
        for key in &targets {
            let Some(primitive) = read
                .handle_for_stable_key(*key)
                .and_then(|handle| read.get(handle))
            else {
                continue;
            };
            if full_component_projection
                || component_geometry.contains(&primitive.descriptor().node_id)
            {
                continue;
            }
            required.extend(geometry_dependencies(primitive));
        }
        RenderSceneGeometryReplaySelection {
            targets,
            required_resources: required.into_iter().collect(),
            resource_revisions,
            resync,
            truncated,
            component_geometry_changed,
            resource_scope,
        }
    }

    fn geometry_replay_targets(
        &self,
        resources: &[UntypedResourceHandle],
        resource_revisions: &HashMap<UntypedResourceHandle, u64>,
        resync: bool,
        limit: usize,
    ) -> (Vec<u64>, bool, HashSet<UntypedResourceHandle>) {
        let resource_scope = resources.iter().copied().collect::<HashSet<_>>();
        let limit = limit.max(1);
        if resync {
            let targets = match self.geometry_resync_after {
                Some(after) => self
                    .geometry_primitive_keys
                    .range((Excluded(after), Unbounded))
                    .take(limit)
                    .inspect(|_| self.record_geometry_selection_visit())
                    .copied()
                    .collect::<Vec<_>>(),
                None => self
                    .geometry_primitive_keys
                    .iter()
                    .take(limit)
                    .inspect(|_| self.record_geometry_selection_visit())
                    .copied()
                    .collect::<Vec<_>>(),
            };
            self.record_geometry_selection_peak_keys(targets.len());
            return (targets, false, resource_scope);
        }

        let after = if self.geometry_replay_revisions.eq(resource_revisions)
            && self.geometry_replay_scope == resource_scope
        {
            self.geometry_replay_after
        } else {
            None
        };
        let mut ranges = resource_scope
            .iter()
            .filter_map(|resource| self.geometry_dependents.get(resource))
            .map(|dependents| dependents.range((after.map_or(Unbounded, Excluded), Unbounded)))
            .collect::<Vec<_>>();
        let mut heads = BinaryHeap::with_capacity(ranges.len());
        for (index, range) in ranges.iter_mut().enumerate() {
            if let Some(key) = range.next() {
                self.record_geometry_selection_visit();
                heads.push(Reverse((*key, index)));
                self.record_geometry_selection_peak_keys(heads.len());
            }
        }

        let mut targets = Vec::new();
        while let Some(&Reverse((key, _))) = heads.peek() {
            // Heads include the next distinct key, so the lookahead is retained only once.
            self.record_geometry_selection_peak_keys(heads.len() + targets.len());
            if targets.len() == limit {
                return (targets, true, resource_scope);
            }
            targets.push(key);
            self.record_geometry_selection_peak_keys(heads.len() + targets.len());
            // Consume every membership of this key before exposing the next distinct key.
            // Each range reads at most limit + 1 entries, including its initial/lookahead head.
            while heads.peek().is_some_and(|Reverse((next, _))| *next == key) {
                let Reverse((_, index)) = heads.pop().expect("peeked geometry dependency head");
                if let Some(next) = ranges[index].next() {
                    self.record_geometry_selection_visit();
                    heads.push(Reverse((*next, index)));
                }
                self.record_geometry_selection_peak_keys(heads.len() + targets.len());
            }
        }
        (targets, false, resource_scope)
    }

    fn artifact_changes_geometry_dependencies(artifact: &RenderComponentChangeArtifact) -> bool {
        matches!(artifact.mode(), RenderComponentProjectionMode::Full(_))
            || !artifact.removals().is_empty()
            || artifact.upserts().iter().any(|patch| {
                !matches!(
                    patch.mesh_renderer(),
                    crate::core::framework::render::RenderComponentValue::Unchanged
                )
            })
    }

    fn apply_geometry_dependency_journal(
        &mut self,
        journal: &RenderSceneChangeJournal,
        resolver: &impl RenderSceneGeometryResolver,
    ) {
        for removal in journal.removals() {
            self.geometry_primitive_keys
                .remove(&removal.primitive().stable_instance_key());
            self.remove_geometry_dependencies(removal.primitive());
        }
        for update in journal.updates() {
            self.geometry_primitive_keys
                .insert(update.primitive().stable_instance_key());
            self.remove_geometry_dependencies(update.previous_primitive());
            self.add_geometry_dependencies(update.primitive(), resolver);
        }
        for addition in journal.additions() {
            self.geometry_primitive_keys
                .insert(addition.primitive().stable_instance_key());
            self.add_geometry_dependencies(addition.primitive(), resolver);
        }
    }

    fn add_geometry_dependencies(
        &mut self,
        primitive: &super::super::RenderScenePrimitive,
        resolver: &impl RenderSceneGeometryResolver,
    ) {
        let key = primitive.stable_instance_key();
        let mut dependencies = geometry_dependencies(primitive);
        dependencies.extend(
            resolver.supplemental_geometry_dependencies(&primitive.descriptor().mesh_source),
        );
        for resource in &dependencies {
            self.geometry_dependents
                .entry(*resource)
                .or_default()
                .insert(key);
        }
        self.geometry_primitive_dependencies
            .insert(key, dependencies);
    }

    fn remove_geometry_dependencies(&mut self, primitive: &super::super::RenderScenePrimitive) {
        let key = primitive.stable_instance_key();
        for resource in self
            .geometry_primitive_dependencies
            .remove(&key)
            .unwrap_or_default()
        {
            let remove_resource =
                if let Some(dependents) = self.geometry_dependents.get_mut(&resource) {
                    dependents.remove(&key);
                    dependents.is_empty()
                } else {
                    false
                };
            if remove_resource {
                self.geometry_dependents.remove(&resource);
            }
        }
    }

    fn is_exact_artifact_replay(&self, artifact: &RenderComponentChangeArtifact) -> bool {
        self.bound_world == Some(artifact.world())
            && self.applied_journal_generation == artifact.journal_generation()
    }

    fn validate_frame<'a>(
        &self,
        frame: &'a RenderFrameExtract,
    ) -> Result<Option<&'a RenderComponentChangeArtifact>, RenderSceneComponentProjectionError>
    {
        let expected_world = self.scene.read().world();
        if frame.world.raw() != expected_world.raw() {
            return Err(RenderSceneComponentProjectionError::FrameWorldMismatch {
                expected: expected_world,
                incoming: frame.world.with_generation(0),
            });
        }
        let Some(artifact) = frame.geometry.scene_changes.as_deref() else {
            return Ok(None);
        };
        if artifact.source_world_generation() > frame.world.generation() {
            return Err(RenderSceneComponentProjectionError::ArtifactAheadOfFrame {
                frame_generation: frame.world.generation(),
                artifact_generation: artifact.source_world_generation(),
            });
        }
        Ok(Some(artifact))
    }

    fn validate_artifact(
        &self,
        artifact: &RenderComponentChangeArtifact,
    ) -> Result<(), RenderSceneComponentProjectionError> {
        let expected_world = RenderComponentSourceWorldId::new(self.scene.read().world().raw());
        if artifact.world() != expected_world {
            return Err(RenderSceneComponentProjectionError::WorldMismatch {
                expected: expected_world,
                incoming: artifact.world(),
            });
        }
        let Some(bound_world) = self.bound_world else {
            return if matches!(artifact.mode(), RenderComponentProjectionMode::Full(_)) {
                Ok(())
            } else {
                Err(RenderSceneComponentProjectionError::IncrementalWithoutBaseline)
            };
        };
        if bound_world != artifact.world() {
            return Err(RenderSceneComponentProjectionError::WorldMismatch {
                expected: bound_world,
                incoming: artifact.world(),
            });
        }
        if artifact.journal_generation() < self.applied_journal_generation {
            return Err(RenderSceneComponentProjectionError::StaleArtifact {
                applied_generation: self.applied_journal_generation,
                incoming_generation: artifact.journal_generation(),
            });
        }
        if artifact.journal_generation() != self.applied_journal_generation.saturating_add(1)
            && !matches!(artifact.mode(), RenderComponentProjectionMode::Full(_))
        {
            return Err(RenderSceneComponentProjectionError::JournalDiscontinuity {
                applied_generation: self.applied_journal_generation,
                incoming_generation: artifact.journal_generation(),
            });
        }
        Ok(())
    }
}

fn geometry_dependencies(
    primitive: &super::super::RenderScenePrimitive,
) -> HashSet<UntypedResourceHandle> {
    let mut resources = HashSet::new();
    let source = &primitive.descriptor().mesh_source;
    for level in std::iter::once(source.base()).chain(source.lods().iter().map(|lod| &lod.source)) {
        if let Some(mesh) = level.mesh {
            resources.insert(UntypedResourceHandle::new(mesh.id(), ResourceKind::Mesh));
        } else {
            resources.insert(UntypedResourceHandle::new(
                level.model.id(),
                ResourceKind::Model,
            ));
        }
        resources.extend(
            level
                .primitives
                .iter()
                .map(|binding| UntypedResourceHandle::new(binding.mesh.id(), ResourceKind::Mesh)),
        );
    }
    resources
}

fn add_artifact_geometry_level(
    resources: &mut HashSet<UntypedResourceHandle>,
    model: ResourceHandle<ModelMarker>,
    mesh: Option<ResourceHandle<MeshMarker>>,
    primitives: &[RenderComponentMeshPrimitiveBinding],
) {
    resources.insert(match mesh {
        Some(mesh) => UntypedResourceHandle::new(mesh.id(), ResourceKind::Mesh),
        None => UntypedResourceHandle::new(model.id(), ResourceKind::Model),
    });
    resources.extend(
        primitives
            .iter()
            .map(|binding| UntypedResourceHandle::new(binding.mesh().id(), ResourceKind::Mesh)),
    );
}
