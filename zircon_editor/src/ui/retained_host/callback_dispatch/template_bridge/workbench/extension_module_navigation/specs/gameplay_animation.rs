use super::types::{action, spec, ActionControl, ExtensionNavigationSpec};

const SEQUENCER_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionSequencerSequenceRow",
    "WorkbenchExtensionSequencerCameraRow",
    "WorkbenchExtensionSequencerEventRow",
    "WorkbenchExtensionSequencerCameraCutTableRow",
    "WorkbenchExtensionSequencerHeroTransformTableRow",
    "WorkbenchExtensionSequencerAudioTableRow",
    "WorkbenchExtensionSequencerEventCueTableRow",
];
const SEQUENCER_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.sequencer.sequence_row.select",
        "WorkbenchExtensionSequencerSequenceRow",
    ),
    action(
        "workbench.extension.sequencer.camera_row.select",
        "WorkbenchExtensionSequencerCameraRow",
    ),
    action(
        "workbench.extension.sequencer.event_row.select",
        "WorkbenchExtensionSequencerEventRow",
    ),
    action(
        "workbench.extension.sequencer.camera_cut_table_row.select",
        "WorkbenchExtensionSequencerCameraCutTableRow",
    ),
    action(
        "workbench.extension.sequencer.hero_transform_table_row.select",
        "WorkbenchExtensionSequencerHeroTransformTableRow",
    ),
    action(
        "workbench.extension.sequencer.audio_table_row.select",
        "WorkbenchExtensionSequencerAudioTableRow",
    ),
    action(
        "workbench.extension.sequencer.event_cue_table_row.select",
        "WorkbenchExtensionSequencerEventCueTableRow",
    ),
];
const SEQUENCER_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAbilityAnimationToolsMenu",
    "WorkbenchExtensionSequencerPreviewButton",
    "WorkbenchExtensionSequencerValidateButton",
];
const SEQUENCER_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.sequencer.open",
        "WorkbenchAbilityAnimationToolsMenu",
    ),
    action(
        "workbench.extension.sequencer.preview.invoke",
        "WorkbenchExtensionSequencerPreviewButton",
    ),
    action(
        "workbench.extension.sequencer.validate.invoke",
        "WorkbenchExtensionSequencerValidateButton",
    ),
];
const SEQUENCER_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.sequencer.sequence.edit",
    "workbench.extension.sequencer.sequence.commit",
    "workbench.extension.sequencer.frame_rate.edit",
    "workbench.extension.sequencer.frame_rate.commit",
    "workbench.extension.sequencer.work_range.edit",
    "workbench.extension.sequencer.work_range.commit",
];

pub(super) const SEQUENCER_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.sequencer.open",
    "WorkbenchExtensionSequencerWorkspace",
    SEQUENCER_ROW_CONTROLS,
    SEQUENCER_ROW_ACTIONS,
    SEQUENCER_COMMAND_CONTROLS,
    SEQUENCER_COMMAND_ACTIONS,
    SEQUENCER_FIELD_ACTIONS,
);

const MONTAGE_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionMontageEditorAttackRow",
    "WorkbenchExtensionMontageEditorComboRow",
    "WorkbenchExtensionMontageEditorCancelWindowRow",
    "WorkbenchExtensionMontageEditorIntroTableRow",
    "WorkbenchExtensionMontageEditorComboTableRow",
    "WorkbenchExtensionMontageEditorNotifyTableRow",
    "WorkbenchExtensionMontageEditorRootMotionTableRow",
];
const MONTAGE_EDITOR_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.montage_editor.attack_row.select",
        "WorkbenchExtensionMontageEditorAttackRow",
    ),
    action(
        "workbench.extension.montage_editor.combo_row.select",
        "WorkbenchExtensionMontageEditorComboRow",
    ),
    action(
        "workbench.extension.montage_editor.cancel_window_row.select",
        "WorkbenchExtensionMontageEditorCancelWindowRow",
    ),
    action(
        "workbench.extension.montage_editor.intro_table_row.select",
        "WorkbenchExtensionMontageEditorIntroTableRow",
    ),
    action(
        "workbench.extension.montage_editor.combo_table_row.select",
        "WorkbenchExtensionMontageEditorComboTableRow",
    ),
    action(
        "workbench.extension.montage_editor.notify_table_row.select",
        "WorkbenchExtensionMontageEditorNotifyTableRow",
    ),
    action(
        "workbench.extension.montage_editor.root_motion_table_row.select",
        "WorkbenchExtensionMontageEditorRootMotionTableRow",
    ),
];
const MONTAGE_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAbilityAnimationToolsMenu",
    "WorkbenchExtensionMontageEditorPreviewButton",
    "WorkbenchExtensionMontageEditorApplyButton",
];
const MONTAGE_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.montage_editor.open",
        "WorkbenchAbilityAnimationToolsMenu",
    ),
    action(
        "workbench.extension.montage_editor.preview.invoke",
        "WorkbenchExtensionMontageEditorPreviewButton",
    ),
    action(
        "workbench.extension.montage_editor.apply.invoke",
        "WorkbenchExtensionMontageEditorApplyButton",
    ),
];
const MONTAGE_EDITOR_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.montage_editor.montage.edit",
    "workbench.extension.montage_editor.montage.commit",
    "workbench.extension.montage_editor.slot.edit",
    "workbench.extension.montage_editor.slot.commit",
    "workbench.extension.montage_editor.blend.edit",
    "workbench.extension.montage_editor.blend.commit",
];

pub(super) const MONTAGE_EDITOR_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.montage_editor.open",
    "WorkbenchExtensionMontageEditorWorkspace",
    MONTAGE_EDITOR_ROW_CONTROLS,
    MONTAGE_EDITOR_ROW_ACTIONS,
    MONTAGE_EDITOR_COMMAND_CONTROLS,
    MONTAGE_EDITOR_COMMAND_ACTIONS,
    MONTAGE_EDITOR_FIELD_ACTIONS,
);

const BLEND_SPACE_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionBlendSpaceIdleRunRow",
    "WorkbenchExtensionBlendSpaceStrafeRow",
    "WorkbenchExtensionBlendSpaceSprintRow",
    "WorkbenchExtensionBlendSpaceIdleSampleTableRow",
    "WorkbenchExtensionBlendSpaceWalkSampleTableRow",
    "WorkbenchExtensionBlendSpaceRunSampleTableRow",
    "WorkbenchExtensionBlendSpaceDiagonalSampleTableRow",
];
const BLEND_SPACE_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.blend_space.idle_run_row.select",
        "WorkbenchExtensionBlendSpaceIdleRunRow",
    ),
    action(
        "workbench.extension.blend_space.strafe_row.select",
        "WorkbenchExtensionBlendSpaceStrafeRow",
    ),
    action(
        "workbench.extension.blend_space.sprint_row.select",
        "WorkbenchExtensionBlendSpaceSprintRow",
    ),
    action(
        "workbench.extension.blend_space.idle_sample_table_row.select",
        "WorkbenchExtensionBlendSpaceIdleSampleTableRow",
    ),
    action(
        "workbench.extension.blend_space.walk_sample_table_row.select",
        "WorkbenchExtensionBlendSpaceWalkSampleTableRow",
    ),
    action(
        "workbench.extension.blend_space.run_sample_table_row.select",
        "WorkbenchExtensionBlendSpaceRunSampleTableRow",
    ),
    action(
        "workbench.extension.blend_space.diagonal_sample_table_row.select",
        "WorkbenchExtensionBlendSpaceDiagonalSampleTableRow",
    ),
];
const BLEND_SPACE_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAbilityAnimationToolsMenu",
    "WorkbenchExtensionBlendSpacePreviewButton",
    "WorkbenchExtensionBlendSpaceApplyButton",
    "WorkbenchValidationLogAll",
    "WorkbenchValidationLogErrors",
    "WorkbenchValidationLogWarnings",
    "WorkbenchValidationLogInfos",
    "WorkbenchValidationLogClear",
];
const BLEND_SPACE_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.blend_space.open",
        "WorkbenchAbilityAnimationToolsMenu",
    ),
    action(
        "workbench.extension.blend_space.preview.invoke",
        "WorkbenchExtensionBlendSpacePreviewButton",
    ),
    action(
        "workbench.extension.blend_space.apply.invoke",
        "WorkbenchExtensionBlendSpaceApplyButton",
    ),
    action(
        "workbench.extension.blend_space.validation.filter_all",
        "WorkbenchValidationLogAll",
    ),
    action(
        "workbench.extension.blend_space.validation.filter_errors",
        "WorkbenchValidationLogErrors",
    ),
    action(
        "workbench.extension.blend_space.validation.filter_warnings",
        "WorkbenchValidationLogWarnings",
    ),
    action(
        "workbench.extension.blend_space.validation.filter_infos",
        "WorkbenchValidationLogInfos",
    ),
    action(
        "workbench.extension.blend_space.validation.clear",
        "WorkbenchValidationLogClear",
    ),
];
const BLEND_SPACE_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.blend_space.search.edit",
    "workbench.extension.blend_space.search.commit",
    "workbench.extension.blend_space.asset.edit",
    "workbench.extension.blend_space.asset.commit",
    "workbench.extension.blend_space.x_axis.edit",
    "workbench.extension.blend_space.x_axis.commit",
    "workbench.extension.blend_space.interpolation.edit",
    "workbench.extension.blend_space.interpolation.commit",
];

pub(super) const BLEND_SPACE_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.blend_space.open",
    "WorkbenchExtensionBlendSpaceWorkspace",
    BLEND_SPACE_ROW_CONTROLS,
    BLEND_SPACE_ROW_ACTIONS,
    BLEND_SPACE_COMMAND_CONTROLS,
    BLEND_SPACE_COMMAND_ACTIONS,
    BLEND_SPACE_FIELD_ACTIONS,
);

const POSE_LIBRARY_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionPoseLibraryCombatRow",
    "WorkbenchExtensionPoseLibraryLocomotionRow",
    "WorkbenchExtensionPoseLibraryEmoteRow",
    "WorkbenchExtensionPoseLibraryIdlePoseTableRow",
    "WorkbenchExtensionPoseLibraryAimPoseTableRow",
    "WorkbenchExtensionPoseLibraryCrouchPoseTableRow",
    "WorkbenchExtensionPoseLibraryMirrorPoseTableRow",
];
const POSE_LIBRARY_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.pose_library.combat_row.select",
        "WorkbenchExtensionPoseLibraryCombatRow",
    ),
    action(
        "workbench.extension.pose_library.locomotion_row.select",
        "WorkbenchExtensionPoseLibraryLocomotionRow",
    ),
    action(
        "workbench.extension.pose_library.emote_row.select",
        "WorkbenchExtensionPoseLibraryEmoteRow",
    ),
    action(
        "workbench.extension.pose_library.idle_pose_table_row.select",
        "WorkbenchExtensionPoseLibraryIdlePoseTableRow",
    ),
    action(
        "workbench.extension.pose_library.aim_pose_table_row.select",
        "WorkbenchExtensionPoseLibraryAimPoseTableRow",
    ),
    action(
        "workbench.extension.pose_library.crouch_pose_table_row.select",
        "WorkbenchExtensionPoseLibraryCrouchPoseTableRow",
    ),
    action(
        "workbench.extension.pose_library.mirror_pose_table_row.select",
        "WorkbenchExtensionPoseLibraryMirrorPoseTableRow",
    ),
];
const POSE_LIBRARY_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAbilityAnimationToolsMenu",
    "WorkbenchExtensionPoseLibraryPreviewButton",
    "WorkbenchExtensionPoseLibraryApplyButton",
];
const POSE_LIBRARY_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.pose_library.open",
        "WorkbenchAbilityAnimationToolsMenu",
    ),
    action(
        "workbench.extension.pose_library.preview.invoke",
        "WorkbenchExtensionPoseLibraryPreviewButton",
    ),
    action(
        "workbench.extension.pose_library.apply.invoke",
        "WorkbenchExtensionPoseLibraryApplyButton",
    ),
];
const POSE_LIBRARY_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.pose_library.asset.edit",
    "workbench.extension.pose_library.asset.commit",
    "workbench.extension.pose_library.tag.edit",
    "workbench.extension.pose_library.tag.commit",
    "workbench.extension.pose_library.mirror.edit",
    "workbench.extension.pose_library.mirror.commit",
];

pub(super) const POSE_LIBRARY_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.pose_library.open",
    "WorkbenchExtensionPoseLibraryWorkspace",
    POSE_LIBRARY_ROW_CONTROLS,
    POSE_LIBRARY_ROW_ACTIONS,
    POSE_LIBRARY_COMMAND_CONTROLS,
    POSE_LIBRARY_COMMAND_ACTIONS,
    POSE_LIBRARY_FIELD_ACTIONS,
);

const RETARGET_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionRetargetMannequinRow",
    "WorkbenchExtensionRetargetRobotRow",
    "WorkbenchExtensionRetargetQuadrupedRow",
    "WorkbenchExtensionRetargetRootChainTableRow",
    "WorkbenchExtensionRetargetSpineChainTableRow",
    "WorkbenchExtensionRetargetArmChainTableRow",
    "WorkbenchExtensionRetargetLegChainTableRow",
];
const RETARGET_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.retarget.mannequin_row.select",
        "WorkbenchExtensionRetargetMannequinRow",
    ),
    action(
        "workbench.extension.retarget.robot_row.select",
        "WorkbenchExtensionRetargetRobotRow",
    ),
    action(
        "workbench.extension.retarget.quadruped_row.select",
        "WorkbenchExtensionRetargetQuadrupedRow",
    ),
    action(
        "workbench.extension.retarget.root_chain_table_row.select",
        "WorkbenchExtensionRetargetRootChainTableRow",
    ),
    action(
        "workbench.extension.retarget.spine_chain_table_row.select",
        "WorkbenchExtensionRetargetSpineChainTableRow",
    ),
    action(
        "workbench.extension.retarget.arm_chain_table_row.select",
        "WorkbenchExtensionRetargetArmChainTableRow",
    ),
    action(
        "workbench.extension.retarget.leg_chain_table_row.select",
        "WorkbenchExtensionRetargetLegChainTableRow",
    ),
];
const RETARGET_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAbilityAnimationToolsMenu",
    "WorkbenchExtensionRetargetPreviewButton",
    "WorkbenchExtensionRetargetApplyButton",
];
const RETARGET_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.retarget.open",
        "WorkbenchAbilityAnimationToolsMenu",
    ),
    action(
        "workbench.extension.retarget.preview.invoke",
        "WorkbenchExtensionRetargetPreviewButton",
    ),
    action(
        "workbench.extension.retarget.apply.invoke",
        "WorkbenchExtensionRetargetApplyButton",
    ),
];
const RETARGET_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.retarget.source.edit",
    "workbench.extension.retarget.source.commit",
    "workbench.extension.retarget.target.edit",
    "workbench.extension.retarget.target.commit",
    "workbench.extension.retarget.solver.edit",
    "workbench.extension.retarget.solver.commit",
];

pub(super) const RETARGET_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.retarget.open",
    "WorkbenchExtensionRetargetWorkspace",
    RETARGET_ROW_CONTROLS,
    RETARGET_ROW_ACTIONS,
    RETARGET_COMMAND_CONTROLS,
    RETARGET_COMMAND_ACTIONS,
    RETARGET_FIELD_ACTIONS,
);

const CONTROL_RIG_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionControlRigHeroRow",
    "WorkbenchExtensionControlRigSpineRow",
    "WorkbenchExtensionControlRigHandRow",
    "WorkbenchExtensionControlRigSpineControlTableRow",
    "WorkbenchExtensionControlRigArmIkTableRow",
    "WorkbenchExtensionControlRigHandIkTableRow",
    "WorkbenchExtensionControlRigOutputPoseTableRow",
];
const CONTROL_RIG_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.control_rig.hero_row.select",
        "WorkbenchExtensionControlRigHeroRow",
    ),
    action(
        "workbench.extension.control_rig.spine_row.select",
        "WorkbenchExtensionControlRigSpineRow",
    ),
    action(
        "workbench.extension.control_rig.hand_row.select",
        "WorkbenchExtensionControlRigHandRow",
    ),
    action(
        "workbench.extension.control_rig.spine_control_table_row.select",
        "WorkbenchExtensionControlRigSpineControlTableRow",
    ),
    action(
        "workbench.extension.control_rig.arm_ik_table_row.select",
        "WorkbenchExtensionControlRigArmIkTableRow",
    ),
    action(
        "workbench.extension.control_rig.hand_ik_table_row.select",
        "WorkbenchExtensionControlRigHandIkTableRow",
    ),
    action(
        "workbench.extension.control_rig.output_pose_table_row.select",
        "WorkbenchExtensionControlRigOutputPoseTableRow",
    ),
];
const CONTROL_RIG_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAbilityAnimationToolsMenu",
    "WorkbenchExtensionControlRigPreviewButton",
    "WorkbenchExtensionControlRigValidateButton",
];
const CONTROL_RIG_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.control_rig.open",
        "WorkbenchAbilityAnimationToolsMenu",
    ),
    action(
        "workbench.extension.control_rig.preview.invoke",
        "WorkbenchExtensionControlRigPreviewButton",
    ),
    action(
        "workbench.extension.control_rig.validate.invoke",
        "WorkbenchExtensionControlRigValidateButton",
    ),
];
const CONTROL_RIG_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.control_rig.control.edit",
    "workbench.extension.control_rig.control.commit",
    "workbench.extension.control_rig.space.edit",
    "workbench.extension.control_rig.space.commit",
    "workbench.extension.control_rig.weight.edit",
    "workbench.extension.control_rig.weight.commit",
];

pub(super) const CONTROL_RIG_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.control_rig.open",
    "WorkbenchExtensionControlRigWorkspace",
    CONTROL_RIG_ROW_CONTROLS,
    CONTROL_RIG_ROW_ACTIONS,
    CONTROL_RIG_COMMAND_CONTROLS,
    CONTROL_RIG_COMMAND_ACTIONS,
    CONTROL_RIG_FIELD_ACTIONS,
);

const MOTION_MATCHING_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionMotionMatchingLocomotionRow",
    "WorkbenchExtensionMotionMatchingCombatRow",
    "WorkbenchExtensionMotionMatchingTraversalRow",
    "WorkbenchExtensionMotionMatchingIdleClipTableRow",
    "WorkbenchExtensionMotionMatchingStartClipTableRow",
    "WorkbenchExtensionMotionMatchingPivotClipTableRow",
    "WorkbenchExtensionMotionMatchingStopClipTableRow",
];
const MOTION_MATCHING_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.motion_matching.locomotion_row.select",
        "WorkbenchExtensionMotionMatchingLocomotionRow",
    ),
    action(
        "workbench.extension.motion_matching.combat_row.select",
        "WorkbenchExtensionMotionMatchingCombatRow",
    ),
    action(
        "workbench.extension.motion_matching.traversal_row.select",
        "WorkbenchExtensionMotionMatchingTraversalRow",
    ),
    action(
        "workbench.extension.motion_matching.idle_clip_table_row.select",
        "WorkbenchExtensionMotionMatchingIdleClipTableRow",
    ),
    action(
        "workbench.extension.motion_matching.start_clip_table_row.select",
        "WorkbenchExtensionMotionMatchingStartClipTableRow",
    ),
    action(
        "workbench.extension.motion_matching.pivot_clip_table_row.select",
        "WorkbenchExtensionMotionMatchingPivotClipTableRow",
    ),
    action(
        "workbench.extension.motion_matching.stop_clip_table_row.select",
        "WorkbenchExtensionMotionMatchingStopClipTableRow",
    ),
];
const MOTION_MATCHING_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAbilityAnimationToolsMenu",
    "WorkbenchExtensionMotionMatchingPreviewButton",
    "WorkbenchExtensionMotionMatchingRebuildButton",
];
const MOTION_MATCHING_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.motion_matching.open",
        "WorkbenchAbilityAnimationToolsMenu",
    ),
    action(
        "workbench.extension.motion_matching.preview.invoke",
        "WorkbenchExtensionMotionMatchingPreviewButton",
    ),
    action(
        "workbench.extension.motion_matching.rebuild.invoke",
        "WorkbenchExtensionMotionMatchingRebuildButton",
    ),
];
const MOTION_MATCHING_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.motion_matching.database.edit",
    "workbench.extension.motion_matching.database.commit",
    "workbench.extension.motion_matching.trajectory.edit",
    "workbench.extension.motion_matching.trajectory.commit",
    "workbench.extension.motion_matching.cost.edit",
    "workbench.extension.motion_matching.cost.commit",
];

pub(super) const MOTION_MATCHING_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.motion_matching.open",
    "WorkbenchExtensionMotionMatchingWorkspace",
    MOTION_MATCHING_ROW_CONTROLS,
    MOTION_MATCHING_ROW_ACTIONS,
    MOTION_MATCHING_COMMAND_CONTROLS,
    MOTION_MATCHING_COMMAND_ACTIONS,
    MOTION_MATCHING_FIELD_ACTIONS,
);

const ANIMATION_COMPRESSION_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionAnimationCompressionLocomotionRow",
    "WorkbenchExtensionAnimationCompressionCombatRow",
    "WorkbenchExtensionAnimationCompressionCinematicRow",
    "WorkbenchExtensionAnimationCompressionRunClipTableRow",
    "WorkbenchExtensionAnimationCompressionAttackClipTableRow",
    "WorkbenchExtensionAnimationCompressionFacialClipTableRow",
    "WorkbenchExtensionAnimationCompressionErrorClipTableRow",
];
const ANIMATION_COMPRESSION_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.animation_compression.locomotion_row.select",
        "WorkbenchExtensionAnimationCompressionLocomotionRow",
    ),
    action(
        "workbench.extension.animation_compression.combat_row.select",
        "WorkbenchExtensionAnimationCompressionCombatRow",
    ),
    action(
        "workbench.extension.animation_compression.cinematic_row.select",
        "WorkbenchExtensionAnimationCompressionCinematicRow",
    ),
    action(
        "workbench.extension.animation_compression.run_clip_table_row.select",
        "WorkbenchExtensionAnimationCompressionRunClipTableRow",
    ),
    action(
        "workbench.extension.animation_compression.attack_clip_table_row.select",
        "WorkbenchExtensionAnimationCompressionAttackClipTableRow",
    ),
    action(
        "workbench.extension.animation_compression.facial_clip_table_row.select",
        "WorkbenchExtensionAnimationCompressionFacialClipTableRow",
    ),
    action(
        "workbench.extension.animation_compression.error_clip_table_row.select",
        "WorkbenchExtensionAnimationCompressionErrorClipTableRow",
    ),
];
const ANIMATION_COMPRESSION_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAbilityAnimationToolsMenu",
    "WorkbenchExtensionAnimationCompressionPreviewButton",
    "WorkbenchExtensionAnimationCompressionCompressButton",
];
const ANIMATION_COMPRESSION_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.animation_compression.open",
        "WorkbenchAbilityAnimationToolsMenu",
    ),
    action(
        "workbench.extension.animation_compression.preview.invoke",
        "WorkbenchExtensionAnimationCompressionPreviewButton",
    ),
    action(
        "workbench.extension.animation_compression.compress.invoke",
        "WorkbenchExtensionAnimationCompressionCompressButton",
    ),
];
const ANIMATION_COMPRESSION_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.animation_compression.codec.edit",
    "workbench.extension.animation_compression.codec.commit",
    "workbench.extension.animation_compression.tolerance.edit",
    "workbench.extension.animation_compression.tolerance.commit",
    "workbench.extension.animation_compression.rate.edit",
    "workbench.extension.animation_compression.rate.commit",
];

pub(super) const ANIMATION_COMPRESSION_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.animation_compression.open",
    "WorkbenchExtensionAnimationCompressionWorkspace",
    ANIMATION_COMPRESSION_ROW_CONTROLS,
    ANIMATION_COMPRESSION_ROW_ACTIONS,
    ANIMATION_COMPRESSION_COMMAND_CONTROLS,
    ANIMATION_COMPRESSION_COMMAND_ACTIONS,
    ANIMATION_COMPRESSION_FIELD_ACTIONS,
);
