import { readFileSync } from "node:fs";
import { nativeModules } from "./src/modules/modules.js";

const previewActionIdPattern = /^[a-z0-9_]+(?:\.[a-z0-9_]+)+$/;
const expectedGeneratedBottomControlEvents = 46;
const generatedBottomComponentUrl = "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_panel.zui";
const generatedBottomDrawerUrl = "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_drawer.zui";
const generatedBottomBodyUrl = "../../../../zircon_editor/assets/ui/editor/host/generated_bottom_body.zui";
const moduleWorkspaceUrl = "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui";
const generatedBottomBindingsUrl =
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_generated_bottom_template_bindings.rs";
const windowTemplateBindingsUrl =
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs";
const templateDocumentsUrl = "../../../../zircon_editor/src/ui/template_runtime/builtin/template_documents.rs";
const componentDescriptorsUrl = "../../../../zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs";
const componentCatalogUrl = "../../../../zircon_editor/assets/ui/editor/components/catalog.toml";
const builtinModUrl = "../../../../zircon_editor/src/ui/template_runtime/builtin/mod.rs";
const generatedBottomViewDescriptorUrl =
  "../../../../zircon_editor/src/ui/host/builtin_views/activity_views/generated_bottom_view_descriptor.rs";
const activityViewDescriptorsUrl =
  "../../../../zircon_editor/src/ui/host/builtin_views/activity_views/activity_view_descriptors.rs";
const shellViewInstancesUrl =
  "../../../../zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs";
const viewContentKindUrl = "../../../../zircon_editor/src/ui/workbench/snapshot/workbench/view_content_kind.rs";
const descriptorContentKindUrl =
  "../../../../zircon_editor/src/ui/workbench/snapshot/workbench/descriptor_content_kind.rs";
const panePayloadKindUrl = "../../../../zircon_editor/src/ui/workbench/view/pane_payload_kind.rs";
const panePayloadUrl = "../../../../zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs";
const paneProjectionUrl = "../../../../zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs";
const applyPresentationUrl =
  "../../../../zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs";
const hostContractPanesUrl =
  "../../../../zircon_editor/src/ui/retained_host/host_contract/data/panes/pane.rs";
const paneDataConversionUrl = "../../../../zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs";
const hostContractWindowUrl =
  "../../../../zircon_editor/src/ui/retained_host/ui/shell_content_presentation.rs";
const profilingArtifactsUrl = "../../../../zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs";
const profilingPaneNodesUrl =
  "../../../../zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/template_nodes/source.rs";
const hitTestTemplateNodeUrl =
  "../../../../zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_nodes.rs";
const painterWorkbenchUrl =
  "../../../../zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/selection.rs";
const generatedBottomBodyTestUrl =
  "../../../../zircon_editor/src/tests/host/retained_generated_bottom_template_body.rs";
const generatedBottomNavigationUrl =
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_navigation.rs";
const generatedBottomFeedbackUrl =
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_feedback.rs";
const generatedBottomActionsUrl =
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_actions.rs";
const generatedBottomLifecycleUrl =
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_lifecycle.rs";
const workbenchModUrl =
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/mod.rs";
const componentizedWindowUrl =
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs";
const referenceMenuActionsUrl =
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs";
const moduleFieldEditUrl =
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs";
const previewActionsUrls = [
  "../../../../zircon_editor/src/ui/retained_host/workbench_preview_actions.rs",
  "../../../../zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs",
];

const sources = {
  matrix: readLocal("./web-native-handoff-matrix.md"),
  readme: readLocal("./README.md"),
  generatedBottomComponent: readRepo(generatedBottomComponentUrl),
  generatedBottomDrawer: readRepo(generatedBottomDrawerUrl),
  generatedBottomBody: readRepo(generatedBottomBodyUrl),
  moduleWorkspace: readRepo(moduleWorkspaceUrl),
  generatedBottomBindings: readRepo(generatedBottomBindingsUrl),
  windowTemplateBindings: readRepo(windowTemplateBindingsUrl),
  templateDocuments: readRepo(templateDocumentsUrl),
  componentDescriptors: readRepo(componentDescriptorsUrl),
  componentCatalog: readRepo(componentCatalogUrl),
  builtinMod: readRepo(builtinModUrl),
  generatedBottomViewDescriptor: readRepo(generatedBottomViewDescriptorUrl),
  activityViewDescriptors: readRepo(activityViewDescriptorsUrl),
  shellViewInstances: readRepo(shellViewInstancesUrl),
  viewContentKind: readRepo(viewContentKindUrl),
  descriptorContentKind: readRepo(descriptorContentKindUrl),
  panePayloadKind: readRepo(panePayloadKindUrl),
  panePayload: readRepo(panePayloadUrl),
  paneProjection: readRepo(paneProjectionUrl),
  applyPresentation: readRepo(applyPresentationUrl),
  hostContractPanes: readRepo(hostContractPanesUrl),
  paneDataConversion: readRepo(paneDataConversionUrl),
  hostContractWindow: readRepo(hostContractWindowUrl),
  profilingArtifacts: readRepo(profilingArtifactsUrl),
  profilingPaneNodes: readRepo(profilingPaneNodesUrl),
  hitTestTemplateNode: readRepo(hitTestTemplateNodeUrl),
  painterWorkbench: readRepo(painterWorkbenchUrl),
  generatedBottomBodyTest: readRepo(generatedBottomBodyTestUrl),
  generatedBottomNavigation: readRepo(generatedBottomNavigationUrl),
  generatedBottomFeedback: readRepo(generatedBottomFeedbackUrl),
  generatedBottomActions: readRepo(generatedBottomActionsUrl),
  generatedBottomLifecycle: readRepo(generatedBottomLifecycleUrl),
  workbenchMod: readRepo(workbenchModUrl),
  componentizedWindow: readRepo(componentizedWindowUrl),
  referenceMenuActions: readRepo(referenceMenuActionsUrl),
  moduleFieldEdit: readRepo(moduleFieldEditUrl),
  previewActions: previewActionsUrls.map(readRepo).join("\n"),
};

const failures = [];
const webGeneratedBottomRoutes = generatedBottomRoutesFromWeb();
const generatedBottomEvents = generatedBottomEventsFromZui(
  "workbench/modules/generated/workbench_generated_bottom_panel.zui",
  sources.generatedBottomComponent,
);
const generatedBottomBindings = generatedBottomBindingsFromRust(sources.generatedBottomBindings);
const previewActions = previewActionsFromRust(sources.previewActions);
const generatedBottomPreviewActions = new Set([...previewActions].filter(isGeneratedBottomPreviewAction));
const routeTargets = generatedBottomRouteTargetsFromRust(sources.generatedBottomNavigation);
const routeTargetByPanelRoute = new Map(routeTargets.map((target) => [target.panelRoute, target]));
const routeTargetByActionId = new Map(routeTargets.map((target) => [target.actionId, target]));
const eventBindingIds = new Set(generatedBottomEvents.map((event) => event.bindingId));
const bindingActionIds = new Set([...generatedBottomBindings.values()].map((binding) => binding.actionId));
const generatedBottomActionIds = new Set([
  ...bindingActionIds,
  ...generatedBottomPreviewActions,
  ...routeTargets.map((target) => target.actionId),
]);
const invalidGeneratedBottomActionIds = [...generatedBottomActionIds].filter(
  (actionId) => !previewActionIdPattern.test(actionId),
);

check(webGeneratedBottomRoutes.length === 37, `expected 37 web generated-bottom routes, found ${webGeneratedBottomRoutes.length}`);
check(routeTargets.length === webGeneratedBottomRoutes.length, `expected ${webGeneratedBottomRoutes.length} native generated-bottom route targets, found ${routeTargets.length}`);
check(generatedBottomEvents.length === expectedGeneratedBottomControlEvents, `expected ${expectedGeneratedBottomControlEvents} generated-bottom ZUI events, found ${generatedBottomEvents.length}`);
check(generatedBottomBindings.size === expectedGeneratedBottomControlEvents, `expected ${expectedGeneratedBottomControlEvents} generated-bottom bindings, found ${generatedBottomBindings.size}`);
check(generatedBottomPreviewActions.size === expectedGeneratedBottomControlEvents, `expected ${expectedGeneratedBottomControlEvents} generated-bottom preview actions, found ${generatedBottomPreviewActions.size}`);
check(
  invalidGeneratedBottomActionIds.length === 0,
  `generated-bottom action ids must use dotted lower-case functional paths: ${invalidGeneratedBottomActionIds.join(", ")}`,
);
check(
  [...generatedBottomActionIds].every((actionId) => actionId.startsWith("workbench.generated_bottom.")),
  "generated-bottom action ids must stay under workbench.generated_bottom.*",
);

for (const route of webGeneratedBottomRoutes) {
  const target = routeTargetByPanelRoute.get(route);
  check(Boolean(target), `web generated-bottom route ${route} has no native route target`);
  if (target) {
    check(sources.generatedBottomComponent.includes(`control_id = "${target.controlId}"`), `${route} row control ${target.controlId} is missing from generated-bottom component`);
    check(generatedBottomPreviewActions.has(target.actionId), `${route} action ${target.actionId} is missing from preview actions`);
    check(
      sources.generatedBottomFeedback.includes("target.panel_route"),
      `${route} feedback should consume the generated-bottom route target`,
    );
  }
}

check(sources.matrix.includes("workbench/modules/generated/workbench_generated_bottom_panel.zui"), "matrix mentions generated bottom component");
check(sources.matrix.includes("workbench/modules/generated/workbench_generated_bottom_drawer.zui"), "matrix mentions generated bottom drawer host");
check(sources.matrix.includes("generated_bottom_body.zui"), "matrix mentions generated bottom shell pane body");
check(sources.matrix.includes("generated_bottom_view_descriptor.rs"), "matrix mentions generated bottom activity view descriptor");
check(sources.matrix.includes("builtin_shell_view_instances.rs"), "matrix mentions generated bottom shell view instance");
check(sources.matrix.includes("pane_data_conversion/mod.rs"), "matrix mentions generated bottom pane conversion");
check(sources.matrix.includes("workbench_generated_bottom_template_bindings.rs"), "matrix mentions generated bottom bindings");
check(sources.matrix.includes("generated_bottom_panel_navigation.rs"), "matrix mentions generated bottom navigation");
check(sources.matrix.includes("generated_bottom_panel_feedback.rs"), "matrix mentions generated bottom feedback");
check(sources.matrix.includes("generated_bottom_panel_actions.rs"), "matrix mentions generated bottom action routing");
check(sources.matrix.includes("generated_bottom_panel_lifecycle.rs"), "matrix mentions generated bottom drawer lifecycle");
check(sources.matrix.includes("verify-native-generated-bottom-contract.mjs"), "matrix mentions generated bottom verifier");
check(sources.matrix.includes("visible shell bottom drawer pane body evidence recorded; module lifecycle remains state owner"), "matrix records generated-bottom visible shell drawer body evidence without over-promoting");
check(sources.readme.includes("verify-native-generated-bottom-contract.mjs"), "README documents generated bottom verifier");
check(sources.readme.includes("visible shell bottom drawer pane body"), "README documents generated bottom visible shell drawer body evidence");

check(
  sources.moduleWorkspace.includes("workbench/modules/generated/workbench_generated_bottom_drawer.zui#WorkbenchGeneratedBottomDrawer") &&
    sources.moduleWorkspace.includes("WorkbenchGeneratedBottomDrawerHost"),
  "native module workspace hosts generated bottom drawer component",
);
check(
  sources.generatedBottomDrawer.includes("workbench/modules/generated/workbench_generated_bottom_panel.zui#WorkbenchGeneratedBottomPanel") &&
    sources.generatedBottomDrawer.includes("[components.WorkbenchGeneratedBottomDrawer]") &&
    sources.generatedBottomDrawer.includes('control_id = "WorkbenchGeneratedBottomDrawer"') &&
    sources.generatedBottomDrawer.includes('props = { visibility = "collapsed" }') &&
    sources.generatedBottomDrawer.includes("WorkbenchGeneratedBottomPanelHost"),
  "generated bottom drawer hosts generated bottom panel content",
);
check(
  sources.generatedBottomBody.includes('id = "res://ui/editor/host/generated_bottom_body.zui"') &&
    sources.generatedBottomBody.includes("workbench/modules/generated/workbench_generated_bottom_panel.zui#WorkbenchGeneratedBottomPanel") &&
    sources.generatedBottomBody.includes('control_id = "GeneratedBottomPaneBodyRoot"') &&
    sources.generatedBottomBody.includes('control_id = "GeneratedBottomPanePanelHost"'),
  "shell generated bottom pane body mounts the shared generated bottom panel",
);
check(
  sources.templateDocuments.includes("PANE_GENERATED_BOTTOM_BODY_DOCUMENT_ID") &&
    sources.templateDocuments.includes('"res://ui/editor/host/generated_bottom_body.zui"') &&
    sources.templateDocuments.includes("generated_bottom_body.zui"),
  "builtin template documents register generated bottom shell body",
);
check(
  sources.componentDescriptors.includes("GeneratedBottomPaneBody") &&
    sources.componentDescriptors.includes("builtin_component_descriptors") &&
    sources.componentCatalog.includes('component_id = "GeneratedBottomPaneBody"') &&
    sources.componentCatalog.includes('document_id = "res://ui/editor/host/generated_bottom_body.zui"'),
  "builtin component descriptors register generated bottom shell body",
);
check(
  sources.generatedBottomViewDescriptor.includes('ViewDescriptorId::new("editor.generated_bottom")') &&
    sources.generatedBottomViewDescriptor.includes("WorkbenchSlot::BottomDrawer") &&
    sources.generatedBottomViewDescriptor.includes("ViewContentKind::GeneratedBottom") &&
    sources.generatedBottomViewDescriptor.includes('"res://ui/editor/host/generated_bottom_body.zui"') &&
    sources.generatedBottomViewDescriptor.includes("PanePayloadKind::GeneratedBottomV1") &&
    sources.generatedBottomViewDescriptor.includes("PaneRouteNamespace::Dock") &&
    sources.generatedBottomViewDescriptor.includes("PaneInteractionMode::TemplateOnly"),
  "generated bottom activity view descriptor targets the bottom drawer shell body",
);
check(
  sources.activityViewDescriptors.includes("generated_bottom_view_descriptor") &&
    sources.activityViewDescriptors.includes("generated_bottom_view_descriptor()"),
  "generated bottom activity view descriptor is registered",
);
check(
  sources.shellViewInstances.includes('ViewInstanceId::new("editor.generated_bottom#1")') &&
    sources.shellViewInstances.includes('ViewDescriptorId::new("editor.generated_bottom")') &&
    sources.shellViewInstances.includes("ViewHost::Drawer(ActivityDrawerSlot::Bottom)"),
  "generated bottom shell view instance is hosted in the bottom drawer",
);
check(
  sources.generatedBottomComponent.includes("[components.WorkbenchGeneratedBottomPanel]") &&
    sources.generatedBottomComponent.includes('control_id = "WorkbenchGeneratedBottomPanel"') &&
    sources.generatedBottomComponent.includes('props = { visibility = "visible" }') &&
    !sources.generatedBottomComponent.includes('props = { visibility = "collapsed" }') &&
    sources.generatedBottomComponent.includes("workbench-generated-bottom-route-row") &&
    !sources.generatedBottomComponent.includes("WorkbenchGeneratedBottomModeDropdown"),
  "generated bottom component exposes visible shared retained content controls",
);
for (const component of ["WorkbenchButton", "WorkbenchField", "WorkbenchSectionTitle", "WorkbenchTab", "WorkbenchTableRow"]) {
  check(sources.generatedBottomComponent.includes(component), `generated bottom component imports/uses ${component}`);
}

check(
  sources.builtinMod.includes("workbench_generated_bottom_template_bindings") &&
    sources.windowTemplateBindings.includes("insert_workbench_generated_bottom_bindings"),
  "window template installs generated bottom bindings",
);
check(
  sources.viewContentKind.includes("GeneratedBottom") &&
    sources.descriptorContentKind.includes('"editor.generated_bottom" => ViewContentKind::GeneratedBottom') &&
    sources.panePayloadKind.includes("GeneratedBottomV1") &&
    sources.panePayload.includes("GeneratedBottomV1(GeneratedBottomPanePayload)") &&
    sources.panePayload.includes("GeneratedBottomPanePayload"),
  "generated bottom view content and payload kinds are declared",
);
check(
  sources.paneProjection.includes("ViewContentKind::GeneratedBottom") &&
    sources.paneProjection.includes("generated_bottom_pane_data") &&
    sources.paneProjection.includes('"Generated Output"') &&
    sources.paneProjection.includes('"Componentized generated editor feedback panels"'),
  "generated bottom pane projection creates native body metadata",
);
check(
  sources.hostContractPanes.includes("GeneratedBottomPaneData") &&
    sources.hostContractPanes.includes("generated_bottom") &&
    sources.paneDataConversion.includes("to_host_contract_generated_bottom_pane_from_host_pane") &&
    sources.paneDataConversion.includes("project_pane_template_nodes") &&
    sources.applyPresentation.includes("to_host_contract_generated_bottom_pane") &&
    sources.applyPresentation.includes("GeneratedBottomV1"),
  "host contract projection converts generated bottom panes into retained data",
);
for (const sourceName of ["hostContractWindow", "profilingPaneNodes", "hitTestTemplateNode", "painterWorkbench"]) {
  check(
    sources[sourceName].includes('"GeneratedBottom"') &&
      sources[sourceName].includes("pane.generated_bottom.nodes"),
    `${sourceName} includes generated bottom template-node routing`,
  );
}
check(
  sources.generatedBottomBodyTest.includes("generated_bottom_template_body_projects_panel_nodes_for_retained_conversion") &&
    sources.generatedBottomBodyTest.includes("GeneratedBottomPaneViewData") &&
    sources.generatedBottomBodyTest.includes("workbench.generated_bottom.open_panel.invoke"),
  "focused retained generated bottom body test projects shell body nodes",
);
check(
  sources.workbenchMod.includes("generated_bottom_panel_navigation") &&
    sources.workbenchMod.includes("generated_bottom_panel_feedback") &&
    sources.workbenchMod.includes("generated_bottom_panel_actions") &&
    sources.workbenchMod.includes("generated_bottom_panel_lifecycle"),
  "retained workbench module declares generated bottom helpers",
);
check(
  sources.referenceMenuActions.includes("is_workbench_generated_bottom_action") &&
    sources.referenceMenuActions.includes("apply_workbench_generated_bottom_action(source_control_id, action_id)") &&
    sources.componentizedWindow.includes("close_workbench_generated_bottom_drawer()") &&
    !sources.componentizedWindow.includes("WorkbenchGeneratedBottomDrawerHost") &&
    !sources.componentizedWindow.includes("workbench_generated_bottom_route_control_id"),
  "reference dispatcher delegates generated bottom actions while componentized window owns drawer close lifecycle",
);
check(
  sources.generatedBottomActions.includes("apply_workbench_generated_bottom_action") &&
    sources.generatedBottomActions.includes("open_workbench_generated_bottom_drawer") &&
    !sources.generatedBottomActions.includes("WorkbenchGeneratedBottomDrawerHost") &&
    sources.generatedBottomActions.includes("apply_workbench_generated_bottom_feedback") &&
    sources.generatedBottomActions.includes("workbench_generated_bottom_route_control_id"),
  "generated bottom action routing opens the drawer through lifecycle and route controls",
);
check(
  sources.generatedBottomLifecycle.includes("open_workbench_generated_bottom_drawer") &&
    sources.generatedBottomLifecycle.includes("close_workbench_generated_bottom_drawer") &&
    sources.generatedBottomLifecycle.includes('GENERATED_BOTTOM_DRAWER_HOST_CONTROL_ID: &str = "WorkbenchGeneratedBottomDrawerHost"') &&
    sources.generatedBottomLifecycle.includes('GENERATED_BOTTOM_DRAWER_CONTROL_ID: &str = "WorkbenchGeneratedBottomDrawer"') &&
    sources.generatedBottomLifecycle.includes('GENERATED_BOTTOM_PANEL_CONTROL_ID: &str = "WorkbenchGeneratedBottomPanel"') &&
    sources.generatedBottomLifecycle.includes("set_visible(GENERATED_BOTTOM_DRAWER_HOST_CONTROL_ID, true)") &&
    sources.generatedBottomLifecycle.includes("set_visible(GENERATED_BOTTOM_DRAWER_CONTROL_ID, true)") &&
    sources.generatedBottomLifecycle.includes("set_visible(GENERATED_BOTTOM_PANEL_CONTROL_ID, true)") &&
    sources.generatedBottomLifecycle.includes("set_visible(GENERATED_BOTTOM_DRAWER_HOST_CONTROL_ID, false)") &&
    sources.generatedBottomLifecycle.includes("set_visible(GENERATED_BOTTOM_DRAWER_CONTROL_ID, false)") &&
    !sources.generatedBottomLifecycle.includes("set_visible(GENERATED_BOTTOM_PANEL_CONTROL_ID, false)"),
  "generated bottom lifecycle owns drawer visibility transitions",
);
check(
  sources.moduleFieldEdit.includes('WORKBENCH_GENERATED_BOTTOM_BINDING_PREFIX: &str = "WorkbenchGeneratedBottom/"'),
  "field edit path accepts generated bottom edit bindings",
);

for (const event of generatedBottomEvents) {
  check(Boolean(event.controlId), `${event.bindingId} has no control_id`);
  check(event.route.startsWith("workbench.generated_bottom."), `${event.bindingId} route ${event.route} is outside workbench.generated_bottom.*`);
  check(["Click", "Change", "Submit"].includes(event.eventKind), `${event.bindingId} uses unsupported event ${event.eventKind}`);
  const binding = generatedBottomBindings.get(event.bindingId);
  check(Boolean(binding), `${event.bindingId} is declared in ZUI but missing from native generated-bottom bindings`);
  if (binding) {
    check(binding.eventKind === event.eventKind, `${event.bindingId} is ${event.eventKind} in ZUI but ${binding.eventKind} in native bindings`);
    check(generatedBottomPreviewActions.has(binding.actionId), `${event.bindingId} resolves to unregistered generated-bottom action ${binding.actionId}`);
  }
}

for (const [bindingId, binding] of generatedBottomBindings) {
  check(eventBindingIds.has(bindingId), `${bindingId} exists in generated-bottom native bindings but has no ZUI declaration`);
  check(generatedBottomPreviewActions.has(binding.actionId), `${binding.actionId} is not registered as generated-bottom preview action`);
  check(
    sources.generatedBottomNavigation.includes(binding.actionId) ||
      sources.generatedBottomFeedback.includes(binding.actionId),
    `${binding.actionId} has no retained generated-bottom route or feedback evidence`,
  );
}

for (const actionId of generatedBottomPreviewActions) {
  check(bindingActionIds.has(actionId), `${actionId} is registered as generated-bottom preview action but has no native binding`);
}

for (const target of routeTargets) {
  check(eventBindingIds.has(`WorkbenchGeneratedBottom/${bindingKeyFromRouteControl(target.controlId)}`), `${target.actionId} route row has no ZUI binding id`);
  check(routeTargetByActionId.get(target.actionId) === target, `${target.actionId} should be unique in generated bottom route targets`);
  check(target.modeControlId.startsWith("WorkbenchGeneratedBottomMode"), `${target.actionId} has invalid mode tab ${target.modeControlId}`);
}

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(`fail ${failure}`);
  }
  process.exit(1);
}

console.log(
  `native generated bottom panel contract: webRoutes=${webGeneratedBottomRoutes.length} routeTargets=${routeTargets.length} events=${generatedBottomEvents.length} bindings=${generatedBottomBindings.size} generatedBottomPreviewActions=${generatedBottomPreviewActions.size}`,
);
console.log("ok native generated bottom panel contract");

function generatedBottomRoutesFromWeb() {
  const routes = [];
  for (const module of nativeModules) {
    const bottom = module.bottom();
    for (const match of bottom.matchAll(/data-generated-bottom-panel="([^"]+)"/g)) {
      routes.push(match[1]);
    }
  }
  return routes;
}

function generatedBottomEventsFromZui(sourceName, source) {
  const events = [];
  for (const block of nodeBlocksFromZui(source)) {
    const nodeName = block.match(/^\[nodes\.([^\]]+)\]$/m)?.[1];
    if (!nodeName) {
      continue;
    }
    const controlId = block.match(/^\s*control_id\s*=\s*"([^"]+)"/m)?.[1] ?? "";
    const component = block.match(/^\s*component\s*=\s*"([^"]+)"/m)?.[1] ?? "";
    for (const eventMatch of block.matchAll(
      /\{\s*id\s*=\s*"(WorkbenchGeneratedBottom\/[^"]+)"\s*,\s*event\s*=\s*"([^"]+)"([^}]*)\}/g,
    )) {
      const [, bindingId, eventKind, tail] = eventMatch;
      const route = tail.match(/route\s*=\s*"([^"]+)"/)?.[1] ?? "";
      events.push({ sourceName, nodeName, component, controlId, bindingId, eventKind, route });
    }
  }
  return events;
}

function generatedBottomBindingsFromRust(source) {
  const bindings = new Map();
  for (const match of source.matchAll(/\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,?\s*\)/g)) {
    const [, controlKey, actionId] = match;
    if (isGeneratedBottomPreviewAction(actionId)) {
      bindings.set(`WorkbenchGeneratedBottom/${controlKey}`, { eventKind: "Click", actionId });
    }
  }
  for (const match of source.matchAll(
    /\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*EditorUiEventKind::(Change|Submit)\s*,?\s*\)/g,
  )) {
    const [, controlKey, actionId, eventKind] = match;
    if (isGeneratedBottomPreviewAction(actionId)) {
      bindings.set(`WorkbenchGeneratedBottom/${controlKey}`, { eventKind, actionId });
    }
  }
  return bindings;
}

function generatedBottomRouteTargetsFromRust(source) {
  const targets = [];
  for (const match of source.matchAll(/GeneratedBottomRouteTarget\s*\{([\s\S]*?)\}/g)) {
    const block = match[1];
    const actionId = rustFieldString(block, "action_id");
    const controlId = rustFieldString(block, "control_id");
    const panelRoute = rustFieldString(block, "panel_route");
    const moduleLabel = rustFieldString(block, "module_label");
    const panelLabel = rustFieldString(block, "panel_label");
    const modeControlId = rustFieldString(block, "mode_control_id");
    if (!actionId || !controlId || !panelRoute || !moduleLabel || !panelLabel || !modeControlId) {
      continue;
    }
    targets.push({ actionId, controlId, panelRoute, moduleLabel, panelLabel, modeControlId });
  }
  return targets;
}

function rustFieldString(block, fieldName) {
  return block.match(new RegExp(`${fieldName}:\\s*"([^"]+)"`))?.[1] ?? "";
}

function previewActionsFromRust(source) {
  const ids = new Set();
  const invalidIds = new Set();
  for (const listMatch of source.matchAll(/PREVIEW_ACTION_IDS:\s*&\[&str\]\s*=\s*&\[([\s\S]*?)\];/g)) {
    for (const idMatch of listMatch[1].matchAll(/"([^"]+)"/g)) {
      const id = idMatch[1];
      if (!previewActionIdPattern.test(id)) {
        invalidIds.add(id);
        continue;
      }
      ids.add(id);
    }
  }
  if (invalidIds.size > 0) {
    throw new Error(
      `preview action ids must use dotted functional paths: ${[...invalidIds].join(", ")}`,
    );
  }
  if (ids.size === 0) {
    throw new Error("preview action arrays were not found");
  }
  return ids;
}

function isGeneratedBottomPreviewAction(actionId) {
  return actionId.startsWith("workbench.generated_bottom.");
}

function nodeBlocksFromZui(source) {
  return source.split(/(?=^\[nodes\.[^\]]+\]$)/gm);
}

function bindingKeyFromRouteControl(controlId) {
  return controlId
    .replace(/^WorkbenchGeneratedBottom/, "")
    .replace(/Row$/, "");
}

function readLocal(path) {
  return readFileSync(new URL(path, import.meta.url), "utf8");
}

function readRepo(path) {
  return readLocal(path);
}

function check(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}
