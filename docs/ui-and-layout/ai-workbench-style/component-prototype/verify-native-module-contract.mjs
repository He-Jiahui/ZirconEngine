import { readFileSync } from "node:fs";
import { nativeModules } from "./modules.js";

const nativeEventSources = [
  [
    "workbench_top_toolbar.zui",
    "../../../../zircon_editor/assets/ui/editor/components/workbench_top_toolbar.zui",
  ],
  [
    "workbench_module_workspace.zui",
    "../../../../zircon_editor/assets/ui/editor/components/workbench_module_workspace.zui",
  ],
  [
    "workbench_additional_module_workspaces.zui",
    "../../../../zircon_editor/assets/ui/editor/components/workbench_additional_module_workspaces.zui",
  ],
];

const expectedModuleEventCount = 188;
const allowedEventKinds = new Set(["Click", "Change", "Submit"]);
const nativeModuleToWebModule = new Map([
  ["Scene", "scene"],
  ["Effect", "gameplay-effect"],
  ["Ability", "gameplay-ability"],
  ["Tags", "gameplay-tags"],
  ["Perception", "ai-perception"],
  ["Material", "material"],
  ["Behavior", "behavior-tree"],
  ["Render", "render-pipeline"],
  ["Assets", "asset-browser"],
  ["Vfx", "vfx"],
  ["Hud", "hud-editor"],
]);

const webCommandNativeActions = new Map([
  ["browse", "InvokeWorkbenchModuleBrowse"],
  ["import", "InvokeWorkbenchAssetsImport"],
  ["import-assets", "InvokeWorkbenchAssetsImport"],
  ["import-from-path", "InvokeWorkbenchAssetsImport"],
  ["simulation", "InvokeWorkbenchModuleSimulate"],
  ["simulate", "InvokeWorkbenchVfxSimulate"],
  ["add-tag", "InvokeWorkbenchTagsAdd"],
  ["rename", "InvokeWorkbenchTagsRename"],
  ["compile", "InvokeWorkbenchModuleCompile"],
  ["diff", "InvokeWorkbenchModuleDiff"],
  ["playtest", "InvokeWorkbenchAbilityPlaytest"],
  ["activate-ability", "SelectWorkbenchAbilityTaskActivate"],
  ["roughness", "SelectWorkbenchMaterialNodeRoughness"],
  ["selector", "SelectWorkbenchBehaviorNodeSelector"],
  ["attack", "SelectWorkbenchBehaviorNodeAttack"],
  ["sight", "SelectWorkbenchPerceptionSightTab"],
  ["hearing", "SelectWorkbenchPerceptionHearingTab"],
  ["simulate-perception", "InvokeWorkbenchPerceptionSimulate"],
  ["compile-pipeline", "InvokeWorkbenchRenderCompile"],
  ["preview-hud", "InvokeWorkbenchHudPreview"],
  ["weapon-panel", "SelectWorkbenchHudWidgetButton"],
  ["blackboard", "SelectWorkbenchBehaviorBlackboardTab"],
  ["timeline", "SelectWorkbenchVfxTimelineTab"],
]);

const webPrototypeOnlyCommands = new Set([
  "reimport",
  "reimport-assets",
  "build",
  "cook",
  "package",
  "preview",
  "preview-level",
  "play",
  "debug",
  "validate",
  "validate-tags",
  "compile-ability",
  "add-modifier",
  "play-montage",
  "texture-sample",
  "multiply",
  "lerp",
  "sequence",
  "validate-query",
  "preview-frame",
  "build-frame",
  "post-process-pass",
  "validate-ui",
  "build-ui",
  "more-editors",
  "find-editor",
  "browse-references",
  "validate-coverage",
  "core-modules",
  "validation",
  "warnings",
  "details",
  "metadata",
  "issues",
  "parameters",
  "node-details",
  "execution",
  "stages",
  "curves",
  "shader-output",
  "queue",
  "output",
  "p-bolt-01",
]);

const nativeModuleFeedbackRows = [
  ["Effect", "WorkbenchEffectOutputRow", "text"],
  ["Ability", "WorkbenchAbilityOutputRow", "value_text"],
  ["Tags", "WorkbenchTagsValidationRow", "value_text"],
  ["Perception", "WorkbenchPerceptionEventRow", "value_text"],
  ["Material", "WorkbenchMaterialOutputRow", "text"],
  ["Behavior", "WorkbenchBehaviorOutputRow", "text"],
  ["Render", "WorkbenchRenderCaptureRow", "value_text"],
  ["Assets", "WorkbenchAssetsOutputRow", "text"],
  ["Vfx", "WorkbenchVfxOutputRow", "text"],
  ["Hud", "WorkbenchHudValidationRow", "value_text"],
];

const nativeSharedModuleCommands = [
  ["InvokeWorkbenchModuleSave", "save_status", "save_output"],
  ["InvokeWorkbenchModuleCompile", "compile_status", "compile_output"],
  ["InvokeWorkbenchModuleDiff", "diff_status", "diff_output"],
  ["InvokeWorkbenchModuleSimulate", "simulate_status", "simulate_output"],
];

const nativeScopedCommandSamples = [
  ["material:compile", "compile", "InvokeWorkbenchModuleCompile"],
  ["vfx:compile", "compile", "InvokeWorkbenchModuleCompile"],
];

const nativeCompileFeedbackSamples = [
  ["Material", "Shader Output: material compile queued"],
  ["Behavior", "Runtime Trace: behavior tree compile queued"],
  ["Assets", "Cook: asset build graph queued"],
  ["Vfx", "Compile Output: E_Bolt compile queued"],
];

const nativeModuleWorkspaceSources = [
  [
    "workbench_main_band.zui",
    "../../../../zircon_editor/assets/ui/editor/components/workbench_main_band.zui",
  ],
  [
    "workbench_scene_tree_panel.zui",
    "../../../../zircon_editor/assets/ui/editor/components/workbench_scene_tree_panel.zui",
  ],
  [
    "workbench_viewport_panel.zui",
    "../../../../zircon_editor/assets/ui/editor/components/workbench_viewport_panel.zui",
  ],
  [
    "workbench_module_workspace.zui",
    "../../../../zircon_editor/assets/ui/editor/components/workbench_module_workspace.zui",
  ],
  [
    "workbench_additional_module_workspaces.zui",
    "../../../../zircon_editor/assets/ui/editor/components/workbench_additional_module_workspaces.zui",
  ],
];

const nativeModuleWorkspaceContracts = [
  ["Effect", "effect", "workbench_module_workspace.zui"],
  ["Ability", "ability", "workbench_additional_module_workspaces.zui"],
  ["Tags", "tags", "workbench_additional_module_workspaces.zui"],
  ["Perception", "perception", "workbench_additional_module_workspaces.zui"],
  ["Material", "material", "workbench_module_workspace.zui"],
  ["Behavior", "behavior", "workbench_module_workspace.zui"],
  ["Render", "render", "workbench_additional_module_workspaces.zui"],
  ["Assets", "assets", "workbench_module_workspace.zui"],
  ["Vfx", "vfx", "workbench_module_workspace.zui"],
  ["Hud", "hud", "workbench_additional_module_workspaces.zui"],
];

const nativeSceneShellWorkspace = {
  moduleId: "Scene",
  mainBandSource: "workbench_main_band.zui",
  sceneTreeSource: "workbench_scene_tree_panel.zui",
  viewportSource: "workbench_viewport_panel.zui",
};
const nativePendingModuleWorkspaceIds = new Set([]);
const requiredWorkspaceComponents = [
  "WorkbenchTab",
  "WorkbenchButton",
  "WorkbenchListRow",
];
const requiredWorkspaceStructuredComponents = [
  "WorkbenchTableRow",
  "WorkbenchPropertyRow",
];
const requiredWorkspaceEditableComponents = [
  "WorkbenchField",
  "WorkbenchDropdown",
];
const moduleEventInteractiveComponents = new Set([
  "WorkbenchButton",
  "WorkbenchDropdown",
  "WorkbenchField",
  "WorkbenchListRow",
  "WorkbenchPropertyRow",
  "WorkbenchTab",
  "WorkbenchTableRow",
]);
const moduleEventKindComponents = new Map([
  ["Click", new Set(["WorkbenchButton", "WorkbenchListRow", "WorkbenchPropertyRow", "WorkbenchTab", "WorkbenchTableRow"])],
  ["Change", new Set(["WorkbenchDropdown", "WorkbenchField"])],
  ["Submit", new Set(["WorkbenchDropdown", "WorkbenchField"])],
]);

function orderedNativeModuleIdsWithoutScene(nativeModuleTabs) {
  return nativeModuleTabs
    .map((tab) => tab.nativeId)
    .filter((nativeId) => nativeId && nativeId !== nativeSceneShellWorkspace.moduleId);
}

function readRelative(path) {
  return readFileSync(new URL(path, import.meta.url), "utf8");
}

function zuiModuleEventsFromBlock(block) {
  const events = [];
  for (const eventMatch of block.matchAll(
    /\{\s*id\s*=\s*"(WorkbenchModule\/[^"]+)"\s*,\s*event\s*=\s*"([^"]+)"([^}]*)\}/g,
  )) {
    const [, bindingId, eventKind, tail] = eventMatch;
    const route = tail.match(/route\s*=\s*"([^"]+)"/)?.[1] ?? "";
    events.push({ bindingId, eventKind, route });
  }
  return events;
}

function nodeBlocksFromZui(source) {
  return source.split(/(?=^\[nodes\.[^\]]+\]$)/gm);
}

function parseZuiNodes(source) {
  const nodes = new Map();
  for (const block of nodeBlocksFromZui(source)) {
    const name = block.match(/^\[nodes\.([^\]]+)\]$/m)?.[1];
    if (!name) {
      continue;
    }
    const component = block.match(/^\s*component\s*=\s*"([^"]+)"/m)?.[1] ?? "";
    const controlId = block.match(/^\s*control_id\s*=\s*"([^"]+)"/m)?.[1] ?? "";
    const children = [...block.matchAll(/\{\s*node\s*=\s*"([^"]+)"\s*\}/g)].map((match) => match[1]);
    const events = zuiModuleEventsFromBlock(block);
    nodes.set(name, { name, component, controlId, children, events, block });
  }
  return nodes;
}

function duplicateValues(values) {
  const seen = new Set();
  const duplicates = new Set();
  for (const value of values) {
    if (seen.has(value)) {
      duplicates.add(value);
    }
    seen.add(value);
  }
  return [...duplicates];
}

function moduleEventsFromZui(sourceName, source) {
  const events = [];
  for (const block of nodeBlocksFromZui(source)) {
    const nodeName = block.match(/^\[nodes\.([^\]]+)\]$/m)?.[1];
    if (!nodeName) {
      continue;
    }
    const controlId = block.match(/^\s*control_id\s*=\s*"([^"]+)"/m)?.[1];
    for (const event of zuiModuleEventsFromBlock(block)) {
      const component = block.match(/^\s*component\s*=\s*"([^"]+)"/m)?.[1] ?? "";
      events.push({ sourceName, nodeName, component, controlId, ...event });
    }
  }
  return events;
}

function nativeModuleTabsFromTopToolbar(source) {
  const tabs = [];
  for (const block of nodeBlocksFromZui(source)) {
    const nodeName = block.match(/^\[nodes\.([^\]]+)\]$/m)?.[1];
    if (!nodeName) {
      continue;
    }
    const isModuleTab =
      /^\s*component\s*=\s*"WorkbenchTab"/m.test(block) &&
      /classes\s*=\s*\[[^\]]*"workbench-module-tab"/m.test(block);
    if (!isModuleTab) {
      continue;
    }
    const nativeId = block.match(/id\s*=\s*"WorkbenchModule\/([^"]+)"/)?.[1] ?? null;
    tabs.push({
      nodeName,
      nativeId,
      webId: nativeId ? nativeModuleToWebModule.get(nativeId) : null,
    });
  }
  return tabs;
}

function moduleBindingsFromRust(source) {
  const bindings = new Map();
  for (const match of source.matchAll(/\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,?\s*\)/g)) {
    const [, controlKey, actionId] = match;
    if (actionId.startsWith("SelectWorkbench") || actionId.startsWith("InvokeWorkbench")) {
      bindings.set(`WorkbenchModule/${controlKey}`, { eventKind: "Click", actionId });
    }
  }
  for (const match of source.matchAll(
    /insert_(change|submit)\s*\(\s*bindings\s*,\s*"WorkbenchModule"\s*,\s*"([^"]+)"\s*,\s*EditorUiBindingPayload::menu_action\("([^"]+)"\)/gs,
  )) {
    const [, kind, controlKey, actionId] = match;
    bindings.set(`WorkbenchModule/${controlKey}`, {
      eventKind: kind[0].toUpperCase() + kind.slice(1),
      actionId,
    });
  }
  for (const match of source.matchAll(
    /\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*EditorUiEventKind::(Change|Submit)\s*,?\s*\)/g,
  )) {
    const [, controlKey, actionId, eventKind] = match;
    bindings.set(`WorkbenchModule/${controlKey}`, { eventKind, actionId });
  }
  return bindings;
}

function previewActionsFromRust(source) {
  const listStart = source.indexOf("WORKBENCH_PREVIEW_ACTION_IDS");
  const bracketStart = source.indexOf("&[", listStart);
  const bracketEnd = source.indexOf("];", bracketStart);
  if (listStart < 0 || bracketStart < 0 || bracketEnd < 0) {
    throw new Error("WORKBENCH_PREVIEW_ACTION_IDS array was not found");
  }
  const listBody = source.slice(bracketStart, bracketEnd);
  return new Set([...listBody.matchAll(/"([A-Za-z0-9_]+)"/g)].map((match) => match[1]));
}

function routeCommandIdsFromSource(source) {
  const commands = new Set();
  for (const mapMatch of source.matchAll(/new Map\(\[([\s\S]*?)\]\)/g)) {
    for (const entryMatch of mapMatch[1].matchAll(/\[\s*"([^"]+)"\s*,\s*"[^"]+"\s*\]/g)) {
      commands.add(entryMatch[1]);
    }
  }
  return commands;
}

function routeScopedKeysFromSource(source) {
  const keys = new Set();
  const scopedMap = source.match(/const\s+moduleScopedRouteMap\s*=\s*new Map\(\[([\s\S]*?)\]\);/);
  if (!scopedMap) {
    return keys;
  }
  for (const entryMatch of scopedMap[1].matchAll(/\[\s*"([^"]+:[^"]+)"\s*,\s*\{/g)) {
    keys.add(entryMatch[1]);
  }
  return keys;
}

function escapeRegExp(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function collectDescendantNodes(nodes, rootName, visited = new Set()) {
  if (visited.has(rootName)) {
    return [];
  }
  visited.add(rootName);
  const node = nodes.get(rootName);
  if (!node) {
    return [];
  }
  return [
    node,
    ...node.children.flatMap((childName) => collectDescendantNodes(nodes, childName, visited)),
  ];
}

function assertIncludes(source, expected, label, failures) {
  if (!source.includes(expected)) {
    failures.push(`${label} should include ${expected}`);
  }
}

function assertSceneShellWorkspaceContract(failures) {
  const mainBandNodes = workspaceNodeMaps.get(nativeSceneShellWorkspace.mainBandSource);
  const sceneTreeNodes = workspaceNodeMaps.get(nativeSceneShellWorkspace.sceneTreeSource);
  const viewportNodes = workspaceNodeMaps.get(nativeSceneShellWorkspace.viewportSource);
  if (!mainBandNodes || !sceneTreeNodes || !viewportNodes) {
    failures.push("Scene shell workspace sources were not loaded");
    return;
  }

  const mainBand = mainBandNodes.get("main_band");
  if (!mainBand) {
    failures.push("Scene shell workspace is missing main_band");
    return;
  }
  const mainBandComponents = new Set(collectDescendantNodes(mainBandNodes, "main_band").map((node) => node.component));
  for (const required of ["WorkbenchSceneTreePanel", "WorkbenchViewportPanel", "WorkbenchModuleWorkspace"]) {
    if (!mainBandComponents.has(required)) {
      failures.push(`Scene shell workspace is missing ${required}`);
    }
  }
  const sceneTree = sceneTreeNodes.get("scene_tree_panel");
  if (!sceneTree) {
    failures.push("Scene shell workspace is missing scene_tree_panel");
  } else {
    const sceneTreeDescendants = collectDescendantNodes(sceneTreeNodes, "scene_tree_panel");
    const sceneTreeComponents = new Set(sceneTreeDescendants.map((node) => node.component));
    for (const required of ["WorkbenchTab", "WorkbenchField", "WorkbenchIconButton", "WorkbenchTreeRow"]) {
      if (!sceneTreeComponents.has(required)) {
        failures.push(`Scene tree panel is missing ${required}`);
      }
    }
    if (sceneTreeDescendants.filter((node) => node.component === "WorkbenchTreeRow").length < 6) {
      failures.push("Scene tree panel should expose at least six authored WorkbenchTreeRow nodes");
    }
  }

  const viewportPanel = viewportNodes.get("viewport_panel");
  if (!viewportPanel) {
    failures.push("Scene viewport workspace is missing viewport_panel");
  } else {
    const viewportDescendants = collectDescendantNodes(viewportNodes, "viewport_panel");
    const viewportComponents = new Set(viewportDescendants.map((node) => node.component));
    for (const required of ["WorkbenchChip", "Space", "Label"]) {
      if (!viewportComponents.has(required)) {
        failures.push(`Scene viewport panel is missing ${required}`);
      }
    }
    const viewportControlIds = new Set(viewportDescendants.map((node) => node.controlId));
    for (const required of [
      "WorkbenchViewportBackdrop",
      "WorkbenchViewportFloor",
      "WorkbenchViewportGridH0",
      "WorkbenchViewportPropBody",
      "WorkbenchViewportSelectionTop",
      "WorkbenchViewportGizmoPanel",
      "WorkbenchViewportGizmoX",
      "WorkbenchViewportGizmoY",
      "WorkbenchViewportGizmoZ",
      "WorkbenchViewportGizmoCenter",
    ]) {
      if (!viewportControlIds.has(required)) {
        failures.push(`Scene viewport panel is missing ${required}`);
      }
    }
  }
}

function isWorkbenchModulePreviewAction(actionId) {
  return /^(Select|Invoke|Edit|Commit)Workbench(Module|Effect|Material|Behavior|Assets|Vfx|Ability|Tags|Perception|Render|Hud)/.test(
    actionId,
  );
}

const moduleEvents = nativeEventSources.flatMap(([sourceName, path]) =>
  moduleEventsFromZui(sourceName, readRelative(path)),
);
const webModuleIds = nativeModules.map((module) => module.id);
const nativeModuleTabs = nativeModuleTabsFromTopToolbar(readRelative(nativeEventSources[0][1]));
const nativeModuleIds = nativeModuleTabs.map((tab) => tab.webId).filter(Boolean);
const moduleBindings = moduleBindingsFromRust(
  readRelative("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs"),
);
const previewActions = previewActionsFromRust(
  readRelative("../../../../zircon_editor/src/ui/retained_host/workbench_preview_actions.rs"),
);
const routesSource = readRelative("./routes.js");
const webRouteCommands = routeCommandIdsFromSource(routesSource);
const webScopedRouteKeys = routeScopedKeysFromSource(routesSource);
const moduleFeedbackSource = readRelative(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs",
);
const workspaceSources = new Map(
  nativeModuleWorkspaceSources.map(([sourceName, path]) => [sourceName, readRelative(path)]),
);
const workspaceNodeMaps = new Map(
  [...workspaceSources].map(([sourceName, source]) => [sourceName, parseZuiNodes(source)]),
);

const failures = [];
const seenBindingIds = new Set();
const declaredBindingIds = new Set(moduleEvents.map((event) => event.bindingId));
const bindingActionIds = new Set([...moduleBindings.values()].map((binding) => binding.actionId));
const modulePreviewActions = new Set([...previewActions].filter(isWorkbenchModulePreviewAction));

for (const tab of nativeModuleTabs) {
  if (!tab.nativeId) {
    failures.push(`native module tab ${tab.nodeName} has no WorkbenchModule/* event`);
  } else if (!tab.webId) {
    failures.push(`native module tab ${tab.nodeName} uses unmapped native module id ${tab.nativeId}`);
  }
}

for (const duplicate of duplicateValues(nativeModuleIds)) {
  failures.push(`native module tab ${duplicate} is declared more than once`);
}

for (const duplicate of duplicateValues(webModuleIds)) {
  failures.push(`web module registry id ${duplicate} is declared more than once`);
}

if (JSON.stringify(nativeModuleIds) !== JSON.stringify(webModuleIds)) {
  failures.push(
    `native module tabs [${nativeModuleIds.join(", ")}] do not match web module registry [${webModuleIds.join(", ")}]`,
  );
}

if (moduleEvents.length !== expectedModuleEventCount) {
  failures.push(
    `expected ${expectedModuleEventCount} WorkbenchModule/* events, found ${moduleEvents.length}`,
  );
}

for (const nativeId of nativeModuleToWebModule.keys()) {
  const hasWorkspaceContract =
    nativeId === nativeSceneShellWorkspace.moduleId
    || nativeModuleWorkspaceContracts.some(([moduleId]) => moduleId === nativeId);
  if (!hasWorkspaceContract && !nativePendingModuleWorkspaceIds.has(nativeId)) {
    failures.push(`${nativeId} has no native workspace contract and is not marked pending`);
  }
}

const expectedWorkspaceModuleOrder = orderedNativeModuleIdsWithoutScene(nativeModuleTabs);
const actualWorkspaceModuleOrder = nativeModuleWorkspaceContracts.map(([moduleId]) => moduleId);
if (JSON.stringify(actualWorkspaceModuleOrder) !== JSON.stringify(expectedWorkspaceModuleOrder)) {
  failures.push(
    `native workspace contracts [${actualWorkspaceModuleOrder.join(", ")}] do not match top toolbar module order without Scene [${expectedWorkspaceModuleOrder.join(", ")}]`,
  );
}

assertSceneShellWorkspaceContract(failures);

for (const [moduleId, prefix, sourceName] of nativeModuleWorkspaceContracts) {
  const nodes = workspaceNodeMaps.get(sourceName);
  if (!nodes) {
    failures.push(`${moduleId} workspace source ${sourceName} was not loaded`);
    continue;
  }

  const workspaceName = `${prefix}_workspace`;
  const workspace = nodes.get(workspaceName);
  const expectedWorkspaceChildren = [`${prefix}_rail_gap`, `${prefix}_left`, `${prefix}_center`, `${prefix}_right`];
  if (!workspace) {
    failures.push(`${moduleId} workspace is missing ${workspaceName}`);
  } else {
    if (workspace.component !== "HorizontalGroup") {
      failures.push(`${moduleId} ${workspaceName} should be a HorizontalGroup, found ${workspace.component}`);
    }
    if (!workspace.block.includes('classes = ["workbench-module-body"]')) {
      failures.push(`${moduleId} ${workspaceName} should use workbench-module-body classes`);
    }
    assertIncludes(workspace.block, 'kind = "HorizontalBox"', `${moduleId} ${workspaceName}`, failures);
    assertIncludes(workspace.block, 'gap = 10.0', `${moduleId} ${workspaceName}`, failures);
    assertIncludes(workspace.block, 'width = { stretch = "Stretch" }', `${moduleId} ${workspaceName}`, failures);
    assertIncludes(workspace.block, 'height = { stretch = "Stretch" }', `${moduleId} ${workspaceName}`, failures);
    if (JSON.stringify(workspace.children) !== JSON.stringify(expectedWorkspaceChildren)) {
      failures.push(
        `${moduleId} ${workspaceName} children [${workspace.children.join(", ")}] should be [${expectedWorkspaceChildren.join(", ")}]`,
      );
    }
  }

  const railGap = nodes.get(`${prefix}_rail_gap`);
  if (!railGap) {
    failures.push(`${moduleId} workspace is missing ${prefix}_rail_gap`);
  } else {
    if (railGap.component !== "Container") {
      failures.push(`${moduleId} ${prefix}_rail_gap should be a Container, found ${railGap.component}`);
    }
    for (const expected of [
      "min = 72.0",
      "preferred = 72.0",
      "max = 72.0",
      'stretch = "Fixed"',
      'height = { stretch = "Stretch" }',
    ]) {
      assertIncludes(railGap.block, expected, `${moduleId} ${prefix}_rail_gap`, failures);
    }
  }

  const regionNames = ["left", "center", "right"].map((region) => `${prefix}_${region}`);
  for (const regionName of regionNames) {
    const region = nodes.get(regionName);
    if (!region) {
      failures.push(`${moduleId} workspace is missing ${regionName}`);
      continue;
    }
    if (region.component !== "VerticalGroup") {
      failures.push(`${moduleId} ${regionName} should be a VerticalGroup region, found ${region.component}`);
    }
    if (!region.block.includes('classes = ["workbench-panel"')) {
      failures.push(`${moduleId} ${regionName} should use workbench-panel classes`);
    }
    assertIncludes(region.block, 'kind = "VerticalBox"', `${moduleId} ${regionName}`, failures);
    assertIncludes(region.block, 'height = { stretch = "Stretch" }', `${moduleId} ${regionName}`, failures);
    if (regionName.endsWith("_center")) {
      assertIncludes(region.block, 'width = { stretch = "Stretch" }', `${moduleId} ${regionName}`, failures);
      if (!region.block.includes('"workbench-module-center"')) {
        failures.push(`${moduleId} ${regionName} should use workbench-module-center classes`);
      }
    } else if (regionName.endsWith("_left")) {
      for (const expected of [
        "min = 270.0",
        "preferred = 270.0",
        "max = 290.0",
        'stretch = "Fixed"',
      ]) {
        assertIncludes(region.block, expected, `${moduleId} ${regionName}`, failures);
      }
      if (!region.block.includes('"workbench-module-side"')) {
        failures.push(`${moduleId} ${regionName} should use workbench-module-side classes`);
      }
    } else if (regionName.endsWith("_right")) {
      for (const expected of [
        "min = 320.0",
        "preferred = 320.0",
        "max = 340.0",
        'stretch = "Fixed"',
      ]) {
        assertIncludes(region.block, expected, `${moduleId} ${regionName}`, failures);
      }
      if (!region.block.includes('"workbench-module-side"')) {
        failures.push(`${moduleId} ${regionName} should use workbench-module-side classes`);
      }
    }
    if (region.children.length < 3) {
      failures.push(`${moduleId} ${regionName} should compose at least three child components`);
    }
  }

  const workspaceNodes = regionNames.flatMap((regionName) => collectDescendantNodes(nodes, regionName));
  const workspaceComponents = new Set(workspaceNodes.map((node) => node.component).filter(Boolean));
  const workspaceEvents = workspaceNodes.flatMap((node) => node.events);

  for (const required of requiredWorkspaceComponents) {
    if (!workspaceComponents.has(required)) {
      failures.push(`${moduleId} workspace is missing ${required}`);
    }
  }
  if (!requiredWorkspaceStructuredComponents.some((component) => workspaceComponents.has(component))) {
    failures.push(`${moduleId} workspace should include a table row or property row`);
  }
  if (!requiredWorkspaceEditableComponents.some((component) => workspaceComponents.has(component))) {
    failures.push(`${moduleId} workspace should include an editable field or dropdown`);
  }
  if (workspaceEvents.length < 10) {
    failures.push(`${moduleId} workspace should expose at least ten WorkbenchModule events, found ${workspaceEvents.length}`);
  }
  for (const event of workspaceEvents) {
    if (!event.bindingId.startsWith(`WorkbenchModule/${moduleId}`)) {
      failures.push(`${moduleId} workspace event ${event.bindingId} does not use the ${moduleId} namespace`);
    }
    if (!event.route.startsWith(`WorkbenchModule.${moduleId}.`)) {
      failures.push(
        `${moduleId} workspace event ${event.bindingId} route ${event.route} does not use the WorkbenchModule.${moduleId} route namespace`,
      );
    }
    if (!declaredBindingIds.has(event.bindingId)) {
      failures.push(`${moduleId} workspace event ${event.bindingId} is not part of the native event contract`);
    }
  }
}

for (const pendingModuleId of nativePendingModuleWorkspaceIds) {
  if (!nativeModuleToWebModule.has(pendingModuleId)) {
    failures.push(`${pendingModuleId} is marked as pending native workspace but is not a known native module`);
  }
  if (nativeModuleWorkspaceContracts.some(([moduleId]) => moduleId === pendingModuleId)) {
    failures.push(`${pendingModuleId} is both pending and covered by a native workspace contract`);
  }
}

for (const event of moduleEvents) {
  if (!event.controlId) {
    failures.push(`${event.sourceName}:${event.nodeName} ${event.bindingId} has no control_id`);
  }
  if (!moduleEventInteractiveComponents.has(event.component)) {
    failures.push(
      `${event.sourceName}:${event.nodeName} ${event.bindingId} is attached to non-interactive component ${event.component || "(missing)"}`,
    );
  }
  if (!allowedEventKinds.has(event.eventKind)) {
    failures.push(
      `${event.sourceName}:${event.nodeName} ${event.bindingId} uses unsupported event ${event.eventKind}`,
    );
  } else if (!moduleEventKindComponents.get(event.eventKind)?.has(event.component)) {
    failures.push(
      `${event.sourceName}:${event.nodeName} ${event.bindingId} uses ${event.eventKind} on ${event.component || "(missing)"}`,
    );
  }
  if (!event.route) {
    failures.push(`${event.sourceName}:${event.nodeName} ${event.bindingId} has no route`);
  } else if (!event.route.startsWith("WorkbenchModule.")) {
    failures.push(
      `${event.sourceName}:${event.nodeName} ${event.bindingId} route ${event.route} is outside WorkbenchModule.*`,
    );
  }
  if (seenBindingIds.has(event.bindingId)) {
    failures.push(`${event.bindingId} is declared more than once`);
  }
  seenBindingIds.add(event.bindingId);

  const binding = moduleBindings.get(event.bindingId);
  if (!binding) {
    failures.push(`${event.bindingId} is declared in ZUI but missing from native bindings`);
    continue;
  }
  if (binding.eventKind !== event.eventKind) {
    failures.push(
      `${event.bindingId} is declared as ${event.eventKind} but native binding is ${binding.eventKind}`,
    );
  }
}

for (const [bindingId, binding] of moduleBindings) {
  if (!declaredBindingIds.has(bindingId)) {
    failures.push(`${bindingId} exists in native bindings but has no ZUI declaration`);
  }
  if (!previewActions.has(binding.actionId)) {
    failures.push(`${bindingId} resolves to unregistered preview action ${binding.actionId}`);
  }
  if (!modulePreviewActions.has(binding.actionId)) {
    failures.push(`${bindingId} resolves outside the Workbench module preview namespace: ${binding.actionId}`);
  }
}

for (const actionId of modulePreviewActions) {
  if (!bindingActionIds.has(actionId)) {
    failures.push(`${actionId} is registered as a Workbench module preview action but has no native binding`);
  }
}

for (const command of webRouteCommands) {
  if (!webCommandNativeActions.has(command) && !webPrototypeOnlyCommands.has(command)) {
    failures.push(`${command} is routed in the web prototype but is not classified for native coverage`);
  }
}

for (const command of webCommandNativeActions.keys()) {
  if (!webRouteCommands.has(command)) {
    failures.push(`${command} is classified as native-covered but no longer exists in web routes.js`);
  }
  if (webPrototypeOnlyCommands.has(command)) {
    failures.push(`${command} is classified as both native-covered and prototype-only`);
  }
}

for (const command of webPrototypeOnlyCommands) {
  if (!webRouteCommands.has(command)) {
    failures.push(`${command} is classified as prototype-only but no longer exists in web routes.js`);
  }
}

for (const [command, actionId] of webCommandNativeActions) {
  if (!previewActions.has(actionId)) {
    failures.push(`${command} expects native preview action ${actionId}, but it is not registered`);
  }
  if (!modulePreviewActions.has(actionId)) {
    failures.push(`${command} expects ${actionId}, but it is outside the Workbench module namespace`);
  }
  if (!bindingActionIds.has(actionId)) {
    failures.push(`${command} expects ${actionId}, but no native module binding resolves to it`);
  }
}

for (const [variant, controlId, property] of nativeModuleFeedbackRows) {
  const tabPattern = new RegExp(
    `"WorkbenchModule${escapeRegExp(variant)}"\\s*,\\s*Self::${escapeRegExp(variant)}`,
  );
  if (!tabPattern.test(moduleFeedbackSource)) {
    failures.push(`module feedback does not resolve WorkbenchModule${variant} as Self::${variant}`);
  }

  const outputPattern = new RegExp(
    `Self::${escapeRegExp(variant)}\\s*=>\\s*Some\\(output\\(\\s*"${escapeRegExp(controlId)}"\\s*,\\s*"${escapeRegExp(property)}"\\s*,\\s*text\\s*\\)\\)`,
    "s",
  );
  if (!outputPattern.test(moduleFeedbackSource)) {
    failures.push(`module feedback does not route ${variant} output to ${controlId}.${property}`);
  }
}

for (const [actionId, statusMethod, outputMethod] of nativeSharedModuleCommands) {
  const feedbackPattern = new RegExp(
    `"${escapeRegExp(actionId)}"\\s*=>\\s*module_feedback\\(\\s*active_module\\.${escapeRegExp(statusMethod)}\\(\\),\\s*active_module\\.output\\(active_module\\.${escapeRegExp(outputMethod)}\\(\\)\\),\\s*\\)`,
    "s",
  );
  if (!feedbackPattern.test(moduleFeedbackSource)) {
    failures.push(`${actionId} should resolve status/output through the active Workbench module`);
  }
}

for (const [scopedKey, command, actionId] of nativeScopedCommandSamples) {
  if (!webScopedRouteKeys.has(scopedKey)) {
    failures.push(`${scopedKey} should stay in the web module-scoped route table`);
  }
  if (webCommandNativeActions.get(command) !== actionId) {
    failures.push(`${command} should remain classified as native action ${actionId}`);
  }
}

for (const [variant, outputText] of nativeCompileFeedbackSamples) {
  const compileOutputPattern = new RegExp(
    `Self::${escapeRegExp(variant)}\\s*=>\\s*"${escapeRegExp(outputText)}"`,
  );
  if (!compileOutputPattern.test(moduleFeedbackSource)) {
    failures.push(`${variant} compile feedback should preserve "${outputText}"`);
  }
}

console.log(
  `native module contract: modules=${nativeModuleIds.length} events=${moduleEvents.length} routed=${moduleEvents.filter((event) => event.route).length} unique=${seenBindingIds.size} bindings=${moduleBindings.size} modulePreviewActions=${modulePreviewActions.size} previewActions=${previewActions.size} webCommands=${webRouteCommands.size} nativeCoveredWebCommands=${webCommandNativeActions.size} scopedRoutes=${webScopedRouteKeys.size} feedbackRows=${nativeModuleFeedbackRows.length}`,
);

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(`fail ${failure}`);
  }
  process.exit(1);
}

console.log("ok native Workbench module event contract");
