import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const windowDir = join(repoRoot, "examples", "woc", "assets", "ui", "windows");
const assetPaths = [
  join(windowDir, "inventory_window.zui"),
  join(windowDir, "quest_log_window.zui"),
  join(windowDir, "settings_window.zui"),
];
const allowedComponents = new Set([
  "Button",
  "Container",
  "GridGroup",
  "HorizontalBox",
  "Label",
  "Overlay",
  "ScrollBox",
  "Space",
  "TextField",
  "ToggleButton",
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
  throw new Error(`M8 client-window asset check failed: ${message}`);
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
  return JSON.parse(result.stdout);
}

function routes(node) {
  return (node.events ?? []).map((event) => event.route);
}

function expectRoutes(nodes, nodeId, expected) {
  expect(nodeId in nodes, `missing routed node ${nodeId}`);
  expect(
    JSON.stringify(routes(nodes[nodeId])) === JSON.stringify(expected),
    `${nodeId} routes ${JSON.stringify(routes(nodes[nodeId]))}`,
  );
}

function validateGraph(name, document) {
  expect(document && typeof document === "object", `tomllib result misses ${name}`);
  expect(document.asset?.kind === "view", `${name} must be a view asset`);
  expect(document.asset?.version === 2, `${name} must use schema version 2`);
  expect(
    document.asset?.id === `res://ui/windows/${name}`,
    `${name} asset id ${document.asset?.id}`,
  );
  expect(
    JSON.stringify(document.imports?.styles) ===
      JSON.stringify(["res://ui/shell/shell_theme.zui", "res://ui/hud/hud_theme.zui"]),
    `${name} style imports differ`,
  );

  const nodes = document.nodes ?? {};
  const root = document.root?.node;
  expect(typeof root === "string" && root in nodes, `${name} root is missing`);
  const reachable = new Set();
  const visiting = new Set();
  function visit(nodeId) {
    expect(nodeId in nodes, `${name} references missing node ${nodeId}`);
    if (visiting.has(nodeId)) fail(`${name} contains a cycle at ${nodeId}`);
    if (reachable.has(nodeId)) return;
    visiting.add(nodeId);
    reachable.add(nodeId);
    for (const child of nodes[nodeId].children ?? []) visit(child.node);
    visiting.delete(nodeId);
  }
  visit(root);
  const unreachable = Object.keys(nodes).filter((nodeId) => !reachable.has(nodeId));
  expect(unreachable.length === 0, `${name} has unreachable nodes: ${unreachable.join(", ")}`);

  const controls = new Set();
  const events = new Set();
  for (const [nodeId, node] of Object.entries(nodes)) {
    expect(allowedComponents.has(node.component), `${name}/${nodeId} uses ${node.component}`);
    if (node.control_id) {
      expect(!controls.has(node.control_id), `${name} duplicates control ${node.control_id}`);
      controls.add(node.control_id);
    }
    for (const event of node.events ?? []) {
      expect(["Click", "Change", "Submit", "Toggle"].includes(event.event), `${event.id} uses ${event.event}`);
      expect(!events.has(event.id), `${name} duplicates event ${event.id}`);
      events.add(event.id);
      expect(event.route.startsWith("woc.window."), `${event.id} escapes the window route namespace`);
    }
    if (["Button", "TextField", "ToggleButton"].includes(node.component)) {
      expect(node.layout?.height?.min >= 40, `${name}/${nodeId} is below the 40px touch floor`);
    }
  }
  expect(nodes[root].props?.visibility === "collapsed", `${name} must start closed`);
  return { nodes, controls, events };
}

function validateInventory(nodes) {
  const categories = ["all", "weapon", "armor", "consumable", "material", "quest"];
  expect(
    JSON.stringify(nodes.inventory_categories.children.map((child) => child.node)) ===
      JSON.stringify(categories.map((category) => `inventory_filter_${category}`)),
    "inventory category order differs from the target",
  );
  categories.forEach((category) => {
    expectRoutes(nodes, `inventory_filter_${category}`, [`woc.window.inventory.filter.${category}`]);
  });
  const sorts = ["recent", "quality", "name"];
  expect(
    JSON.stringify(nodes.inventory_sorts.children.map((child) => child.node)) ===
      JSON.stringify(sorts.map((sort) => `inventory_sort_${sort}`)),
    "inventory sort order differs from the target",
  );
  sorts.forEach((sort) => {
    expectRoutes(nodes, `inventory_sort_${sort}`, [`woc.window.inventory.sort.${sort}`]);
  });

  expect(nodes.inventory_search.component === "TextField", "inventory search must be a TextField");
  expectRoutes(nodes, "inventory_search", ["woc.window.inventory.search", "woc.window.inventory.search"]);
  expect(
    JSON.stringify(nodes.inventory_bag_bar.children.map((child) => child.node)) ===
      JSON.stringify([
        "inventory_backpack",
        "inventory_bag_socket_0",
        "inventory_bag_socket_1",
        "inventory_bag_socket_2",
        "inventory_bag_socket_3",
      ]),
    "inventory bag bar must contain the backpack plus four sockets",
  );
  for (let socket = 0; socket < 4; socket += 1) {
    expectRoutes(nodes, `inventory_bag_socket_${socket}`, [`woc.window.inventory.bag_socket.${socket}`]);
  }
  expect(nodes.inventory_grid.component === "ScrollBox", "inventory grid must scroll");
  expect(
    JSON.stringify(nodes.inventory_grid.children.map((child) => child.node)) ===
      JSON.stringify(["inventory_grid_host", "inventory_empty", "inventory_no_match"]),
    "inventory grid host/empty/no-match layers differ",
  );
  expect(nodes.inventory_grid_host.control_id === "InventoryGridHost", "inventory dynamic grid host id differs");
  expectRoutes(nodes, "inventory_close", ["woc.window.inventory.close"]);
}

function validateQuestLog(nodes) {
  expect(
    JSON.stringify(nodes.quest_desktop.props?.visibility) === JSON.stringify({ xs: "collapsed", md: "visible" }),
    "quest desktop breakpoint differs",
  );
  expect(
    JSON.stringify(nodes.quest_compact.props?.visibility) === JSON.stringify({ xs: "visible", md: "collapsed" }),
    "quest compact breakpoint differs",
  );
  expect(
    JSON.stringify(nodes.quest_desktop_columns.children.map((child) => child.node)) ===
      JSON.stringify(["quest_desktop_list", "quest_desktop_detail"]),
    "quest desktop must be list + detail",
  );
  expect(
    JSON.stringify(nodes.quest_compact_stack.children.map((child) => child.node)) ===
      JSON.stringify(["quest_compact_list", "quest_compact_detail"]),
    "quest compact must stack list before detail",
  );
  for (const host of [
    "QuestDesktopListHost",
    "QuestDesktopObjectiveHost",
    "QuestDesktopRewardHost",
    "QuestCompactListHost",
    "QuestCompactObjectiveHost",
    "QuestCompactRewardHost",
  ]) {
    expect(Object.values(nodes).some((node) => node.control_id === host), `quest log misses ${host}`);
  }
  expectRoutes(nodes, "quest_desktop_abandon", ["woc.window.quest_log.abandon"]);
  expectRoutes(nodes, "quest_desktop_share", ["woc.window.quest_log.share"]);
  expectRoutes(nodes, "quest_compact_abandon", ["woc.window.quest_log.abandon"]);
  expectRoutes(nodes, "quest_compact_share", ["woc.window.quest_log.share"]);
  expectRoutes(nodes, "quest_close", ["woc.window.quest_log.close"]);
}

function validateSettings(nodes) {
  const panels = ["keybinds", "controller", "graphics", "interface", "audio", "performance", "bugreport"];
  expect(
    JSON.stringify(nodes.settings_content.props?.supported_panels) === JSON.stringify(panels),
    "settings panel contract differs from the target",
  );
  expect(nodes.settings_content.component === "ScrollBox", "settings content must scroll");
  expect(nodes.settings_content.control_id === "SettingsControlHost", "settings dynamic control host id differs");
  expectRoutes(nodes, "settings_back", ["woc.window.settings.back"]);
  expectRoutes(nodes, "settings_reset", ["woc.window.settings.reset"]);
  expectRoutes(nodes, "settings_close", ["woc.window.settings.close"]);
}

const parsed = parseAssets();
const documents = new Map(
  parsed.map(({ path, document }) => [path.replaceAll("\\", "/").split("/").at(-1), document]),
);
const inventory = validateGraph("inventory_window.zui", documents.get("inventory_window.zui"));
const questLog = validateGraph("quest_log_window.zui", documents.get("quest_log_window.zui"));
const settings = validateGraph("settings_window.zui", documents.get("settings_window.zui"));
validateInventory(inventory.nodes);
validateQuestLog(questLog.nodes);
validateSettings(settings.nodes);

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
    views: assetPaths.length,
    nodes:
      Object.keys(inventory.nodes).length +
      Object.keys(questLog.nodes).length +
      Object.keys(settings.nodes).length,
    controlIds: inventory.controls.size + questLog.controls.size + settings.controls.size,
    events: inventory.events.size + questLog.events.size + settings.events.size,
    inventoryCategories: 6,
    inventorySorts: 3,
    inventoryBagSockets: 4,
    settingsPanels: 7,
    sha256: digest.digest("hex"),
  }),
);
