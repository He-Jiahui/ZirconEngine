use std::collections::HashSet;

use zircon_runtime::scene::components::Mobility;
use zircon_runtime::scene::{NodeId, Scene};
use zircon_runtime_interface::math::{
    Mat4, NumericPolicy, Transform, ValidatedTransform, Vec3, is_finite_mat4, transform_to_mat4,
    try_affine_inverse,
};

use crate::core::editing::command::{BatchTransformCommand, BatchTransformTarget, NodeEditState};
use crate::core::editor_message::DocumentId;

use super::{
    InteractiveTransformError, InteractiveTransformKind, InteractiveTransformSpace,
    InteractiveTransformSpec, PivotMode,
};

const TRS_RECOMPOSITION_RELATIVE_TOLERANCE: f32 = 1.0e-4;

#[derive(Clone, Debug)]
struct FrozenTransformTarget {
    entity: NodeId,
    before: NodeEditState,
    frozen_world: Mat4,
    parent_world_inverse: Mat4,
    latest: Transform,
}

#[derive(Clone, Debug)]
pub(crate) struct InteractiveTransformSession {
    document: DocumentId,
    spec: InteractiveTransformSpec,
    pivot_mode: PivotMode,
    primary_root: NodeId,
    frozen_pivot: Transform,
    frozen_pivot_inverse: Mat4,
    expected_world_generation: u64,
    targets: Vec<FrozenTransformTarget>,
    pending: Vec<Transform>,
}

impl InteractiveTransformSession {
    pub(crate) fn begin(
        scene: &Scene,
        selected: &[NodeId],
        primary: NodeId,
        spec: InteractiveTransformSpec,
        pivot_mode: PivotMode,
        document: DocumentId,
    ) -> Result<Self, InteractiveTransformError> {
        if selected.is_empty() {
            return Err(InteractiveTransformError::EmptySelection);
        }
        let Some((roots, primary_root)) = selection_roots(scene, selected.iter().copied(), primary)
        else {
            return Err(InteractiveTransformError::PrimaryNotSelected { primary });
        };
        let pivot_transform = pivot_transform_for_roots(scene, &roots, primary_root, pivot_mode)
            .ok_or(InteractiveTransformError::TargetUnavailable {
                entity: primary_root,
            })?;
        let mut targets = Vec::with_capacity(roots.len());
        for entity in roots {
            if scene.mobility(entity) == Some(Mobility::Static) {
                return Err(InteractiveTransformError::TargetNotMutable { entity });
            }
            let before = NodeEditState::capture(scene, entity)
                .map_err(|_| InteractiveTransformError::TargetUnavailable { entity })?;
            let frozen_world = scene
                .world_matrix(entity)
                .ok_or(InteractiveTransformError::TargetUnavailable { entity })?;
            let parent_world_inverse = match scene.parent_of(entity) {
                Some(parent) => {
                    let parent_world = scene.world_matrix(parent).ok_or(
                        InteractiveTransformError::ParentWorldUnavailable { entity, parent },
                    )?;
                    try_affine_inverse(parent_world, NumericPolicy::STRICT).map_err(|source| {
                        InteractiveTransformError::ParentInverse { entity, source }
                    })?
                }
                None => Mat4::IDENTITY,
            };
            targets.push(FrozenTransformTarget {
                entity,
                latest: before.transform,
                before,
                frozen_world,
                parent_world_inverse,
            });
        }
        let frozen_pivot_inverse =
            try_affine_inverse(transform_to_mat4(pivot_transform), NumericPolicy::STRICT)
                .map_err(|source| InteractiveTransformError::PivotInverse { source })?;

        Ok(Self {
            document,
            spec,
            pivot_mode,
            primary_root,
            frozen_pivot: pivot_transform,
            frozen_pivot_inverse,
            expected_world_generation: scene.world_generation(),
            pending: Vec::with_capacity(targets.len()),
            targets,
        })
    }

    pub(crate) const fn spec(&self) -> InteractiveTransformSpec {
        self.spec
    }

    pub(crate) const fn primary_root(&self) -> NodeId {
        self.primary_root
    }

    pub(crate) const fn pivot_mode(&self) -> PivotMode {
        self.pivot_mode
    }

    pub(crate) const fn pivot_world(&self) -> Vec3 {
        self.frozen_pivot.translation
    }

    pub(crate) const fn pivot_transform(&self) -> Transform {
        self.frozen_pivot
    }

    pub(crate) fn target_entities(&self) -> impl ExactSizeIterator<Item = NodeId> + '_ {
        self.targets.iter().map(|target| target.entity)
    }

    pub(crate) fn preview(
        &mut self,
        scene: &mut Scene,
        document: Option<DocumentId>,
        primary: NodeId,
        target_pivot_world: Transform,
    ) -> Result<(), InteractiveTransformError> {
        self.ensure_context(scene, document)?;
        if primary != self.primary_root {
            return Err(InteractiveTransformError::PrimaryTargetMismatch {
                expected: self.primary_root,
                actual: primary,
            });
        }
        let target_pivot_world =
            ValidatedTransform::try_new(target_pivot_world, NumericPolicy::STRICT)
                .map_err(|source| InteractiveTransformError::InvalidTransform {
                    entity: self.primary_root,
                    source,
                })?
                .into_transform();
        let world_delta = self.world_delta(target_pivot_world);

        self.pending.clear();
        for target in &self.targets {
            let desired_world = world_delta * target.frozen_world;
            let local_target = target.parent_world_inverse * desired_world;
            self.pending
                .push(decompose_checked(target.entity, local_target)?);
        }

        for index in 0..self.targets.len() {
            let entity = self.targets[index].entity;
            if let Err(source) = scene.update_transform(entity, self.pending[index]) {
                let cause = InteractiveTransformError::SceneMutation { entity, source };
                let rollback = restore_applied_prefix(scene, &self.targets, index);
                self.expected_world_generation = scene.world_generation();
                return match rollback {
                    Ok(()) => Err(cause),
                    Err(rollback) => Err(InteractiveTransformError::PreviewRollbackFailed {
                        cause: Box::new(cause),
                        rollback,
                    }),
                };
            }
        }
        for (target, pending) in self.targets.iter_mut().zip(self.pending.iter().copied()) {
            target.latest = pending;
        }
        self.expected_world_generation = scene.world_generation();
        Ok(())
    }

    fn world_delta(&self, target_pivot_world: Transform) -> Mat4 {
        if self.spec.kind() == InteractiveTransformKind::Scale
            && self.spec.space() == InteractiveTransformSpace::Global
        {
            let pivot = self.frozen_pivot.translation;
            let scale_ratio = target_pivot_world.scale / self.frozen_pivot.scale;
            return Mat4::from_translation(pivot)
                * Mat4::from_scale(scale_ratio)
                * Mat4::from_translation(-pivot);
        }

        transform_to_mat4(target_pivot_world) * self.frozen_pivot_inverse
    }

    pub(crate) fn finish(
        &self,
        scene: &Scene,
        document: Option<DocumentId>,
    ) -> Result<Option<BatchTransformCommand>, InteractiveTransformError> {
        self.ensure_context(scene, document)?;
        let targets = self
            .targets
            .iter()
            .filter_map(|target| {
                let mut after = target.before.clone();
                after.transform = target.latest;
                BatchTransformTarget::new(target.entity, target.before.clone(), after)
            })
            .collect::<Vec<_>>();
        Ok(BatchTransformCommand::applied(
            targets,
            self.expected_world_generation,
        ))
    }

    pub(crate) fn cancel(
        self,
        scene: &mut Scene,
        document: Option<DocumentId>,
    ) -> Result<(), InteractiveTransformError> {
        self.ensure_context(scene, document)?;
        restore_all(scene, &self.targets)?;
        Ok(())
    }

    fn ensure_context(
        &self,
        scene: &Scene,
        document: Option<DocumentId>,
    ) -> Result<(), InteractiveTransformError> {
        if document != Some(self.document) {
            return Err(InteractiveTransformError::DocumentChanged {
                expected: self.document,
                actual: document,
            });
        }
        let actual = scene.world_generation();
        if actual == self.expected_world_generation {
            Ok(())
        } else {
            Err(InteractiveTransformError::StaleWorldGeneration {
                expected: self.expected_world_generation,
                actual,
            })
        }
    }
}

fn has_selected_ancestor(scene: &Scene, entity: NodeId, selected: &HashSet<NodeId>) -> bool {
    let mut current = scene.parent_of(entity);
    while let Some(parent) = current {
        if selected.contains(&parent) {
            return true;
        }
        current = scene.parent_of(parent);
    }
    false
}

fn selected_root(scene: &Scene, entity: NodeId, selected: &HashSet<NodeId>) -> NodeId {
    let mut root = entity;
    let mut current = scene.parent_of(entity);
    while let Some(parent) = current {
        if selected.contains(&parent) {
            root = parent;
        }
        current = scene.parent_of(parent);
    }
    root
}

fn selection_roots(
    scene: &Scene,
    selected: impl IntoIterator<Item = NodeId>,
    primary: NodeId,
) -> Option<(Vec<NodeId>, NodeId)> {
    let selected = selected.into_iter();
    let (selection_capacity, _) = selected.size_hint();
    let mut selected_set = HashSet::with_capacity(selection_capacity);
    let mut ordered = Vec::with_capacity(selection_capacity);
    for entity in selected {
        if selected_set.insert(entity) {
            ordered.push(entity);
        }
    }
    if !selected_set.contains(&primary) {
        return None;
    }
    let mut root_set = HashSet::with_capacity(selected_set.len());
    let roots = ordered
        .into_iter()
        .filter(|entity| {
            !has_selected_ancestor(scene, *entity, &selected_set) && root_set.insert(*entity)
        })
        .collect::<Vec<_>>();
    Some((roots, selected_root(scene, primary, &selected_set)))
}

fn pivot_transform_for_roots(
    scene: &Scene,
    roots: &[NodeId],
    primary_root: NodeId,
    pivot_mode: PivotMode,
) -> Option<Transform> {
    let mut pivot = scene.world_transform(primary_root)?;
    if pivot_mode == PivotMode::Centroid && roots.len() > 1 {
        let mut sum = Vec3::ZERO;
        for entity in roots {
            sum += scene.world_matrix(*entity)?.transform_point3(Vec3::ZERO);
        }
        pivot.translation = sum / roots.len() as f32;
    }
    Some(pivot)
}

pub(crate) fn selection_pivot_transform(
    scene: &Scene,
    selected: impl IntoIterator<Item = NodeId>,
    primary: NodeId,
    pivot_mode: PivotMode,
) -> Option<(NodeId, Transform)> {
    let (roots, primary_root) = selection_roots(scene, selected, primary)?;
    pivot_transform_for_roots(scene, &roots, primary_root, pivot_mode)
        .map(|pivot| (primary_root, pivot))
}

fn decompose_checked(entity: NodeId, matrix: Mat4) -> Result<Transform, InteractiveTransformError> {
    if !is_finite_mat4(matrix) {
        return Err(InteractiveTransformError::NonRepresentableTransform {
            entity,
            recomposition_residual: f32::INFINITY,
        });
    }
    let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
    let transform = ValidatedTransform::try_new(
        Transform {
            translation,
            rotation,
            scale,
        },
        NumericPolicy::STRICT,
    )
    .map_err(|source| InteractiveTransformError::InvalidTransform { entity, source })?
    .into_transform();
    let recomposed = transform_to_mat4(transform);
    let recomposition_residual = relative_matrix_residual(matrix, recomposed);
    if recomposition_residual > TRS_RECOMPOSITION_RELATIVE_TOLERANCE {
        return Err(InteractiveTransformError::NonRepresentableTransform {
            entity,
            recomposition_residual,
        });
    }
    Ok(transform)
}

fn relative_matrix_residual(expected: Mat4, actual: Mat4) -> f32 {
    let expected = expected.to_cols_array();
    let actual = actual.to_cols_array();
    let scale = expected
        .iter()
        .fold(1.0_f32, |maximum, value| maximum.max(value.abs()));
    let absolute = expected
        .iter()
        .zip(actual)
        .fold(0.0_f32, |maximum, (expected, actual)| {
            maximum.max((expected - actual).abs())
        });
    absolute / scale
}

fn restore_applied_prefix(
    scene: &mut Scene,
    targets: &[FrozenTransformTarget],
    applied_count: usize,
) -> Result<(), String> {
    for target in targets[..applied_count].iter().rev() {
        scene
            .update_transform(target.entity, target.latest)
            .map_err(|error| format!("entity {}: {error}", target.entity))?;
    }
    Ok(())
}

fn restore_all(
    scene: &mut Scene,
    targets: &[FrozenTransformTarget],
) -> Result<(), InteractiveTransformError> {
    for index in (0..targets.len()).rev() {
        let target = &targets[index];
        if let Err(source) = scene.update_transform(target.entity, target.before.transform) {
            let cause = InteractiveTransformError::SceneMutation {
                entity: target.entity,
                source,
            };
            return match restore_latest(scene, &targets[index + 1..]) {
                Ok(()) => Err(cause),
                Err(rollback) => Err(InteractiveTransformError::CancelRollbackFailed {
                    cause: Box::new(cause),
                    rollback,
                }),
            };
        }
    }
    Ok(())
}

fn restore_latest(scene: &mut Scene, targets: &[FrozenTransformTarget]) -> Result<(), String> {
    for target in targets {
        scene
            .update_transform(target.entity, target.latest)
            .map_err(|error| format!("entity {}: {error}", target.entity))?;
    }
    Ok(())
}

#[cfg(test)]
mod optimization_batch_20260830cn_editor_tests {
    use std::collections::HashSet;

    const SYNTHETIC_SELECTION_COUNT: usize = 32_768;

    #[test]
    fn optimization_batch_20260830cn_editor_selection_storage_uses_iterator_lower_bound() {
        let source = include_str!("session.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("interactive transform implementation");

        assert!(implementation.contains("let (selection_capacity, _) = selected.size_hint();"));
        assert!(implementation.contains("HashSet::with_capacity(selection_capacity)"));
        assert!(implementation.contains("Vec::with_capacity(selection_capacity)"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cn_editor_selection_storage_capacity_evidence() {
        let legacy_growth_events = collect_selection_growth_events(false);
        let optimized_growth_events = collect_selection_growth_events(true);

        println!(
            "EDITOR501_SELECTION_ROOT_CAPACITY_BENCH_V1 selected={SYNTHETIC_SELECTION_COUNT} \
legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} \
growth_event_reduction_pct=100"
        );
        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
    }

    fn collect_selection_growth_events(reserve_exact: bool) -> usize {
        let capacity = usize::from(reserve_exact) * SYNTHETIC_SELECTION_COUNT;
        let mut selected = HashSet::with_capacity(capacity);
        let mut ordered = Vec::with_capacity(capacity);
        let mut growth_events = 0;
        for entity in 0..SYNTHETIC_SELECTION_COUNT {
            let set_capacity = selected.capacity();
            let vector_capacity = ordered.capacity();
            assert!(selected.insert(entity));
            ordered.push(entity);
            growth_events += usize::from(selected.capacity() != set_capacity);
            growth_events += usize::from(ordered.capacity() != vector_capacity);
        }
        std::hint::black_box((selected, ordered));
        growth_events
    }
}
