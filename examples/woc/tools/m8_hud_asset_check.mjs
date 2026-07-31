import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const hudDir = join(repoRoot, "examples", "woc", "assets", "ui", "hud");
const assetPaths = [
  join(hudDir, "hud_theme.zui"),
  join(hudDir, "in_world_hud.zui"),
  join(hudDir, "lockpick_window.zui"),
];
const allowedComponents = new Set([
  "Button",
  "Container",
  "GridGroup",
  "HorizontalBox",
  "Icon",
  "Label",
  "Overlay",
  "Space",
  "Stack",
  "VerticalBox",
]);

const pythonLoader = String.raw`
import json
import pathlib
import sys
import tomllib

documents = []
for raw_path in sys.argv[1:]:
    path = pathlib.Path(raw_path)
    with path.open("rb") as stream:
        document = tomllib.load(stream)
    documents.append({"path": str(path), "document": document})
print(json.dumps(documents, separators=(",", ":")))
`;

function fail(message) {
  throw new Error(`M8 in-world HUD asset check failed: ${message}`);
}

function expect(condition, message) {
  if (!condition) fail(message);
}

function parseAssets() {
  const result = spawnSync("python", ["-c", pythonLoader, ...assetPaths], {
    cwd: repoRoot,
    encoding: "utf8",
    windowsHide: true,
  });
  if (result.error) fail(`could not launch Python tomllib: ${result.error.message}`);
  if (result.status !== 0) fail(`Python tomllib exited ${result.status}: ${result.stderr.trim()}`);
  try {
    return JSON.parse(result.stdout);
  } catch (error) {
    fail(`Python tomllib returned invalid JSON: ${error.message}`);
  }
}

function routes(node) {
  return (node.events ?? []).map((event) => event.route);
}

function expectRoutes(node, expected, label) {
  expect(JSON.stringify(routes(node)) === JSON.stringify(expected), `${label} routes ${JSON.stringify(routes(node))}`);
}

function validateTheme(document) {
  expect(document.asset?.kind === "style", "HUD theme must be a style asset");
  expect(document.asset?.id === "res://ui/hud/hud_theme.zui", "HUD theme id mismatch");
  expect(document.asset?.version === 2, "HUD theme must use schema version 2");
  expect(
    JSON.stringify(document.imports?.styles) === JSON.stringify(["res://ui/shell/shell_theme.zui"]),
    "HUD theme must extend only the WOC shell theme",
  );
  for (const token of [
    "woc_hud_panel",
    "woc_hud_health",
    "woc_hud_absorb",
    "woc_hud_mana",
    "woc_hud_rage",
    "woc_hud_energy",
    "woc_hud_cooldown",
    "woc_hud_danger",
  ]) {
    expect(token in (document.tokens ?? {}), `HUD theme misses token ${token}`);
  }

  const selectors = new Set();
  for (const stylesheet of document.stylesheets ?? []) {
    for (const rule of stylesheet.rules ?? []) {
      expect(!selectors.has(rule.selector), `HUD theme duplicates selector ${rule.selector}`);
      selectors.add(rule.selector);
      const stack = [rule.set];
      while (stack.length > 0) {
        const value = stack.pop();
        if (typeof value === "string" && value.startsWith("$")) {
          const token = value.slice(1);
          expect(token in document.tokens || token.startsWith("woc_"), `${rule.selector} references invalid token ${value}`);
        } else if (Array.isArray(value)) {
          stack.push(...value);
        } else if (value && typeof value === "object") {
          stack.push(...Object.values(value));
        }
      }
    }
  }
  for (const selector of [
    ".woc-hud-root",
    ".woc-unit-frame",
    ".woc-health-fill",
    ".woc-resource-fill",
    ".woc-action-slot",
    ".woc-action-slot:focus",
    ".woc-action-slot:disabled",
    ".woc-touch-action",
    ".woc-pause-menu",
    ".woc-low-health-vignette",
    ".woc-lockpick-window",
    ".woc-lockpick-ante",
    ".woc-lockpick-board",
    ".woc-lockpick-timer",
    ".woc-lockpick-action",
  ]) {
    expect(selectors.has(selector), `HUD theme misses selector ${selector}`);
  }
  return selectors.size;
}

function validateGraph(document, expectedId = "res://ui/hud/in_world_hud.zui") {
  expect(document.asset?.kind === "view", `${expectedId} must be a view asset`);
  expect(document.asset?.id === expectedId, `${expectedId} id mismatch`);
  expect(document.asset?.version === 2, `${expectedId} must use schema version 2`);
  expect(
    JSON.stringify(document.imports?.styles) ===
      JSON.stringify(["res://ui/shell/shell_theme.zui", "res://ui/hud/hud_theme.zui"]),
    "in-world HUD style imports differ from the project contract",
  );

  const nodes = document.nodes ?? {};
  const root = document.root?.node;
  expect(typeof root === "string" && root in nodes, "in-world HUD root is missing");
  const reachable = new Set();
  const visiting = new Set();
  function visit(nodeId) {
    expect(nodeId in nodes, `in-world HUD references missing node ${nodeId}`);
    if (visiting.has(nodeId)) fail(`in-world HUD contains a cycle at ${nodeId}`);
    if (reachable.has(nodeId)) return;
    visiting.add(nodeId);
    reachable.add(nodeId);
    for (const child of nodes[nodeId].children ?? []) visit(child.node);
    visiting.delete(nodeId);
  }
  visit(root);
  const unreachable = Object.keys(nodes).filter((nodeId) => !reachable.has(nodeId));
  expect(unreachable.length === 0, `in-world HUD has unreachable nodes: ${unreachable.join(", ")}`);

  const controls = new Set();
  const events = new Set();
  for (const [nodeId, node] of Object.entries(nodes)) {
    expect(allowedComponents.has(node.component), `${nodeId} uses unaudited component ${node.component}`);
    if (node.control_id) {
      expect(!controls.has(node.control_id), `in-world HUD duplicates control ${node.control_id}`);
      controls.add(node.control_id);
    }
    for (const event of node.events ?? []) {
      expect(["Click", "Change", "Submit", "Toggle"].includes(event.event), `${event.id} uses unsupported event ${event.event}`);
      expect(!events.has(event.id), `in-world HUD duplicates event ${event.id}`);
      events.add(event.id);
      expect(event.route.startsWith("woc.hud."), `${event.id} route escapes the WOC HUD namespace`);
    }
  }
  return { nodes, controls, events };
}

function validateDesktop(nodes) {
  expect(
    JSON.stringify(nodes.desktop_hud.props?.visibility) === JSON.stringify({ xs: "collapsed", md: "visible" }),
    "desktop HUD breakpoint gate is wrong",
  );
  expect(
    JSON.stringify(nodes.desktop_hud.children.map((child) => child.node)) ===
      JSON.stringify(["target_frame", "tracker_stack", "minimap_cluster", "desktop_bottom", "low_health_vignette"]),
    "desktop HUD layer order differs from the target",
  );
  expect(nodes.target_frame.props?.visibility === "collapsed", "target frame must start hidden without a target");
  expect(nodes.target_of_target.props?.visibility === "collapsed", "target-of-target must start hidden");
  expect(nodes.secondary_action_bar.props?.visibility === "collapsed", "secondary bar must start disabled");

  const primary = Array.from({ length: 12 }, (_, index) => `action_slot_${index}`);
  const secondary = Array.from({ length: 11 }, (_, index) => `action_slot_${index + 12}`);
  expect(
    JSON.stringify(nodes.primary_action_bar.children.map((child) => child.node)) === JSON.stringify(primary),
    "primary action bar must contain target slots 0..11",
  );
  expect(
    JSON.stringify(nodes.secondary_action_bar.children.map((child) => child.node)) === JSON.stringify(secondary),
    "secondary action bar must contain target slots 12..22",
  );
  for (let index = 0; index < 23; index += 1) {
    const node = nodes[`action_slot_${index}`];
    expect(node.component === "Button", `desktop action slot ${index} must be a Button`);
    expect(node.layout.width.min >= 44 && node.layout.height.min >= 44, `desktop action slot ${index} is below target size`);
    expectRoutes(node, [`woc.hud.action.activate.${index}`], `desktop action slot ${index}`);
  }

  for (const nodeId of [
    "player_frame",
    "target_frame",
    "target_of_target",
    "player_health",
    "player_absorb",
    "player_resource",
    "target_health",
    "target_absorb",
    "target_resource",
    "player_cast_bar",
    "target_cast_bar",
    "xp_bar",
    "quest_tracker",
    "minimap_cluster",
  ]) {
    expect(nodeId in nodes, `desktop HUD misses ${nodeId}`);
  }
  expect(
    nodes.target_frame.children.some((child) => child.node === "target_of_target"),
    "target-of-target must remain owned by the target frame",
  );
}

function validateTouch(nodes) {
  expect(
    JSON.stringify(nodes.touch_hud.props?.visibility) === JSON.stringify({ xs: "visible", md: "collapsed" }),
    "touch HUD breakpoint gate is wrong",
  );
  const expectedRing = [
    "touch_action_0",
    "touch_action_1",
    "touch_action_2",
    "touch_action_3",
    "touch_action_4",
    "touch_attack",
    "touch_target",
    "touch_interact",
    "touch_jump",
    "touch_page",
  ];
  expect(
    JSON.stringify(nodes.touch_action_ring.children.map((child) => child.node)) === JSON.stringify(expectedRing),
    "touch action ring order differs from the target",
  );
  for (let index = 0; index < 5; index += 1) {
    expectRoutes(nodes[`touch_action_${index}`], [`woc.hud.touch.activate.${index}`], `touch action ${index}`);
  }
  expectRoutes(nodes.touch_attack, ["woc.hud.touch.attack"], "touch attack");
  expectRoutes(nodes.touch_target, ["woc.hud.touch.target_cycle"], "touch target cycle");
  expectRoutes(nodes.touch_interact, ["woc.hud.touch.interact"], "touch interact");
  expectRoutes(nodes.touch_jump, ["woc.hud.touch.jump"], "touch jump");
  expectRoutes(nodes.touch_page, ["woc.hud.touch.next_page"], "touch action page");
  expect(nodes.touch_page.props?.text === "1 / 2", "touch action page must start at page 1 of 2");

  const consumables = Array.from({ length: 6 }, (_, index) => `touch_consumable_${index}`);
  expect(
    JSON.stringify(nodes.touch_consumables.children.map((child) => child.node)) === JSON.stringify(consumables),
    "touch consumable bar must expose six ordered slots",
  );
  consumables.forEach((nodeId, index) => {
    expectRoutes(nodes[nodeId], [`woc.hud.touch.consume.${index}`], `touch consumable ${index}`);
  });
  expect(nodes.touch_target_frame.props?.visibility === "collapsed", "touch target frame must start hidden");
}

function validatePause(nodes) {
  expect(nodes.pause_layer.props?.visibility === "collapsed", "pause layer must start collapsed");
  const menuOrder = [
    "pause_keybinds",
    "pause_controller",
    "pause_graphics",
    "pause_interface",
    "pause_audio",
    "pause_performance",
    "pause_bug_report",
    "pause_logout",
    "pause_return",
  ];
  expect(
    JSON.stringify(nodes.pause_menu.children.map((child) => child.node)) === JSON.stringify(menuOrder),
    "pause menu order differs from the target",
  );
  expect(nodes.pause_bug_report.props?.visibility === "collapsed", "online-only bug report must start gated");
  const routesByNode = {
    pause_keybinds: "woc.hud.pause.open.keybinds",
    pause_controller: "woc.hud.pause.open.controller",
    pause_graphics: "woc.hud.pause.open.graphics",
    pause_interface: "woc.hud.pause.open.interface",
    pause_audio: "woc.hud.pause.open.audio",
    pause_performance: "woc.hud.pause.open.performance",
    pause_bug_report: "woc.hud.pause.open.bug_report",
    pause_logout: "woc.hud.pause.logout",
    pause_return: "woc.hud.pause.return_to_game",
  };
  for (const [nodeId, route] of Object.entries(routesByNode)) expectRoutes(nodes[nodeId], [route], nodeId);
}

function validateLockpick(document) {
  const { nodes, controls, events } = validateGraph(document, "res://ui/hud/lockpick_window.zui");
  expect(document.asset?.id === "res://ui/hud/lockpick_window.zui", "lockpick window id mismatch");
  expect(
    JSON.stringify(document.imports?.styles) ===
      JSON.stringify(["res://ui/shell/shell_theme.zui", "res://ui/hud/hud_theme.zui"]),
    "lockpick window styles differ from the HUD contract",
  );
  expect(nodes.lockpick_root.component === "Container", "lockpick root must be a Container");
  expect(nodes.lockpick_root.props?.visibility === "collapsed", "lockpick root must start hidden");
  expect(
    JSON.stringify(nodes.lockpick_root.children.map((child) => child.node)) ===
      JSON.stringify(["lockpick_selector", "lockpick_live"]),
    "lockpick root layer order is wrong",
  );
  expect(nodes.lockpick_live.props?.visibility === "collapsed", "live lockpick board must await authoritative state");
  expect(nodes.lockpick_board_host.component === "GridGroup", "lockpick board must use a dynamic track host");
  expect(
    nodes.lockpick_board_host.props?.dynamic_route_prefix === "woc.hud.lockpick.track",
    "lockpick track host route prefix is wrong",
  );
  expect(nodes.lockpick_board_host.children.length === 0, "lockpick tracks must remain host-projected");
  expect(
    JSON.stringify(nodes.lockpick_ante_row.children.map((child) => child.node)) ===
      JSON.stringify(["lockpick_ante_premium", "lockpick_ante_medium", "lockpick_ante_low"]),
    "lockpick ante order differs from the source",
  );
  const anteRoutes = [
    ["lockpick_ante_premium", "woc.hud.lockpick.engage.1"],
    ["lockpick_ante_medium", "woc.hud.lockpick.engage.2"],
    ["lockpick_ante_low", "woc.hud.lockpick.engage.3"],
  ];
  for (const [nodeId, route] of anteRoutes) {
    expectRoutes(nodes[nodeId], [route], nodeId);
    expect(nodes[nodeId].layout.height.min >= 44, `${nodeId} must remain touch-sized`);
  }
  const actionIds = ["hard_set", "set", "steady", "ease", "drop"];
  expect(
    JSON.stringify(nodes.lockpick_actions.children.map((child) => child.node)) ===
      JSON.stringify(actionIds.map((id) => `lockpick_action_${id}`)),
    "lockpick action order differs from the source",
  );
  for (const action of actionIds) {
    const nodeId = `lockpick_action_${action}`;
    expectRoutes(nodes[nodeId], [`woc.hud.lockpick.action.${action}`], nodeId);
    expect(nodes[nodeId].layout.height.min >= 44, `${nodeId} must remain touch-sized`);
  }
  expectRoutes(nodes.lockpick_abort, ["woc.hud.lockpick.abort"], "lockpick abort");
  expectRoutes(nodes.lockpick_selector_close, ["woc.hud.lockpick.close"], "lockpick selector close");
  for (const controlId of [
    "LockpickWindow",
    "LockpickSelector",
    "LockpickCofferHint",
    "LockpickLiveBoard",
    "LockpickPageStatus",
    "LockpickTriesStatus",
    "LockpickTimer",
    "LockpickTumblerTrackHost",
    "LockpickFeedback",
    "LockpickActions",
    "LockpickAbort",
  ]) {
    expect(controls.has(controlId), `lockpick window misses control ${controlId}`);
  }
  return { nodes, controls, events };
}

const parsed = parseAssets();
const byName = new Map(parsed.map(({ path, document }) => [path.replaceAll("\\", "/").split("/").at(-1), document]));
expect(byName.has("hud_theme.zui"), "tomllib result misses hud_theme.zui");
expect(byName.has("in_world_hud.zui"), "tomllib result misses in_world_hud.zui");
expect(byName.has("lockpick_window.zui"), "tomllib result misses lockpick_window.zui");

const selectorCount = validateTheme(byName.get("hud_theme.zui"));
const { nodes, controls, events } = validateGraph(byName.get("in_world_hud.zui"));
expect(nodes.hud_root.component === "Overlay", "in-world HUD root must be an Overlay");
expect(
  JSON.stringify(nodes.hud_root.children.map((child) => child.node)) ===
    JSON.stringify(["live_regions", "desktop_hud", "touch_hud", "pause_layer"]),
  "in-world HUD root layer order is wrong",
);
expect(
  JSON.stringify(nodes.live_regions.children.map((child) => child.node)) ===
    JSON.stringify(["combat_live", "target_live", "chat_live"]),
  "HUD live-region order differs from the target",
);
validateDesktop(nodes);
validateTouch(nodes);
validatePause(nodes);
const lockpick = validateLockpick(byName.get("lockpick_window.zui"));

const digest = createHash("sha256");
for (const path of assetPaths) {
  digest.update(relative(repoRoot, path).replaceAll("\\", "/"));
  digest.update("\0");
  digest.update(readFileSync(path));
  digest.update("\0");
}

console.log(
  JSON.stringify({
    schemaVersion: 2,
    assets: assetPaths.length,
    views: 2,
    nodes: Object.keys(nodes).length + Object.keys(lockpick.nodes).length,
    controlIds: controls.size + lockpick.controls.size,
    events: events.size + lockpick.events.size,
    desktopActionSlots: 23,
    touchActionSlots: 5,
    touchActionPages: 2,
    touchConsumableSlots: 6,
    styleSelectors: selectorCount,
    sha256: digest.digest("hex"),
  }),
);
