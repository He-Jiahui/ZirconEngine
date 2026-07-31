import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const assetDir = join(repoRoot, "examples", "woc", "assets", "ui", "shell");
const assetPaths = [
  join(assetDir, "shell_theme.zui"),
  join(assetDir, "offline_picker.zui"),
  join(assetDir, "welcome_screen.zui"),
  join(assetDir, "character_select.zui"),
  join(assetDir, "character_create.zui"),
  join(assetDir, "realm_select.zui"),
  join(assetDir, "mode_select.zui"),
  join(assetDir, "auth_form.zui"),
  join(assetDir, "password_recovery.zui"),
];
const allowedComponents = new Set([
  "Button",
  "GridGroup",
  "HorizontalBox",
  "Label",
  "Overlay",
  "ScrollBox",
  "Space",
  "Stack",
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
  throw new Error(`M8 retained UI asset check failed: ${message}`);
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

function eventRoutes(node) {
  return (node.events ?? []).map((event) => event.route);
}

function expectRoutes(node, expected, label) {
  const actual = eventRoutes(node);
  expect(JSON.stringify(actual) === JSON.stringify(expected), `${label} routes ${JSON.stringify(actual)}`);
}

function validateView(document, expectedId) {
  expect(document.asset.kind === "view", `${expectedId} must be a view`);
  expect(document.asset.id === expectedId, `${expectedId} asset id mismatch`);
  expect(document.asset.version === 2, `${expectedId} must use schema version 2`);
  expect(
    JSON.stringify(document.imports?.styles) === JSON.stringify(["res://ui/shell/shell_theme.zui"]),
    `${expectedId} must import only the WOC shell theme`,
  );
  expect(document.imports?.resources?.length === 1, `${expectedId} must import one fallback backdrop`);
  expect(document.imports.resources[0].kind === "image", `${expectedId} backdrop must be an image`);
  expect(
    document.imports.resources[0].uri === "res://m8/env/vale_backdrop.webp",
    `${expectedId} backdrop resource mismatch`,
  );
  expect(document.imports.resources[0].fallback?.mode === "optional", `${expectedId} backdrop must remain optional`);

  const nodes = document.nodes ?? {};
  const root = document.root?.node;
  expect(typeof root === "string" && root in nodes, `${expectedId} root is missing`);

  const reachable = new Set();
  const visiting = new Set();
  function visit(nodeId) {
    expect(nodeId in nodes, `${expectedId} references missing node ${nodeId}`);
    if (visiting.has(nodeId)) fail(`${expectedId} contains a node cycle at ${nodeId}`);
    if (reachable.has(nodeId)) return;
    visiting.add(nodeId);
    reachable.add(nodeId);
    for (const child of nodes[nodeId].children ?? []) visit(child.node);
    visiting.delete(nodeId);
  }
  visit(root);
  const unreachable = Object.keys(nodes).filter((nodeId) => !reachable.has(nodeId));
  expect(unreachable.length === 0, `${expectedId} has unreachable nodes: ${unreachable.join(", ")}`);

  const controlIds = new Set();
  const eventIds = new Set();
  for (const [nodeId, node] of Object.entries(nodes)) {
    expect(typeof node.component === "string" && node.component.length > 0, `${expectedId}.${nodeId} has no component`);
    expect(allowedComponents.has(node.component), `${expectedId}.${nodeId} uses unaudited component ${node.component}`);
    if (node.control_id) {
      expect(!controlIds.has(node.control_id), `${expectedId} duplicates control id ${node.control_id}`);
      controlIds.add(node.control_id);
    }
    for (const event of node.events ?? []) {
      expect(typeof event.id === "string" && event.id.length > 0, `${expectedId}.${nodeId} has an event without id`);
      expect(!eventIds.has(event.id), `${expectedId} duplicates event id ${event.id}`);
      eventIds.add(event.id);
      expect(typeof event.event === "string" && event.event.length > 0, `${event.id} has no event kind`);
      expect(typeof event.route === "string" && event.route.startsWith("woc.shell."), `${event.id} has invalid route`);
    }
  }
  return { nodes, controlIds, eventIds };
}

function validateTheme(document) {
  const expectedId = "res://ui/shell/shell_theme.zui";
  expect(document.asset.kind === "style", "shell theme must be a style asset");
  expect(document.asset.id === expectedId, "shell theme asset id mismatch");
  expect(document.asset.version === 2, "shell theme must use schema version 2");

  const requiredTokens = [
    "woc_shell_bg",
    "woc_panel_bg",
    "woc_border",
    "woc_gold",
    "woc_text",
    "woc_text_muted",
    "woc_discord",
    "woc_success",
    "woc_error",
  ];
  for (const token of requiredTokens) expect(token in document.tokens, `shell theme misses token ${token}`);

  const selectors = new Set();
  for (const stylesheet of document.stylesheets ?? []) {
    for (const rule of stylesheet.rules ?? []) {
      expect(!selectors.has(rule.selector), `shell theme duplicates selector ${rule.selector}`);
      selectors.add(rule.selector);
      const stack = [rule.set];
      while (stack.length > 0) {
        const value = stack.pop();
        if (typeof value === "string" && value.startsWith("$")) {
          expect(value.slice(1) in document.tokens, `${rule.selector} references missing token ${value}`);
        } else if (Array.isArray(value)) {
          stack.push(...value);
        } else if (value && typeof value === "object") {
          stack.push(...Object.values(value));
        }
      }
    }
  }
  for (const selector of [
    ".woc-shell",
    ".woc-panel",
    ".woc-class-button:selected",
    ".woc-primary-action",
    ".woc-primary-action:disabled",
    ".woc-discord-strip",
    ".woc-roster-host",
    ".woc-modal-card",
    ".woc-danger-action",
    ".woc-danger-action:disabled",
    ".woc-mode-trigger",
    ".woc-mode-option:selected",
    ".woc-play-action",
    ".woc-contract-address",
    ".woc-auth-form",
    ".woc-auth-link",
    ".woc-auth-provider-host",
    ".woc-auth-status",
  ]) {
    expect(selectors.has(selector), `shell theme misses selector ${selector}`);
  }
  return selectors.size;
}

function validateOffline(document) {
  const { nodes, controlIds, eventIds } = validateView(document, "res://ui/shell/offline_picker.zui");
  const classes = ["warrior", "paladin", "hunter", "rogue", "priest", "shaman", "mage", "warlock", "druid"];
  const expectedClassNodes = classes.map((playerClass) => `class_${playerClass}`);
  const actualClassNodes = nodes.class_rail.children.map((child) => child.node);
  expect(JSON.stringify(actualClassNodes) === JSON.stringify(expectedClassNodes), "offline class order differs from the target");
  const expectedCompactClassNodes = classes.map((playerClass) => `compact_class_${playerClass}`);
  const actualCompactClassNodes = nodes.compact_class_grid.children.map((child) => child.node);
  expect(
    JSON.stringify(actualCompactClassNodes) === JSON.stringify(expectedCompactClassNodes),
    "compact offline class order differs from the target",
  );
  classes.forEach((playerClass, index) => {
    const node = nodes[`class_${playerClass}`];
    expect(node.props.checked === (index === 0), `${playerClass} default selection is wrong`);
    expect(node.events[0].event === "Toggle", `${playerClass} class button must emit Toggle`);
    expectRoutes(node, [`woc.shell.offline.select_class.${playerClass}`], `${playerClass} class button`);
    const compactNode = nodes[`compact_class_${playerClass}`];
    expect(compactNode.props.checked === (index === 0), `compact ${playerClass} default selection is wrong`);
    expect(compactNode.events[0].event === "Toggle", `compact ${playerClass} class button must emit Toggle`);
    expectRoutes(
      compactNode,
      [`woc.shell.offline.select_class.${playerClass}`],
      `compact ${playerClass} class button`,
    );
  });

  expect(
    JSON.stringify(nodes.skin_column.children.map((child) => child.node)) ===
      JSON.stringify(["skin_title", "skin_0", "skin_1", "skin_2", "skin_3"]),
    "offline skin picker must expose four ordered variants",
  );
  for (let index = 0; index < 4; index += 1) {
    expect(nodes[`skin_${index}`].props.checked === (index === 0), `skin ${index} default selection is wrong`);
    expect(nodes[`skin_${index}`].events[0].event === "Toggle", `skin ${index} must emit Toggle`);
    expectRoutes(nodes[`skin_${index}`], [`woc.shell.offline.select_skin.${index}`], `skin ${index}`);
    expect(
      nodes[`compact_skin_${index}`].props.checked === (index === 0),
      `compact skin ${index} default selection is wrong`,
    );
    expect(nodes[`compact_skin_${index}`].events[0].event === "Toggle", `compact skin ${index} must emit Toggle`);
    expectRoutes(
      nodes[`compact_skin_${index}`],
      [`woc.shell.offline.select_skin.${index}`],
      `compact skin ${index}`,
    );
  }

  expectRoutes(nodes.back, ["woc.shell.offline.back"], "offline Back");
  expectRoutes(nodes.enter_world, ["woc.shell.offline.enter_world"], "offline Enter World");
  expectRoutes(nodes.compact_back, ["woc.shell.offline.back"], "compact offline Back");
  expectRoutes(nodes.compact_enter_world, ["woc.shell.offline.enter_world"], "compact offline Enter World");
  expectRoutes(
    nodes.name_field,
    ["woc.shell.offline.set_name", "woc.shell.offline.enter_world"],
    "offline character name",
  );
  expectRoutes(
    nodes.compact_name_field,
    ["woc.shell.offline.set_name", "woc.shell.offline.enter_world"],
    "compact offline character name",
  );
  for (const controlId of [
    "OfflineCharacterPreviewHost",
    "OfflineCharacterName",
    "OfflineCharacterError",
    "OfflineSkinPicker",
    "OfflineClassDetails",
    "OfflineClassPicker",
    "OfflineBack",
    "OfflineEnterWorld",
    "OfflineDesktopLayer",
    "OfflineCompactLayer",
    "OfflineCompactCharacterPreviewHost",
    "OfflineCompactCharacterName",
    "OfflineCompactCharacterError",
    "OfflineCompactSkinPicker",
    "OfflineCompactClassDetails",
    "OfflineCompactClassPicker",
    "OfflineCompactBack",
    "OfflineCompactEnterWorld",
  ]) {
    expect(controlIds.has(controlId), `offline picker misses control ${controlId}`);
  }
  expect(nodes.offline_root.component === "Overlay", "offline picker must remain a full-screen overlay");
  expect(
    nodes.offline_root.props.background_image === "res://m8/env/vale_backdrop.webp",
    "offline picker backdrop mismatch",
  );
  expect(
    JSON.stringify(nodes.offline_root.children.map((child) => child.node)) ===
      JSON.stringify(["desktop_layer", "compact_layer"]),
    "offline root must mount desktop and compact responsive layers",
  );
  expect(
    JSON.stringify(nodes.desktop_layer.props.visibility) === JSON.stringify({ xs: "collapsed", md: "visible" }),
    "offline desktop layer breakpoint gate is wrong",
  );
  expect(
    JSON.stringify(nodes.compact_layer.props.visibility) === JSON.stringify({ xs: "visible", md: "collapsed" }),
    "offline compact layer breakpoint gate is wrong",
  );
  expect(nodes.compact_class_grid.layout.container.columns === 3, "compact offline picker must use a three-column class grid");
  expect(nodes.class_rail.layout.anchor.y === 1, "offline class rail must remain bottom-anchored");
  expect(nodes.details_panel.layout.anchor.x === 1, "offline class details must remain right-anchored");
  return { nodeCount: Object.keys(nodes).length, controlCount: controlIds.size, eventCount: eventIds.size };
}

function validateWelcome(document) {
  const { nodes, controlIds, eventIds } = validateView(document, "res://ui/shell/welcome_screen.zui");
  expect(
    JSON.stringify(nodes.body.children.map((child) => child.node)) === JSON.stringify(["main_column", "rail"]),
    "Welcome body must keep the news/rail split",
  );
  expect(nodes.body.component === "Stack", "Welcome body must use the responsive Stack component");
  expect(
    JSON.stringify(nodes.body.props.direction) === JSON.stringify({ xs: "column", md: "row" }),
    "Welcome body breakpoint direction is wrong",
  );
  expect(nodes.content.layout.width.min === undefined, "Welcome content must not impose a 720px minimum width");
  expect(
    nodes.welcome_root.props.background_image === "res://m8/env/vale_backdrop.webp",
    "Welcome backdrop mismatch",
  );
  expect(
    JSON.stringify(nodes.main_column.children.map((child) => child.node)) === JSON.stringify(["news_panel", "discord_strip"]),
    "Welcome main column must keep news before Discord",
  );
  expect(
    JSON.stringify(nodes.rail.children.map((child) => child.node)) === JSON.stringify(["armory_card", "chest_tile", "event_slot"]),
    "Welcome rail order differs from the target",
  );
  for (const nodeId of ["discord_strip", "rail", "armory_card", "chest_tile", "event_slot"]) {
    expect(nodes[nodeId].props.visibility === "collapsed", `${nodeId} must start collapsed for view-model gating`);
  }
  expect(nodes.continue.props.disabled === true, "online Welcome Continue must start disabled");
  expectRoutes(nodes.discord_join, ["woc.shell.welcome.join_discord"], "Welcome Discord");
  expectRoutes(nodes.armory_card, ["woc.shell.welcome.open_armory"], "Welcome Armory");
  expectRoutes(nodes.continue, ["woc.shell.welcome.continue"], "Welcome Continue");
  for (const controlId of [
    "WelcomeCharacterStageHost",
    "WelcomeHeader",
    "WelcomeNewsFeed",
    "WelcomeDiscordStrip",
    "WelcomeRail",
    "WelcomeArmoryCard",
    "WelcomeChestTile",
    "WelcomeConnectionStatus",
    "WelcomeContinue",
    "WelcomeContinueHint",
    "WelcomeVersion",
  ]) {
    expect(controlIds.has(controlId), `Welcome screen misses control ${controlId}`);
  }
  return { nodeCount: Object.keys(nodes).length, controlCount: controlIds.size, eventCount: eventIds.size };
}

function expectMinimumTouchHeight(nodes, nodeIds, label) {
  for (const nodeId of nodeIds) {
    expect(nodes[nodeId].layout?.height?.min >= 40, `${label}.${nodeId} must expose a 40px touch target`);
  }
}

function validateCharacterSelect(document) {
  const { nodes, controlIds, eventIds } = validateView(
    document,
    "res://ui/shell/character_select.zui",
  );
  expect(nodes.character_select_root.component === "Overlay", "character select must be a full-screen overlay");
  expect(
    nodes.character_select_root.props.background_image === "res://m8/env/vale_backdrop.webp",
    "character select backdrop mismatch",
  );
  expect(nodes.body.component === "Stack", "character select body must be responsive");
  expect(
    JSON.stringify(nodes.body.props.direction) === JSON.stringify({ xs: "column", md: "row" }),
    "character select body breakpoint direction is wrong",
  );
  expect(nodes.roster_host.component === "ScrollBox", "character roster host must own scrolling");
  expect(
    nodes.roster_host.props.dynamic_route_prefix === "woc.shell.characters.row",
    "character roster dynamic route prefix mismatch",
  );
  expect(nodes.primary.props.disabled === true, "character primary action must start disabled");
  expectRoutes(nodes.realm_button, ["woc.shell.characters.change_realm"], "character realm");
  expectRoutes(nodes.sort_button, ["woc.shell.characters.toggle_sort"], "character sort toggle");
  expectRoutes(nodes.back, ["woc.shell.characters.back"], "character Back");
  expectRoutes(nodes.new_character, ["woc.shell.characters.new"], "character New");
  expectRoutes(nodes.primary, ["woc.shell.characters.primary"], "character primary");

  const modes = ["level", "name", "recent", "playtime"];
  expect(
    JSON.stringify(nodes.sort_menu.children.map((child) => child.node)) ===
      JSON.stringify(modes.map((mode) => `sort_${mode}`)),
    "character sort menu order differs from the target",
  );
  expect(nodes.sort_menu.props.visibility === "collapsed", "character sort menu must start collapsed");
  for (const mode of modes) {
    expectRoutes(nodes[`sort_${mode}`], [`woc.shell.characters.sort.${mode}`], `character sort ${mode}`);
  }

  for (const nodeId of ["wallet_host", "github_host", "steam_host", "integration_hosts"]) {
    expect(nodes[nodeId].props.visibility === "collapsed", `${nodeId} must remain capability-gated`);
  }
  for (const nodeId of ["takeover_modal", "delete_modal"]) {
    expect(nodes[nodeId].props.visibility === "collapsed", `${nodeId} must start collapsed`);
  }
  expectRoutes(
    nodes.takeover_cancel,
    ["woc.shell.characters.takeover.cancel"],
    "takeover Cancel",
  );
  expectRoutes(
    nodes.takeover_confirm,
    ["woc.shell.characters.takeover.confirm"],
    "takeover Confirm",
  );
  expectRoutes(
    nodes.delete_confirmation,
    ["woc.shell.characters.delete.set_confirmation", "woc.shell.characters.delete.submit"],
    "delete confirmation field",
  );
  expectRoutes(nodes.delete_cancel, ["woc.shell.characters.delete.cancel"], "delete Cancel");
  expectRoutes(nodes.delete_submit, ["woc.shell.characters.delete.submit"], "delete Submit");
  expect(nodes.delete_submit.props.disabled === true, "delete Submit must start disabled");

  for (const controlId of [
    "OnlineCharacterAccountName",
    "OnlineCharacterRealm",
    "OnlineCharacterSort",
    "OnlineCharacterSortMenu",
    "OnlineCharacterRosterHost",
    "OnlineCharacterPreviewHost",
    "OnlineCharacterClassDetailsHost",
    "OnlineCharacterPrimary",
    "OnlineCharacterTakeOverModal",
    "OnlineCharacterDeleteModal",
    "OnlineCharacterDeleteConfirmation",
    "OnlineCharacterDeleteSubmit",
  ]) {
    expect(controlIds.has(controlId), `character select misses control ${controlId}`);
  }
  expectMinimumTouchHeight(
    nodes,
    [
      "realm_button",
      "sort_button",
      "sort_level",
      "sort_name",
      "sort_recent",
      "sort_playtime",
      "back",
      "new_character",
      "primary",
      "takeover_cancel",
      "takeover_confirm",
      "delete_confirmation",
      "delete_cancel",
      "delete_submit",
    ],
    "character select",
  );
  return { nodeCount: Object.keys(nodes).length, controlCount: controlIds.size, eventCount: eventIds.size };
}

function validateCharacterCreate(document) {
  const { nodes, controlIds, eventIds } = validateView(
    document,
    "res://ui/shell/character_create.zui",
  );
  const classes = ["warrior", "paladin", "hunter", "rogue", "priest", "shaman", "mage", "warlock", "druid"];
  expect(
    JSON.stringify(nodes.class_grid.children.map((child) => child.node)) ===
      JSON.stringify(classes.map((playerClass) => `class_${playerClass}`)),
    "online create class order differs from the target",
  );
  for (const [index, playerClass] of classes.entries()) {
    const node = nodes[`class_${playerClass}`];
    expect(node.props.checked === (index === 0), `${playerClass} online create default is wrong`);
    expect(node.events[0].event === "Toggle", `${playerClass} online create class must emit Toggle`);
    expectRoutes(
      node,
      [`woc.shell.characters.create.select_class.${playerClass}`],
      `${playerClass} online create class`,
    );
  }
  expect(
    JSON.stringify(nodes.skin_row.children.map((child) => child.node)) ===
      JSON.stringify(["skin_0", "skin_1", "skin_2", "skin_3"]),
    "online create must expose four class-catalog skin sockets",
  );
  expect(
    nodes.skin_row.props.dynamic_visibility_owner === "class_catalog",
    "online create skin visibility must remain class-catalog owned",
  );
  for (let index = 0; index < 4; index += 1) {
    expect(nodes[`skin_${index}`].props.checked === (index === 0), `online create skin ${index} default is wrong`);
    expectRoutes(
      nodes[`skin_${index}`],
      [`woc.shell.characters.create.select_skin.${index}`],
      `online create skin ${index}`,
    );
  }
  expect(nodes.name_field.props.max_length === 16, "online create name must retain the 16-byte UI cap");
  expectRoutes(
    nodes.name_field,
    ["woc.shell.characters.create.set_name", "woc.shell.characters.create.submit"],
    "online create name",
  );
  expectRoutes(nodes.back, ["woc.shell.characters.create.back"], "online create Back");
  expectRoutes(nodes.submit, ["woc.shell.characters.create.submit"], "online create Submit");
  expect(nodes.body.component === "Stack", "online create body must be responsive");
  expect(
    JSON.stringify(nodes.body.props.direction) === JSON.stringify({ xs: "column", md: "row" }),
    "online create body breakpoint direction is wrong",
  );
  expect(nodes.form_panel.component === "ScrollBox", "online create form must remain scrollable");
  for (const controlId of [
    "OnlineCharacterCreateName",
    "OnlineCharacterCreateError",
    "OnlineCharacterCreateClassPicker",
    "OnlineCharacterCreateSkinPicker",
    "OnlineCharacterCreateBack",
    "OnlineCharacterCreateSubmit",
    "OnlineCharacterCreatePreviewHost",
    "OnlineCharacterCreateClassDetailsHost",
  ]) {
    expect(controlIds.has(controlId), `online create misses control ${controlId}`);
  }
  expectMinimumTouchHeight(
    nodes,
    [
      "name_field",
      ...classes.map((playerClass) => `class_${playerClass}`),
      "skin_0",
      "skin_1",
      "skin_2",
      "skin_3",
      "back",
      "submit",
    ],
    "online create",
  );
  return { nodeCount: Object.keys(nodes).length, controlCount: controlIds.size, eventCount: eventIds.size };
}

function validateRealmSelect(document) {
  const { nodes, controlIds, eventIds } = validateView(
    document,
    "res://ui/shell/realm_select.zui",
  );
  expect(nodes.realm_select_root.component === "Overlay", "realm directory must be a full-screen overlay");
  expect(
    nodes.realm_select_root.props.background_image === "res://m8/env/vale_backdrop.webp",
    "realm directory backdrop mismatch",
  );
  expect(nodes.realm_list_host.component === "ScrollBox", "realm directory host must own scrolling");
  expect(
    nodes.realm_list_host.props.dynamic_route_prefix === "woc.shell.realms.row",
    "realm directory dynamic route prefix mismatch",
  );
  expect(
    JSON.stringify(nodes.realm_list_host.props.row_fields) ===
      JSON.stringify(["name", "type", "character_count", "status", "population", "recommended"]),
    "realm directory row projection mismatch",
  );
  expect(nodes.loading.props.visibility === undefined, "realm directory Loading must start visible");
  expect(nodes.empty.props.visibility === "collapsed", "realm directory Empty must start collapsed");
  expect(nodes.error.props.visibility === "collapsed", "realm directory Error must start collapsed");
  expectRoutes(nodes.back, ["woc.shell.realms.back"], "realm directory Back");
  expectMinimumTouchHeight(nodes, ["back"], "realm directory");
  for (const controlId of [
    "RealmDirectoryAccountName",
    "RealmDirectoryLoading",
    "RealmDirectoryEmpty",
    "RealmDirectoryError",
    "RealmDirectoryListHost",
    "RealmDirectoryBack",
  ]) {
    expect(controlIds.has(controlId), `realm directory misses control ${controlId}`);
  }
  return { nodeCount: Object.keys(nodes).length, controlCount: controlIds.size, eventCount: eventIds.size };
}

function validateModeSelect(document) {
  const { nodes, controlIds, eventIds } = validateView(
    document,
    "res://ui/shell/mode_select.zui",
  );
  expect(nodes.mode_select_root.component === "Overlay", "mode selection must be a full-screen overlay");
  expect(
    nodes.mode_select_root.props.background_image === "res://m8/env/vale_backdrop.webp",
    "mode selection backdrop mismatch",
  );
  expect(nodes.panel.component === "ScrollBox", "mode selection panel must scroll on short viewports");
  expect(nodes.online_option.props.checked === true, "mode selection must default Online");
  expect(nodes.offline_option.props.checked === false, "Offline must not be selected by default");
  expect(nodes.mode_menu.props.visibility === "collapsed", "mode selection menu must start collapsed");
  expect(nodes.offline_summary.props.visibility === "collapsed", "Offline summary must start collapsed");
  expect(
    JSON.stringify(nodes.mode_menu.children.map((child) => child.node)) ===
      JSON.stringify(["online_option", "offline_option"]),
    "mode selection option order differs from the target",
  );
  expectRoutes(nodes.mode_trigger, ["woc.shell.mode.toggle_menu"], "mode trigger");
  expectRoutes(nodes.online_option, ["woc.shell.mode.select.online"], "Online mode");
  expectRoutes(nodes.offline_option, ["woc.shell.mode.select.offline"], "Offline mode");
  expectRoutes(nodes.play, ["woc.shell.mode.play"], "mode Play");
  expectRoutes(nodes.contract_address, ["woc.shell.mode.copy_contract"], "contract copy");
  expect(
    nodes.contract_address.props.text === "3WjLscH2JsXLEFJZRA9z8ti8yRGxWGKbqymPd7UicRth",
    "mode selection contract address drifted",
  );
  expectMinimumTouchHeight(
    nodes,
    ["mode_trigger", "online_option", "offline_option", "play", "contract_address"],
    "mode selection",
  );
  for (const controlId of [
    "ModeSelectionTrigger",
    "ModeSelectionOnlineSummary",
    "ModeSelectionOfflineSummary",
    "ModeSelectionMenu",
    "ModeSelectionOnline",
    "ModeSelectionOffline",
    "ModeSelectionPlay",
    "ModeSelectionContractAddress",
    "ModeSelectionPerformanceTip",
  ]) {
    expect(controlIds.has(controlId), `mode selection misses control ${controlId}`);
  }
  return { nodeCount: Object.keys(nodes).length, controlCount: controlIds.size, eventCount: eventIds.size };
}

function validateAuthForm(document) {
  const { nodes, controlIds, eventIds } = validateView(document, "res://ui/shell/auth_form.zui");
  expect(nodes.auth_root.component === "Overlay", "authentication must be a full-screen overlay");
  expect(
    nodes.auth_root.props.background_image === "res://m8/env/vale_backdrop.webp",
    "authentication backdrop mismatch",
  );
  expect(nodes.panel.component === "ScrollBox", "authentication panel must scroll on short viewports");
  expect(nodes.username.props.max_length === 24, "authentication username must keep the 24-character target cap");
  expect(nodes.password.props.max_length === 128, "authentication password must keep the 128-character target cap");
  expect(nodes.email.props.max_length === 254, "authentication email must keep the 254-character target cap");
  expect(nodes.two_factor.props.max_length === 14, "authentication second factor must keep the 14-character target cap");
  for (const nodeId of ["email_field", "turnstile_host", "two_factor_field", "apple_host", "discord_host"]) {
    expect(nodes[nodeId].props.visibility === "collapsed", `${nodeId} must start hidden until its host condition is met`);
  }
  expectRoutes(nodes.username, ["woc.shell.auth.set_username", "woc.shell.auth.submit"], "authentication username");
  expectRoutes(nodes.password, ["woc.shell.auth.set_password", "woc.shell.auth.submit"], "authentication password");
  expectRoutes(nodes.email, ["woc.shell.auth.set_email", "woc.shell.auth.submit"], "authentication email");
  expectRoutes(nodes.two_factor, ["woc.shell.auth.set_two_factor", "woc.shell.auth.submit"], "authentication second factor");
  expectRoutes(nodes.back, ["woc.shell.auth.back"], "authentication Back");
  expectRoutes(nodes.submit, ["woc.shell.auth.submit"], "authentication Submit");
  expectRoutes(nodes.toggle_mode, ["woc.shell.auth.toggle_mode"], "authentication mode toggle");
  expectRoutes(nodes.open_forgot, ["woc.shell.auth.open_forgot"], "authentication forgot password");
  expectMinimumTouchHeight(
    nodes,
    ["username", "password", "email", "two_factor", "back", "submit", "toggle_mode", "open_forgot"],
    "authentication",
  );
  for (const controlId of [
    "AuthTitle",
    "AuthUsername",
    "AuthPassword",
    "AuthEmailField",
    "AuthEmail",
    "AuthTurnstileHost",
    "AuthTwoFactorField",
    "AuthTwoFactorCode",
    "AuthError",
    "AuthAppleProviderHost",
    "AuthDiscordProviderHost",
    "AuthBack",
    "AuthSubmit",
    "AuthToggleMode",
    "AuthOpenForgotPassword",
    "AuthPanel",
  ]) {
    expect(controlIds.has(controlId), `authentication misses control ${controlId}`);
  }
  return { nodeCount: Object.keys(nodes).length, controlCount: controlIds.size, eventCount: eventIds.size };
}

function validatePasswordRecovery(document) {
  const { nodes, controlIds, eventIds } = validateView(
    document,
    "res://ui/shell/password_recovery.zui",
  );
  expect(nodes.password_recovery_root.component === "Overlay", "password recovery must be a full-screen overlay");
  expect(nodes.forgot_panel.component === "ScrollBox", "forgot-password panel must scroll on short viewports");
  expect(nodes.reset_panel.component === "ScrollBox", "reset-password panel must scroll on short viewports");
  expect(nodes.reset_panel.props.visibility === "collapsed", "reset-password panel must await a host-provided token");
  expect(nodes.forgot_username.props.max_length === 24, "forgot-password username cap drifted");
  expect(nodes.reset_password.props.max_length === 128, "new-password cap drifted");
  expect(nodes.reset_confirmation.props.max_length === 128, "password confirmation cap drifted");
  expectRoutes(
    nodes.forgot_username,
    ["woc.shell.auth.forgot.set_username", "woc.shell.auth.forgot.submit"],
    "forgot-password username",
  );
  expectRoutes(nodes.forgot_back, ["woc.shell.auth.forgot.back"], "forgot-password Back");
  expectRoutes(nodes.forgot_submit, ["woc.shell.auth.forgot.submit"], "forgot-password Submit");
  expectRoutes(
    nodes.reset_password,
    ["woc.shell.auth.reset.set_password", "woc.shell.auth.reset.submit"],
    "reset-password input",
  );
  expectRoutes(
    nodes.reset_confirmation,
    ["woc.shell.auth.reset.set_confirmation", "woc.shell.auth.reset.submit"],
    "reset-password confirmation",
  );
  expectRoutes(nodes.reset_back, ["woc.shell.auth.reset.back"], "reset-password Back");
  expectRoutes(nodes.reset_submit, ["woc.shell.auth.reset.submit"], "reset-password Submit");
  expectMinimumTouchHeight(
    nodes,
    ["forgot_username", "forgot_back", "forgot_submit", "reset_password", "reset_confirmation", "reset_back", "reset_submit"],
    "password recovery",
  );
  for (const controlId of [
    "PasswordRecoveryRequestTitle",
    "PasswordRecoveryUsername",
    "PasswordRecoveryRequestStatus",
    "PasswordRecoveryRequestBack",
    "PasswordRecoveryRequestSubmit",
    "PasswordRecoveryRequestPanel",
    "PasswordRecoveryResetTitle",
    "PasswordRecoveryNewPassword",
    "PasswordRecoveryPasswordConfirmation",
    "PasswordRecoveryResetStatus",
    "PasswordRecoveryResetBack",
    "PasswordRecoveryResetSubmit",
    "PasswordRecoveryResetPanel",
  ]) {
    expect(controlIds.has(controlId), `password recovery misses control ${controlId}`);
  }
  return { nodeCount: Object.keys(nodes).length, controlCount: controlIds.size, eventCount: eventIds.size };
}

const parsed = parseAssets();
const backdrop = readFileSync(join(repoRoot, "examples", "woc", "assets", "m8", "env", "vale_backdrop.webp"));
expect(backdrop.subarray(0, 4).toString("ascii") === "RIFF", "fallback backdrop has no RIFF header");
expect(backdrop.subarray(8, 12).toString("ascii") === "WEBP", "fallback backdrop has no WEBP signature");
const byName = new Map(parsed.map(({ path, document }) => [path.replaceAll("\\", "/").split("/").at(-1), document]));
for (const name of [
  "shell_theme.zui",
  "offline_picker.zui",
  "welcome_screen.zui",
  "character_select.zui",
  "character_create.zui",
  "realm_select.zui",
  "mode_select.zui",
  "auth_form.zui",
  "password_recovery.zui",
]) {
  expect(byName.has(name), `tomllib result misses ${name}`);
}

const selectorCount = validateTheme(byName.get("shell_theme.zui"));
const offline = validateOffline(byName.get("offline_picker.zui"));
const welcome = validateWelcome(byName.get("welcome_screen.zui"));
const characterSelect = validateCharacterSelect(byName.get("character_select.zui"));
const characterCreate = validateCharacterCreate(byName.get("character_create.zui"));
const realmSelect = validateRealmSelect(byName.get("realm_select.zui"));
const modeSelect = validateModeSelect(byName.get("mode_select.zui"));
const authForm = validateAuthForm(byName.get("auth_form.zui"));
const passwordRecovery = validatePasswordRecovery(byName.get("password_recovery.zui"));
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
    views: 8,
    nodes:
      offline.nodeCount +
      welcome.nodeCount +
      characterSelect.nodeCount +
      characterCreate.nodeCount +
      realmSelect.nodeCount +
      modeSelect.nodeCount +
      authForm.nodeCount +
      passwordRecovery.nodeCount,
    controlIds:
      offline.controlCount +
      welcome.controlCount +
      characterSelect.controlCount +
      characterCreate.controlCount +
      realmSelect.controlCount +
      modeSelect.controlCount +
      authForm.controlCount +
      passwordRecovery.controlCount,
    events:
      offline.eventCount +
      welcome.eventCount +
      characterSelect.eventCount +
      characterCreate.eventCount +
      realmSelect.eventCount +
      modeSelect.eventCount +
      authForm.eventCount +
      passwordRecovery.eventCount,
    styleSelectors: selectorCount,
    sha256: digest.digest("hex"),
  }),
);
