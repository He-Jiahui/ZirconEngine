import { readFileSync, readdirSync } from "node:fs";
import { extname } from "node:path";
import { fileURLToPath } from "node:url";
import { inspector, popups, rail, scenePanel, showcase, statusbar, topbar, workbenchWindow } from "./src/components/surfaces/surfaces.js";
import { defaultModuleId, extensionModules, webModuleTabs, moduleWorkspace } from "./src/modules/modules.js";
import { actionPath, actionRouteKey, normalizeActionId } from "./src/foundation/action-paths.js";

const dottedActionPattern = /^[a-z0-9_]+(?:\.[a-z0-9_]+)+$/;
const webActionRoots = ["workbench."];
const nativeActionSourceExtensions = new Set([".rs", ".toml", ".zui"]);
const nativeActionSourceRoots = [
  "../../../../zircon_editor/src/core/",
  "../../../../zircon_editor/src/ui/",
  "../../../../zircon_editor/assets/ui/editor/",
];

const nativeActionPatterns = [
  ["menu action payload", /EditorUiBindingPayload::menu_action\(\s*"([^"]+)"/g],
  ["menu action helper", /\bmenu_action\(\s*"([^"]+)"/g],
  ["editor operation payload", /EditorUiBindingPayload::editor_operation\(\s*"([^"]+)"/g],
  ["editor operation parse", /EditorOperationPath::parse\(\s*"([^"]+)"/g],
  ["editor operation helper", /\boperation\(\s*"([^"]+)"/g],
  ["parse operation helper", /\bparse_operation\(\s*"([^"]+)"/g],
  ["operation control response", /EditorOperationControlResponse::success\(\s*"([^"]+)"/g],
  [
    "template action field",
    /\b(?:action_id|edit_action_id|commit_action_id|primary_action_id|secondary_action_id|feature_action_id|packaging_action_id|target_modes_action_id|unload_action_id|hot_reload_action_id)\s*[:=]\s*"([^"]+)"/g,
  ],
  ["reflection action descriptor", /UiActionDescriptor::new\(\s*"([^"]+)"/g],
  ["zui binding route", /\broute\s*=\s*"([^"]+)"/g],
  ["template action route", /\b((?:menu_action|editor_action|MenuAction|EditorAction)\.[A-Za-z0-9_.]+)/g],
  ["action id helper prefix", /\b[A-Za-z_]*action_id\(\s*"([^"]+)"/g],
  [
    "action id format template",
    /\b(?:action_id|edit_action_id|commit_action_id|primary_action_id|secondary_action_id|feature_action_id|packaging_action_id|target_modes_action_id|unload_action_id|hot_reload_action_id)\s*:\s*format!\(\s*"([^"]+)"/g,
  ],
];

const forbiddenNativeActionPattern =
  /^(?:File|Window|Scene|Runtime|Edit|Inspector|View|Weather|Authoring|Sdk|Editor|Tools|MenuAction|EditorAction|Route)\./;

const failures = [];
const actionPathsSource = readLocal("./src/foundation/action-paths.js");
const routesSource = [
  "./src/routing/routes.js",
  "./src/routing/commands/route-for-command.js"
].map(readLocal).join("\n");
const appSource = [
  "./app.js",
  "./src/app/controller.js",
  "./src/app/controller/activation.js",
  "./src/app/controller/command-application.js",
  "./src/app/controller/command-application/apply.js",
  "./src/app/controller/command-application/module.js",
  "./src/app/controller/command-application/panel.js",
  "./src/app/controller/command-application/record.js",
  "./src/app/controller/command-application/status.js",
  "./src/app/controller/create-workbench-controller.js",
  "./src/app/controller/command-routing.js",
  "./src/app/controller/command-routing/explicit.js",
  "./src/app/controller/command-routing/fallback.js",
  "./src/app/controller/command-routing/label.js",
  "./src/app/controller/command-routing/resolve.js",
  "./src/app/controller/history.js",
  "./src/app/controller/location-state.js",
  "./src/app/controller/rendering.js",
  "./src/app/controller/state.js",
  "./src/app/controller/status.js",
  "./src/app/route-state.js",
  "./src/app/interactions/click.js",
  "./src/app/interactions/click/bind.js",
  "./src/app/interactions/click/dispatch.js",
  "./src/app/interactions/click/handlers.js",
  "./src/app/interactions/click/actions.js",
  "./src/app/interactions/click/actions/feedback.js",
  "./src/app/interactions/click/actions/group.js",
  "./src/app/interactions/click/actions/handle.js",
  "./src/app/interactions/click/actions/menu.js",
  "./src/app/interactions/click/actions/target.js",
  "./src/app/interactions/click/dropdowns.js",
  "./src/app/interactions/click/dropdowns/dismissal.js",
  "./src/app/interactions/click/dropdowns/feedback.js",
  "./src/app/interactions/click/dropdowns/placement.js",
  "./src/app/interactions/click/dropdowns/state.js",
  "./src/app/interactions/click/dropdowns/target.js",
  "./src/app/interactions/click/dropdowns/trigger.js",
  "./src/app/interactions/click/generic.js",
  "./src/app/interactions/click/generic/feedback.js",
  "./src/app/interactions/click/generic/handle.js",
  "./src/app/interactions/click/generic/target.js",
  "./src/app/interactions/click/navigation.js",
  "./src/app/interactions/click/navigation/activate.js",
  "./src/app/interactions/click/navigation/handle.js",
  "./src/app/interactions/click/navigation/target.js",
  "./src/app/interactions/click/rows.js",
  "./src/app/interactions/click/rows/data.js",
  "./src/app/interactions/click/rows/feedback.js",
  "./src/app/interactions/click/rows/selection.js",
  "./src/app/interactions/click/rows/tree.js",
  "./src/app/interactions/click/selection.js",
  "./src/app/interactions/click/selection/feedback.js",
  "./src/app/interactions/click/selection/radio.js",
  "./src/app/interactions/click/selection/state.js",
  "./src/app/interactions/click/selection/target.js",
  "./src/app/interactions/click/selection/toggle.js",
  "./src/app/interactions/click/tabs.js",
  "./src/app/interactions/click/tabs/feedback.js",
  "./src/app/interactions/click/tabs/handle.js",
  "./src/app/interactions/click/tabs/panel.js",
  "./src/app/interactions/click/tabs/state.js",
  "./src/app/interactions/click/tabs/target.js",
  "./src/app/interactions/click/toolbar.js",
  "./src/app/interactions/click/toolbar/feedback.js",
  "./src/app/interactions/click/toolbar/rail.js",
  "./src/app/interactions/click/toolbar/state.js",
  "./src/app/interactions/click/toolbar/target.js",
  "./src/app/interactions/click/toolbar/tool.js",
  "./src/app/interactions/click/utils.js",
  "./src/app/interactions/fields.js",
  "./src/app/interactions/fields/bind.js",
  "./src/app/interactions/fields/focus.js",
  "./src/app/interactions/fields/input.js",
  "./src/app/interactions/fields/target.js",
  "./src/app/interactions/keyboard.js",
  "./src/app/interactions/keyboard/activate.js",
  "./src/app/interactions/keyboard/bind.js",
  "./src/app/interactions/keyboard/filter.js",
  "./src/app/interactions/keyboard/target.js",
  "./src/app/interactions/history.js",
  "./src/app/interactions/history/bind.js",
  "./src/app/interactions/history/events.js"
].map(readLocal).join("\n");
const controlRouteSource = readLocal("./validate-control-routes.mjs");
const readmeSource = readLocal("./README.md");
const handoffMatrixSource = readLocal("./web-native-handoff-matrix.md");
const webActionRecords = collectRenderedWebActionIds();
const nativeActionRecords = collectNativeActionIds();
const forbiddenNativeActionRecords = nativeActionRecords.filter(({ actionId }) =>
  forbiddenNativeActionPattern.test(actionId),
);
const invalidWebActionIds = webActionRecords.filter(
  ({ actionId }) => !dottedActionPattern.test(actionId) || !webActionRoots.some((root) => actionId.startsWith(root)),
);
const invalidNativeActionIds = nativeActionRecords.filter(({ actionId }) => !dottedActionPattern.test(actionId));
const allActionIds = [
  ...webActionRecords.map((record) => record.actionId),
  ...nativeActionRecords.map((record) => record.actionId),
];
const uniqueActionIds = new Set(allActionIds);

check(webActionRecords.length >= 3000, `expected at least 3000 rendered browser actions, found ${webActionRecords.length}`);
check(invalidWebActionIds.length === 0, formatInvalid("browser data-action", invalidWebActionIds));
check(nativeActionRecords.length >= 1200, `expected at least 1200 native action ids, found ${nativeActionRecords.length}`);
check(invalidNativeActionIds.length === 0, formatInvalid("native action id", invalidNativeActionIds));
check(
  forbiddenNativeActionRecords.length === 0,
  formatInvalid("legacy native action id", forbiddenNativeActionRecords),
);
check(
  [
    "workbench.module.toolbar",
    "workbench.module.table",
    "workbench.collection.menu",
    "workbench.generated_bottom",
  ].every((prefix) => webActionRecords.some(({ actionId }) => actionId.startsWith(prefix))),
  "rendered browser actions must cover module toolbar, module table, collection menu, and generated-bottom namespaces",
);
check(
  [
    "workbench.module.",
    "workbench.extension.",
    "workbench.generated_bottom.",
    "component_lab.",
  ].every((prefix) => nativeActionRecords.some(({ actionId }) => actionId.startsWith(prefix))),
  "native action ids must cover module, extension, generated-bottom, and component-lab namespaces",
);
check(
  actionPathsSource.includes("export function actionPath") &&
    actionPathsSource.includes("export function actionRouteKey") &&
    actionPathsSource.includes("export function normalizeActionId") &&
    routesSource.includes("actionRouteKey") &&
    appSource.includes("normalizeActionId"),
  "browser action naming must stay centralized in action-paths.js with route-key extraction",
);
check(
  appSource.includes('params.get("action")') &&
    appSource.includes('params.get("command")') &&
    appSource.includes('params.set("action",') &&
    !appSource.includes('params.set("command",') &&
    controlRouteSource.includes('baselineParams.set("action"') &&
    !controlRouteSource.includes('baselineParams.set("command"'),
  "browser route hashes must write dotted action ids through action=, while command= stays read-only legacy input",
);
check(
  readmeSource.includes("node verify-action-naming-contract.mjs") &&
    handoffMatrixSource.includes("node verify-action-naming-contract.mjs"),
  "README and handoff matrix must record the action naming contract verifier",
);
check(
  [
    "component_lab.button.primary",
    "component_lab.button.secondary",
    "component_lab.button.tertiary",
    "component_lab.button.outline",
    "component_lab.button.icon",
    "component_lab.button.delete",
  ].every((actionId) => uniqueActionIds.has(actionId)),
  "Workbench ComponentLab preview buttons must expose dotted component_lab.button.* actions",
);
check(
  !allActionIds.some((actionId) => /[A-Z/]/.test(actionId)),
  "action ids must not use CamelCase or slash-delimited namespaces",
);
check(
  uniqueActionIds.has("workbench.module.table.health_regen") &&
    uniqueActionIds.has("workbench.module.table.incoming_healing") &&
    !uniqueActionIds.has("workbench.module.table.healthregen") &&
    !uniqueActionIds.has("workbench.module.table.incominghealing"),
  "CamelCase labels must split into readable action segments such as health_regen, not flattened healthregen",
);
check(
  actionPath("workbench.tree.select", "GE_HealthRegen") === "workbench.tree.select.ge_health_regen" &&
    actionRouteKey("IncomingHealing") === "incoming-healing" &&
    normalizeActionId("workbench.module.table.HealthRegen") === "workbench.module.table.health_regen",
  "action path helpers must preserve CamelCase word boundaries before lower-case normalization",
);

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(`fail ${failure}`);
  }
  process.exit(1);
}

console.log(
  `action naming contract: webActions=${webActionRecords.length} nativeActions=${nativeActionRecords.length} uniqueActions=${uniqueActionIds.size}`,
);
console.log("ok action naming contract");

function collectRenderedWebActionIds() {
  const html = [
    workbenchWindow([topbar(defaultModuleId), rail(defaultModuleId), moduleWorkspace(defaultModuleId), statusbar("Ready"), popups()]),
    scenePanel(),
    inspector(),
    showcase(),
    moduleWorkspace("editor-library"),
    ...webModuleTabs.map((module) => moduleWorkspace(module.id)),
    ...extensionModules.map((module) => moduleWorkspace(module.id)),
  ].join("\n");
  return Array.from(html.matchAll(/data-action="([^"]+)"/g), (match) => ({
    sourceName: "rendered web component tree",
    kind: "data-action",
    actionId: match[1],
  }));
}

function collectNativeActionIds() {
  const records = [];
  for (const fileUrl of nativeActionSourceFiles()) {
    const source = readFileSync(fileUrl, "utf8");
    const sourceName = sourceLabel(fileUrl);
    for (const [kind, pattern] of nativeActionPatterns) {
      for (const match of source.matchAll(pattern)) {
        addNativeActionRecord(records, sourceName, kind, match[1]);
      }
    }
    for (const listMatch of source.matchAll(/PREVIEW_ACTION_IDS:\s*&\[&str\]\s*=\s*&\[([\s\S]*?)\];/g)) {
      for (const idMatch of listMatch[1].matchAll(/"([^"]+)"/g)) {
        addNativeActionRecord(records, sourceName, "preview action array", idMatch[1]);
      }
    }
  }
  return dedupeRecords(records);
}

function addNativeActionRecord(records, sourceName, kind, rawActionId) {
  const actionId = normalizeRustActionTemplate(rawActionId);
  if (isPartialActionTemplate(actionId)) {
    return;
  }
  records.push({ sourceName, kind, actionId });
}

function normalizeRustActionTemplate(actionId) {
  return String(actionId ?? "")
    .trim()
    .replace(/\{[^}]*\}/g, "template");
}

function isPartialActionTemplate(actionId) {
  return !actionId || actionId.endsWith(".");
}

function dedupeRecords(records) {
  const byKey = new Map();
  for (const record of records) {
    byKey.set(`${record.sourceName}:${record.kind}:${record.actionId}`, record);
  }
  return [...byKey.values()];
}

function nativeActionSourceFiles() {
  return nativeActionSourceRoots.flatMap((root) => collectNativeActionSourceFiles(new URL(root, import.meta.url)));
}

function collectNativeActionSourceFiles(rootUrl) {
  const files = [];
  for (const entry of readdirSync(rootUrl, { withFileTypes: true })) {
    const childUrl = new URL(`${entry.name}${entry.isDirectory() ? "/" : ""}`, rootUrl);
    if (entry.isDirectory()) {
      files.push(...collectNativeActionSourceFiles(childUrl));
      continue;
    }
    if (nativeActionSourceExtensions.has(extname(entry.name))) {
      files.push(childUrl);
    }
  }
  return files;
}

function sourceLabel(fileUrl) {
  const repoRoot = fileURLToPath(new URL("../../../../", import.meta.url)).replaceAll("\\", "/");
  const filePath = fileURLToPath(fileUrl).replaceAll("\\", "/");
  return filePath.startsWith(repoRoot) ? filePath.slice(repoRoot.length) : filePath;
}

function formatInvalid(scope, records) {
  if (records.length === 0) {
    return "";
  }
  return `${scope} values must use lower-case dotted functional paths, not CamelCase, slugs, or slash paths: ${records
    .slice(0, 30)
    .map(({ sourceName, kind, actionId }) => `${sourceName} ${kind} ${actionId}`)
    .join("; ")}`;
}

function readLocal(path) {
  return readFileSync(new URL(path, import.meta.url), "utf8");
}

function check(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}
