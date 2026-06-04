import {
  abilityBottom,
  assetBottom,
  behaviorBottom,
  gameplayBottom,
  hudBottom,
  materialBottom,
  perceptionBottom,
  renderPipelineBottom,
  sceneBottom,
  tagsBottom,
  vfxBottom
} from "./core-module-bottoms.js";
import {
  abilityCenter,
  assetCenter,
  behaviorCenter,
  gameplayCenter,
  hudCenter,
  materialCenter,
  perceptionCenter,
  renderPipelineCenter,
  sceneCenter,
  tagsCenter,
  vfxCenter
} from "./core-module-centers.js";
import {
  abilityDetails,
  assetDetails,
  behaviorDetails,
  gameplayDetails,
  hudDetails,
  materialDetails,
  perceptionDetails,
  renderPipelineDetails,
  sceneDetails,
  tagsDetails,
  vfxDetails
} from "./core-module-details.js";
import {
  abilityLeft,
  assetLeft,
  behaviorLeft,
  gameplayLeft,
  hudLeft,
  materialLeft,
  perceptionLeft,
  renderPipelineLeft,
  sceneLeft,
  tagsLeft,
  vfxLeft
} from "./core-module-lefts.js";
import { bottomOutput } from "./module-components.js";

export const coreModules = [
  {
    id: "scene",
    label: "Scene",
    icon: "cube",
    status: "Scene workbench ready",
    actions: [
      ["save", "Save"],
      ["folder", "Browse"],
      ["grid", "Snap"],
      ["play", "Preview"]
    ],
    left: () => sceneLeft(),
    center: () => sceneCenter(),
    right: () => sceneDetails(),
    bottom: () => bottomOutput("scene", ["Selection", "Console", "Validation"], sceneBottom())
  },
  {
    id: "gameplay-effect",
    label: "Gameplay Effect",
    shortLabel: "Effect",
    icon: "component",
    status: "GE_HealthRegen selected",
    actions: [
      ["save", "Save"],
      ["folder", "Browse"],
      ["check", "Compile"],
      ["history", "Diff"],
      ["play", "Simulation"]
    ],
    left: () => gameplayLeft(),
    center: () => gameplayCenter(),
    right: () => gameplayDetails(),
    bottom: () => bottomOutput("gameplay-effect", ["Simulation Output", "Attribute Delta", "Validation", "Compile Log"], gameplayBottom())
  },
  {
    id: "gameplay-ability",
    label: "Gameplay Ability",
    shortLabel: "Ability",
    icon: "play",
    status: "GA_DashAttack ability graph open",
    actions: [
      ["save", "Save"],
      ["check", "Compile Ability"],
      ["history", "Diff"],
      ["search", "Find"],
      ["play", "Playtest"]
    ],
    left: () => abilityLeft(),
    center: () => abilityCenter(),
    right: () => abilityDetails(),
    bottom: () => bottomOutput("gameplay-ability", ["Timeline", "Compile Log", "Gameplay Event Log", "Simulation Console"], abilityBottom())
  },
  {
    id: "gameplay-tags",
    label: "Gameplay Tags",
    shortLabel: "Tags",
    icon: "target",
    status: "Character.State.Stunned selected",
    actions: [
      ["plus", "Add Tag"],
      ["file", "Rename"],
      ["move", "Move"],
      ["trash", "Delete"],
      ["check", "Validate Tags"]
    ],
    left: () => tagsLeft(),
    center: () => tagsCenter(),
    right: () => tagsDetails(),
    bottom: () => bottomOutput("gameplay-tags", ["Validation Log", "Reference Scan", "Migration Preview", "Compile Log"], tagsBottom())
  },
  {
    id: "ai-perception",
    label: "AI Perception",
    shortLabel: "Perception",
    icon: "eye",
    status: "Guard_Perception drawing sight and hearing stimuli",
    actions: [
      ["play", "Simulate Perception"],
      ["target", "Focus"],
      ["grid", "2D View"],
      ["cube", "3D View"],
      ["check", "Validate Query"]
    ],
    left: () => perceptionLeft(),
    center: () => perceptionCenter(),
    right: () => perceptionDetails(),
    bottom: () => bottomOutput("ai-perception", ["Perception Timeline", "Debug Log", "Query Output", "Validation", "Compile Log"], perceptionBottom())
  },
  {
    id: "material",
    label: "Material",
    icon: "material",
    status: "M_Rock_Cliff graph open",
    actions: [
      ["save", "Save"],
      ["undo", "Undo"],
      ["check", "Compile"],
      ["play", "Preview"],
      ["cube", "Build"]
    ],
    left: () => materialLeft(),
    center: () => materialCenter(),
    right: () => materialDetails(),
    bottom: () => bottomOutput("material", ["Shader Output", "Preview Variants", "Warnings"], materialBottom())
  },
  {
    id: "behavior-tree",
    label: "Behavior Tree",
    shortLabel: "Behavior",
    icon: "component",
    status: "BT_Enemy running in preview",
    actions: [
      ["save", "Save"],
      ["undo", "Undo"],
      ["play", "Play"],
      ["target", "Debug"],
      ["check", "Validate"]
    ],
    left: () => behaviorLeft(),
    center: () => behaviorCenter(),
    right: () => behaviorDetails(),
    bottom: () => bottomOutput("behavior-tree", ["AI Debug Log", "Runtime Trace", "Breakpoint Output", "Validation Issues"], behaviorBottom())
  },
  {
    id: "render-pipeline",
    label: "Render Pipeline",
    shortLabel: "Render",
    icon: "renderer",
    status: "Frame 1234 render graph captured",
    actions: [
      ["save", "Save"],
      ["undo", "Undo"],
      ["check", "Compile Pipeline"],
      ["play", "Preview Frame"],
      ["cube", "Build Frame"]
    ],
    left: () => renderPipelineLeft(),
    center: () => renderPipelineCenter(),
    right: () => renderPipelineDetails(),
    bottom: () => bottomOutput("render-pipeline", ["Frame Capture Log", "Compile Output", "Resource Transitions", "Warnings", "Errors", "Compile Log"], renderPipelineBottom())
  },
  {
    id: "asset-browser",
    label: "Asset Browser",
    shortLabel: "Assets",
    icon: "image",
    status: "SM_Tree_Oak_01 selected",
    actions: [
      ["save", "Save All"],
      ["folder", "Import"],
      ["history", "Reimport"],
      ["check", "Validate"],
      ["cube", "Build"]
    ],
    left: () => assetLeft(),
    center: () => assetCenter(),
    right: () => assetDetails(),
    bottom: () => bottomOutput("asset-browser", ["Queue", "Output", "Validation", "Cook", "Package"], assetBottom())
  },
  {
    id: "vfx",
    label: "VFX",
    icon: "sun",
    status: "P_Bolt_01 previewing at 60 fps",
    actions: [
      ["save", "Save"],
      ["save", "Save All"],
      ["undo", "Undo"],
      ["play", "Simulate"],
      ["check", "Compile"]
    ],
    left: () => vfxLeft(),
    center: () => vfxCenter(),
    right: () => vfxDetails(),
    bottom: () => bottomOutput("vfx", ["Timeline", "Curves", "Niagara Log", "Compile Output", "Event Log"], vfxBottom())
  },
  {
    id: "hud-editor",
    label: "HUD Editor",
    shortLabel: "HUD",
    icon: "image",
    status: "WeaponPanel selected in Gameplay_HUD",
    actions: [
      ["save", "Save All"],
      ["undo", "Undo"],
      ["play", "Preview HUD"],
      ["check", "Validate UI"],
      ["cube", "Build UI"]
    ],
    left: () => hudLeft(),
    center: () => hudCenter(),
    right: () => hudDetails(),
    bottom: () => bottomOutput("hud-editor", ["Validation", "Binding Errors", "Preview Log", "Performance", "Compile Log"], hudBottom())
  }
];
