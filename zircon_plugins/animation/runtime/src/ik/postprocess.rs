use std::collections::BTreeMap;

use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::core::framework::animation::AnimationSkeletonAsset;
use zircon_runtime::core::framework::animation::{
    AnimationIkCommand, AnimationLookAtCommand, AnimationPoseOutput, AnimationTwoBoneIkCommand,
};
use zircon_runtime::core::math::{Mat4, Quat, Real, Vec3};
use zircon_runtime::scene::EntityId;

use crate::evaluation::TargetSlot;
use crate::{AnimationEvaluationPipeline, SkeletonTargetTable};

use super::{AnimationIkDiagnostic, AnimationIkExecutionError, LookAtJob, TwoBoneIkJob};

#[derive(Clone, Copy, Debug, PartialEq)]
struct ModelBone {
    matrix: Mat4,
    position: Vec3,
    rotation: Quat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ModelBoneState {
    #[default]
    Unresolved,
    Visiting,
    Resolved,
}

#[derive(Default)]
struct ModelPoseScratch {
    model: Vec<ModelBone>,
    states: Vec<ModelBoneState>,
}

impl ModelPoseScratch {
    fn prepare(&mut self, bone_count: usize) {
        self.model.resize(
            bone_count,
            ModelBone {
                matrix: Mat4::IDENTITY,
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            },
        );
        self.states.resize(bone_count, ModelBoneState::Unresolved);
        self.states.fill(ModelBoneState::Unresolved);
    }
}

#[derive(Clone, Copy)]
struct CompiledTwoBoneIkJob {
    root: TargetSlot,
    mid: TargetSlot,
    tip: TargetSlot,
    target: Vec3,
    pole: Option<Vec3>,
    weight: Real,
}

#[derive(Clone, Copy)]
struct CompiledLookAtJob {
    bone: TargetSlot,
    target: Vec3,
    axis: Vec3,
    clamp_degrees: Real,
    weight: Real,
}

pub(crate) fn apply_ik_commands(
    pipeline: &AnimationEvaluationPipeline,
    assets: &ProjectAssetManager,
    skeletons_by_entity: &BTreeMap<EntityId, AssetId>,
    commands: Vec<AnimationIkCommand>,
    poses: &mut BTreeMap<EntityId, AnimationPoseOutput>,
) -> Vec<AnimationIkDiagnostic> {
    let mut model_pose_scratch = ModelPoseScratch::default();
    commands
        .into_iter()
        .filter_map(|command| {
            execute_command(
                pipeline,
                assets,
                skeletons_by_entity,
                poses,
                &command,
                &mut model_pose_scratch,
            )
            .err()
            .map(|error| AnimationIkDiagnostic {
                entity: command.entity(),
                skeleton: skeletons_by_entity.get(&command.entity()).copied(),
                error,
            })
        })
        .collect()
}

fn execute_command(
    pipeline: &AnimationEvaluationPipeline,
    assets: &ProjectAssetManager,
    skeletons_by_entity: &BTreeMap<EntityId, AssetId>,
    poses: &mut BTreeMap<EntityId, AnimationPoseOutput>,
    command: &AnimationIkCommand,
    model_pose_scratch: &mut ModelPoseScratch,
) -> Result<(), AnimationIkExecutionError> {
    let entity = command.entity();
    let skeleton_id = skeletons_by_entity
        .get(&entity)
        .copied()
        .ok_or(AnimationIkExecutionError::MissingSkeletonBinding)?;
    let skeleton = assets
        .load_animation_skeleton_asset(skeleton_id)
        .map_err(|_| AnimationIkExecutionError::MissingSkeletonAsset {
            skeleton: skeleton_id,
        })?;
    let targets = pipeline.skeleton_target_table(skeleton_id).ok_or(
        AnimationIkExecutionError::MissingCompiledTargets {
            skeleton: skeleton_id,
        },
    )?;
    let pose = poses
        .get_mut(&entity)
        .ok_or(AnimationIkExecutionError::MissingPose)?;
    validate_pose_shape(&skeleton, pose)?;

    match command {
        AnimationIkCommand::TwoBone(command) => apply_two_bone(
            &skeleton,
            &targets,
            pose,
            compile_two_bone(&targets, command)?,
            model_pose_scratch,
        ),
        AnimationIkCommand::LookAt(command) => apply_look_at(
            &skeleton,
            &targets,
            pose,
            compile_look_at(&targets, command)?,
            model_pose_scratch,
        ),
    }
}

fn compile_two_bone(
    targets: &SkeletonTargetTable,
    command: &AnimationTwoBoneIkCommand,
) -> Result<CompiledTwoBoneIkJob, AnimationIkExecutionError> {
    Ok(CompiledTwoBoneIkJob {
        root: resolve_target(targets, command.root)?,
        mid: resolve_target(targets, command.mid)?,
        tip: resolve_target(targets, command.tip)?,
        target: command.target,
        pole: command.pole,
        weight: command.weight,
    })
}

fn compile_look_at(
    targets: &SkeletonTargetTable,
    command: &AnimationLookAtCommand,
) -> Result<CompiledLookAtJob, AnimationIkExecutionError> {
    Ok(CompiledLookAtJob {
        bone: resolve_target(targets, command.bone)?,
        target: command.target,
        axis: command.axis,
        clamp_degrees: command.clamp_degrees,
        weight: command.weight,
    })
}

fn resolve_target(
    targets: &SkeletonTargetTable,
    target: zircon_runtime::core::framework::animation::AnimationTargetId,
) -> Result<TargetSlot, AnimationIkExecutionError> {
    targets
        .slot_for_target(target)
        .ok_or(AnimationIkExecutionError::UnresolvedTarget { target })
}

fn apply_two_bone(
    skeleton: &AnimationSkeletonAsset,
    targets: &SkeletonTargetTable,
    pose: &mut AnimationPoseOutput,
    job: CompiledTwoBoneIkJob,
    model_pose_scratch: &mut ModelPoseScratch,
) -> Result<(), AnimationIkExecutionError> {
    let root = targets
        .bone_index_for_slot(job.root)
        .ok_or(AnimationIkExecutionError::InvalidTwoBoneChain)?;
    let mid = targets
        .bone_index_for_slot(job.mid)
        .ok_or(AnimationIkExecutionError::InvalidTwoBoneChain)?;
    let tip = targets
        .bone_index_for_slot(job.tip)
        .ok_or(AnimationIkExecutionError::InvalidTwoBoneChain)?;
    if skeleton.bones[mid].parent_index != Some(root as u32)
        || skeleton.bones[tip].parent_index != Some(mid as u32)
    {
        return Err(AnimationIkExecutionError::InvalidTwoBoneChain);
    }

    let (solved, solved_root_local) = {
        let model = model_pose(skeleton, pose, model_pose_scratch)?;
        let mut solver = TwoBoneIkJob::new(job.target).with_weight(job.weight);
        if let Some(pole) = job.pole {
            solver = solver.with_pole(pole - model[root].position);
        }
        let solved = solver
            .solve_positions(
                model[root].position,
                model[mid].position,
                model[tip].position,
            )
            .map_err(AnimationIkExecutionError::Solver)?;
        let root_delta = rotation_arc(
            model[mid].position - model[root].position,
            solved.mid - solved.root,
        )?;
        let solved_root_model = (root_delta * model[root].rotation).normalize();
        (
            solved,
            local_rotation(skeleton, model, root, solved_root_model),
        )
    };
    pose.bones[root].local_transform.rotation = solved_root_local;

    let solved_mid_local = {
        let model = model_pose(skeleton, pose, model_pose_scratch)?;
        let mid_delta = rotation_arc(
            model[tip].position - model[mid].position,
            solved.tip - model[mid].position,
        )?;
        let solved_mid_model = (mid_delta * model[mid].rotation).normalize();
        local_rotation(skeleton, model, mid, solved_mid_model)
    };
    pose.bones[mid].local_transform.rotation = solved_mid_local;
    Ok(())
}

fn apply_look_at(
    skeleton: &AnimationSkeletonAsset,
    targets: &SkeletonTargetTable,
    pose: &mut AnimationPoseOutput,
    job: CompiledLookAtJob,
    model_pose_scratch: &mut ModelPoseScratch,
) -> Result<(), AnimationIkExecutionError> {
    let bone = targets
        .bone_index_for_slot(job.bone)
        .ok_or(AnimationIkExecutionError::InvalidSkeletonHierarchy)?;
    let solved_local = {
        let model = model_pose(skeleton, pose, model_pose_scratch)?;
        let direction = job.target - model[bone].position;
        let solved_model = LookAtJob::new(direction, job.axis)
            .with_clamp_degrees(job.clamp_degrees)
            .with_weight(job.weight)
            .solve_rotation(model[bone].rotation)
            .map_err(AnimationIkExecutionError::Solver)?;
        local_rotation(skeleton, model, bone, solved_model)
    };
    pose.bones[bone].local_transform.rotation = solved_local;
    Ok(())
}

fn rotation_arc(from: Vec3, to: Vec3) -> Result<Quat, AnimationIkExecutionError> {
    let from = from
        .try_normalize()
        .ok_or(AnimationIkExecutionError::Solver(
            super::AnimationIkError::DegenerateChain,
        ))?;
    let to = to.try_normalize().ok_or(AnimationIkExecutionError::Solver(
        super::AnimationIkError::DegenerateChain,
    ))?;
    Ok(Quat::from_rotation_arc(from, to))
}

fn local_rotation(
    skeleton: &AnimationSkeletonAsset,
    model: &[ModelBone],
    bone: usize,
    model_rotation: Quat,
) -> Quat {
    skeleton.bones[bone]
        .parent_index
        .map(|parent| model[parent as usize].rotation.inverse() * model_rotation)
        .unwrap_or(model_rotation)
        .normalize()
}

fn validate_pose_shape(
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
) -> Result<(), AnimationIkExecutionError> {
    if pose.bones.len() != skeleton.bones.len() {
        return Err(AnimationIkExecutionError::PoseShapeMismatch {
            expected: skeleton.bones.len(),
            actual: pose.bones.len(),
        });
    }
    Ok(())
}

fn model_pose<'scratch>(
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
    scratch: &'scratch mut ModelPoseScratch,
) -> Result<&'scratch [ModelBone], AnimationIkExecutionError> {
    scratch.prepare(skeleton.bones.len());
    for bone in 0..skeleton.bones.len() {
        resolve_model_bone(skeleton, pose, bone, scratch)?;
    }
    Ok(&scratch.model)
}

fn resolve_model_bone(
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
    bone: usize,
    scratch: &mut ModelPoseScratch,
) -> Result<ModelBone, AnimationIkExecutionError> {
    if bone >= skeleton.bones.len() || bone >= pose.bones.len() {
        return Err(AnimationIkExecutionError::InvalidSkeletonHierarchy);
    }
    match scratch.states[bone] {
        ModelBoneState::Resolved => return Ok(scratch.model[bone]),
        ModelBoneState::Visiting => {
            return Err(AnimationIkExecutionError::InvalidSkeletonHierarchy);
        }
        ModelBoneState::Unresolved => {}
    }
    scratch.states[bone] = ModelBoneState::Visiting;
    let local = pose.bones[bone].local_transform;
    let (parent_matrix, parent_rotation) = match skeleton.bones[bone].parent_index {
        Some(parent) => {
            let parent = resolve_model_bone(skeleton, pose, parent as usize, scratch)?;
            (parent.matrix, parent.rotation)
        }
        None => (Mat4::IDENTITY, Quat::IDENTITY),
    };
    let matrix = parent_matrix * local.matrix();
    let resolved = ModelBone {
        matrix,
        position: matrix.transform_point3(Vec3::ZERO),
        rotation: (parent_rotation * local.rotation).normalize(),
    };
    scratch.model[bone] = resolved;
    scratch.states[bone] = ModelBoneState::Resolved;
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::framework::animation::{
        AnimationPoseBone, AnimationPoseSource, AnimationSkeletonBoneAsset, AnimationTargetId,
    };
    use zircon_runtime::core::framework::scene::WorldHandle;
    use zircon_runtime::core::math::Transform;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS_PER_SAMPLE: usize = 16_384;

    #[test]
    fn reusable_model_pose_scratch_matches_allocating_hierarchy_evaluation() {
        let (skeleton, pose) = chain_fixture();
        let expected = allocating_model_pose(&skeleton, &pose).unwrap();
        let mut scratch = ModelPoseScratch::default();

        let actual = model_pose(&skeleton, &pose, &mut scratch).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn reusable_model_pose_scratch_retains_buffers_for_stable_topology() {
        let (skeleton, pose) = chain_fixture();
        let mut scratch = ModelPoseScratch::default();
        black_box(model_pose(&skeleton, &pose, &mut scratch).unwrap());
        let model_ptr = scratch.model.as_ptr();
        let model_capacity = scratch.model.capacity();
        let state_ptr = scratch.states.as_ptr();
        let state_capacity = scratch.states.capacity();

        black_box(model_pose(&skeleton, &pose, &mut scratch).unwrap());

        assert_eq!(scratch.model.as_ptr(), model_ptr);
        assert_eq!(scratch.model.capacity(), model_capacity);
        assert_eq!(scratch.states.as_ptr(), state_ptr);
        assert_eq!(scratch.states.capacity(), state_capacity);
    }

    #[test]
    fn reusable_model_pose_scratch_resets_traversal_state_after_error() {
        let (mut skeleton, pose) = chain_fixture();
        let mut scratch = ModelPoseScratch::default();
        black_box(model_pose(&skeleton, &pose, &mut scratch).unwrap());
        skeleton.bones[0].parent_index = Some(2);

        assert_eq!(
            model_pose(&skeleton, &pose, &mut scratch),
            Err(AnimationIkExecutionError::InvalidSkeletonHierarchy)
        );

        skeleton.bones[0].parent_index = None;
        assert!(model_pose(&skeleton, &pose, &mut scratch).is_ok());
    }

    #[test]
    fn ik_model_pose_scratch_two_bone_path_rebuilds_after_root_write_and_recovers_after_failed_command()
     {
        let (skeleton, mut pose) = two_bone_fixture();
        let targets = SkeletonTargetTable::compile(&skeleton).unwrap();
        let target = Vec3::new(1.2, 0.8, 0.0);
        let job = compile_two_bone(
            &targets,
            &AnimationTwoBoneIkCommand {
                world: WorldHandle::new(1),
                entity: 7,
                root: AnimationTargetId::from_segments(["Root"]),
                mid: AnimationTargetId::from_segments(["Root", "Mid"]),
                tip: AnimationTargetId::from_segments(["Root", "Mid", "Tip"]),
                target,
                pole: Some(Vec3::Z),
                weight: 1.0,
            },
        )
        .unwrap();
        let mut scratch = ModelPoseScratch::default();
        let mut invalid_skeleton = skeleton.clone();
        invalid_skeleton.bones.push(skeleton_bone("Cycle", Some(3)));
        let mut invalid_pose = pose.clone();
        invalid_pose.bones.push(pose_bone("Cycle", Vec3::ZERO, 0.0));

        assert_eq!(
            apply_two_bone(
                &invalid_skeleton,
                &targets,
                &mut invalid_pose,
                job,
                &mut scratch,
            ),
            Err(AnimationIkExecutionError::InvalidSkeletonHierarchy)
        );

        apply_two_bone(&skeleton, &targets, &mut pose, job, &mut scratch).unwrap();
        let solved_tip = model_pose(&skeleton, &pose, &mut scratch).unwrap()[2].position;
        assert!(
            solved_tip.abs_diff_eq(target, 1.0e-4),
            "expected {target:?}, got {solved_tip:?}"
        );
    }

    #[test]
    #[ignore = "managed Runtime08C IK model-pose scratch release performance gate"]
    fn ik_model_pose_scratch_release_benchmark_evidence() {
        let (skeleton, pose) = chain_fixture();
        let mut allocating_us = Vec::with_capacity(SAMPLE_PAIRS);
        let mut scratch_us = Vec::with_capacity(SAMPLE_PAIRS);
        let mut scratch = ModelPoseScratch::default();

        black_box(allocating_model_pose(&skeleton, &pose).unwrap());
        black_box(model_pose(&skeleton, &pose, &mut scratch).unwrap());
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                allocating_us.push(measure_allocating(&skeleton, &pose));
                scratch_us.push(measure_scratch(&skeleton, &pose, &mut scratch));
            } else {
                scratch_us.push(measure_scratch(&skeleton, &pose, &mut scratch));
                allocating_us.push(measure_allocating(&skeleton, &pose));
            }
        }

        let allocating_p50_us = nearest_rank(&allocating_us, 50);
        let allocating_p95_us = nearest_rank(&allocating_us, 95);
        let scratch_p50_us = nearest_rank(&scratch_us, 50);
        let scratch_p95_us = nearest_rank(&scratch_us, 95);
        let p95_ratio = scratch_p95_us as f64 / allocating_p95_us as f64;
        println!(
            "IK_MODEL_POSE_SCRATCH_BENCH_V1 bone_count={} iterations_per_sample={} sample_pairs={} sample_order=alternating percentile_method=nearest_rank allocating_p50_us={} allocating_p95_us={} scratch_p50_us={} scratch_p95_us={} p95_ratio={:.6} allocating_us={} scratch_us={}",
            skeleton.bones.len(),
            ITERATIONS_PER_SAMPLE,
            SAMPLE_PAIRS,
            allocating_p50_us,
            allocating_p95_us,
            scratch_p50_us,
            scratch_p95_us,
            p95_ratio,
            sample_csv(&allocating_us),
            sample_csv(&scratch_us),
        );
        assert!(
            scratch_p95_us.saturating_mul(4) <= allocating_p95_us.saturating_mul(3),
            "scratch P95 {scratch_p95_us}us must be at most 75% of allocating P95 {allocating_p95_us}us"
        );
    }

    fn measure_allocating(skeleton: &AnimationSkeletonAsset, pose: &AnimationPoseOutput) -> u128 {
        let started = Instant::now();
        for _ in 0..ITERATIONS_PER_SAMPLE {
            black_box(allocating_model_pose(skeleton, pose).unwrap());
        }
        started.elapsed().as_micros().max(1)
    }

    fn measure_scratch(
        skeleton: &AnimationSkeletonAsset,
        pose: &AnimationPoseOutput,
        scratch: &mut ModelPoseScratch,
    ) -> u128 {
        let started = Instant::now();
        for _ in 0..ITERATIONS_PER_SAMPLE {
            black_box(model_pose(skeleton, pose, scratch).unwrap());
        }
        started.elapsed().as_micros().max(1)
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn chain_fixture() -> (AnimationSkeletonAsset, AnimationPoseOutput) {
        let skeleton = AnimationSkeletonAsset {
            name: Some("IkScratchChain".to_string()),
            bones: vec![
                skeleton_bone("Root", None),
                skeleton_bone("Mid", Some(0)),
                skeleton_bone("Tip", Some(1)),
            ],
        };
        let pose = AnimationPoseOutput {
            source: AnimationPoseSource::Graph,
            active_state: None,
            bones: vec![
                pose_bone("Root", Vec3::new(0.5, 0.0, 0.0), 0.15),
                pose_bone("Mid", Vec3::new(1.0, 0.25, 0.0), -0.2),
                pose_bone("Tip", Vec3::new(0.75, 0.5, 0.1), 0.35),
            ],
        };
        (skeleton, pose)
    }

    fn two_bone_fixture() -> (AnimationSkeletonAsset, AnimationPoseOutput) {
        let (skeleton, mut pose) = chain_fixture();
        pose.bones[0].local_transform = Transform::default();
        pose.bones[1].local_transform = Transform {
            translation: Vec3::X,
            ..Transform::default()
        };
        pose.bones[2].local_transform = Transform {
            translation: Vec3::X,
            ..Transform::default()
        };
        (skeleton, pose)
    }

    fn skeleton_bone(name: &str, parent_index: Option<u32>) -> AnimationSkeletonBoneAsset {
        AnimationSkeletonBoneAsset {
            name: name.to_string(),
            parent_index,
            local_translation: [0.0, 0.0, 0.0],
            local_rotation: [0.0, 0.0, 0.0, 1.0],
            local_scale: [1.0, 1.0, 1.0],
        }
    }

    fn pose_bone(name: &str, translation: Vec3, rotation_radians: Real) -> AnimationPoseBone {
        AnimationPoseBone {
            name: name.to_string(),
            local_transform: Transform {
                translation,
                rotation: Quat::from_rotation_z(rotation_radians),
                scale: Vec3::ONE,
            },
        }
    }

    fn allocating_model_pose(
        skeleton: &AnimationSkeletonAsset,
        pose: &AnimationPoseOutput,
    ) -> Result<Vec<ModelBone>, AnimationIkExecutionError> {
        let mut model = vec![None; skeleton.bones.len()];
        let mut visiting = vec![false; skeleton.bones.len()];
        for bone in 0..skeleton.bones.len() {
            resolve_allocating_model_bone(skeleton, pose, bone, &mut model, &mut visiting)?;
        }
        model
            .into_iter()
            .map(|bone| bone.ok_or(AnimationIkExecutionError::InvalidSkeletonHierarchy))
            .collect()
    }

    fn resolve_allocating_model_bone(
        skeleton: &AnimationSkeletonAsset,
        pose: &AnimationPoseOutput,
        bone: usize,
        model: &mut [Option<ModelBone>],
        visiting: &mut [bool],
    ) -> Result<ModelBone, AnimationIkExecutionError> {
        if let Some(model) = model.get(bone).copied().flatten() {
            return Ok(model);
        }
        if bone >= skeleton.bones.len() || visiting[bone] {
            return Err(AnimationIkExecutionError::InvalidSkeletonHierarchy);
        }
        visiting[bone] = true;
        let local = pose
            .bones
            .get(bone)
            .ok_or(AnimationIkExecutionError::InvalidSkeletonHierarchy)?
            .local_transform;
        let (parent_matrix, parent_rotation) = match skeleton.bones[bone].parent_index {
            Some(parent) => {
                let parent = resolve_allocating_model_bone(
                    skeleton,
                    pose,
                    parent as usize,
                    model,
                    visiting,
                )?;
                (parent.matrix, parent.rotation)
            }
            None => (Mat4::IDENTITY, Quat::IDENTITY),
        };
        let matrix = parent_matrix * local.matrix();
        let resolved = ModelBone {
            matrix,
            position: matrix.transform_point3(Vec3::ZERO),
            rotation: (parent_rotation * local.rotation).normalize(),
        };
        visiting[bone] = false;
        model[bone] = Some(resolved);
        Ok(resolved)
    }
}
