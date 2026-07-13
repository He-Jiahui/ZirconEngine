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

#[derive(Clone, Copy)]
struct ModelBone {
    matrix: Mat4,
    position: Vec3,
    rotation: Quat,
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
    commands
        .into_iter()
        .filter_map(|command| {
            execute_command(pipeline, assets, skeletons_by_entity, poses, &command)
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
        ),
        AnimationIkCommand::LookAt(command) => apply_look_at(
            &skeleton,
            &targets,
            pose,
            compile_look_at(&targets, command)?,
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

    let model = model_pose(skeleton, pose)?;
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
    pose.bones[root].local_transform.rotation =
        local_rotation(skeleton, &model, root, solved_root_model);

    let model = model_pose(skeleton, pose)?;
    let mid_delta = rotation_arc(
        model[tip].position - model[mid].position,
        solved.tip - model[mid].position,
    )?;
    let solved_mid_model = (mid_delta * model[mid].rotation).normalize();
    pose.bones[mid].local_transform.rotation =
        local_rotation(skeleton, &model, mid, solved_mid_model);
    Ok(())
}

fn apply_look_at(
    skeleton: &AnimationSkeletonAsset,
    targets: &SkeletonTargetTable,
    pose: &mut AnimationPoseOutput,
    job: CompiledLookAtJob,
) -> Result<(), AnimationIkExecutionError> {
    let bone = targets
        .bone_index_for_slot(job.bone)
        .ok_or(AnimationIkExecutionError::InvalidSkeletonHierarchy)?;
    let model = model_pose(skeleton, pose)?;
    let direction = job.target - model[bone].position;
    let solved_model = LookAtJob::new(direction, job.axis)
        .with_clamp_degrees(job.clamp_degrees)
        .with_weight(job.weight)
        .solve_rotation(model[bone].rotation)
        .map_err(AnimationIkExecutionError::Solver)?;
    pose.bones[bone].local_transform.rotation =
        local_rotation(skeleton, &model, bone, solved_model);
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

fn model_pose(
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
) -> Result<Vec<ModelBone>, AnimationIkExecutionError> {
    let mut model = vec![None; skeleton.bones.len()];
    let mut visiting = vec![false; skeleton.bones.len()];
    for bone in 0..skeleton.bones.len() {
        resolve_model_bone(skeleton, pose, bone, &mut model, &mut visiting)?;
    }
    model
        .into_iter()
        .map(|bone| bone.ok_or(AnimationIkExecutionError::InvalidSkeletonHierarchy))
        .collect()
}

fn resolve_model_bone(
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
            let parent = resolve_model_bone(skeleton, pose, parent as usize, model, visiting)?;
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
