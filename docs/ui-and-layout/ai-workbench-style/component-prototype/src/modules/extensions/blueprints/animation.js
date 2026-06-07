import { animationCompressionBlueprint } from "./animation/animation-compression.js";
import { blendSpaceBlueprint } from "./animation/blend-space.js";
import { controlRigBlueprint } from "./animation/control-rig.js";
import { montageEditorBlueprint } from "./animation/montage-editor.js";
import { motionMatchingBlueprint } from "./animation/motion-matching.js";
import { poseLibraryBlueprint } from "./animation/pose-library.js";
import { retargetBlueprint } from "./animation/retarget.js";
import { sequencerBlueprint } from "./animation/sequencer.js";

export const animationBlueprints = {
  sequencer: sequencerBlueprint,
  "montage-editor": montageEditorBlueprint,
  "animation-compression": animationCompressionBlueprint,
  "blend-space": blendSpaceBlueprint,
  "control-rig": controlRigBlueprint,
  "motion-matching": motionMatchingBlueprint,
  "pose-library": poseLibraryBlueprint,
  retarget: retargetBlueprint
};
