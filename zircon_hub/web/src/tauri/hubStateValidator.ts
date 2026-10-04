import type { HubShellState } from "../types/hub";

const requiredStrings = [
  "backendEpoch",
  "stateRevision",
  "productName",
  "engineVersion",
  "activePage",
  "pageTitle",
  "pageSubtitle",
  "projectFilter",
  "projectSort",
  "projectViewMode",
  "projectSubpage",
] as const;

const requiredArrays = [
  "projectTemplates",
  "taskStatus",
  "projects",
  "browserProjects",
  "recentProjects",
  "quickActions",
  "sourceEngines",
  "assets",
  "plugins",
  "learnResources",
  "actionHistory",
  "comingSoon",
] as const;

const hubPageIds = new Set([
  "projects",
  "editor",
  "assets",
  "builds",
  "plugins",
  "cloud",
  "team",
  "learn",
  "settings",
]);

// The Rust view model serializes canonical ids for these fields.  Aliases are
// accepted only at the action boundary; accepting them in a published DTO
// would let a future value silently fall through to a different UI branch.
const projectFilterIds = new Set(["all", "existing", "missing"]);
const projectSortIds = new Set(["last-modified", "name"]);
const projectViewModeIds = new Set(["grid", "list"]);
const projectSubpageIds = new Set(["dashboard", "new-project", "project-browser", "project-detail"]);
const statusToneIds = new Set(["running", "success", "warning", "error", "neutral"]);

export function assertHubShellState(value: unknown): HubShellState {
  const state = assertRecord(value, "HubShellState");

  for (const field of requiredStrings) {
    assertString(state[field], field);
  }

  if (!/^(0|[1-9]\d*)$/.test(state.stateRevision as string)) {
    throw new Error("Hub state field 'stateRevision' must be a canonical decimal string");
  }
  if (!(state.backendEpoch as string).trim()) {
    throw new Error("Hub state field 'backendEpoch' must not be empty");
  }

  assertEnum(state.activePage, hubPageIds, "activePage");
  assertEnum(state.projectFilter, projectFilterIds, "projectFilter");
  assertEnum(state.projectSort, projectSortIds, "projectSort");
  assertEnum(state.projectViewMode, projectViewModeIds, "projectViewMode");
  assertEnum(state.projectSubpage, projectSubpageIds, "projectSubpage");

  for (const field of requiredArrays) {
    assertArray(state[field], field);
  }

  assertUniqueIds(state.projectTemplates, "projectTemplates");
  assertUniqueIds(state.projects, "projects");
  assertUniqueIds(state.browserProjects, "browserProjects");
  assertUniqueIds(state.recentProjects, "recentProjects");
  assertUniqueIds(state.quickActions, "quickActions");
  assertUniqueIds(state.assets, "assets");
  assertUniqueIds(state.learnResources, "learnResources");
  assertUniqueIds(state.comingSoon, "comingSoon");

  assertNullableString(state.selectedProjectId, "selectedProjectId");
  assertNullableString(state.activeSourceEngineId, "activeSourceEngineId");
  assertTaskSummary(state.taskSummary);
  if (state.windowCloseSaveError !== undefined && state.windowCloseSaveError !== null) {
    const saveError = assertRecord(state.windowCloseSaveError, "windowCloseSaveError");
    assertString(saveError.label, "windowCloseSaveError.label");
    assertString(saveError.detail, "windowCloseSaveError.detail");
    assertNullableString(saveError.recovery, "windowCloseSaveError.recovery");
  }
  assertStatusPills(state.taskStatus);
  assertSourceEngines(state.sourceEngines);
  assertPlugins(state.plugins);
  assertActionHistory(state.actionHistory);
  assertTeam(state.team);
  assertSettings(state.settings);
  assertNullableRecord(state.selectedProject, "selectedProject");
  assertNullableRecord(state.settingsDraft, "settingsDraft");
  assertRecord(state.ui, "ui");
  const shell = assertRecord((state.ui as Record<string, unknown>).shell, "ui.shell");
  assertString(shell.demoModeBadge, "ui.shell.demoModeBadge");
  assertNavigationItems(shell.navItems);
  assertRecord((state.ui as Record<string, unknown>).common, "ui.common");

  return value as HubShellState;
}

function assertRecord(value: unknown, field: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`Hub state field '${field}' must be an object`);
  }

  return value as Record<string, unknown>;
}

function assertString(value: unknown, field: string) {
  if (typeof value !== "string") {
    throw new Error(`Hub state field '${field}' must be a string`);
  }
}

function assertEnum(value: unknown, supported: ReadonlySet<string>, field: string) {
  if (typeof value !== "string" || !supported.has(value)) {
    throw new Error(`Hub state field '${field}' must use a supported canonical id`);
  }
}

function assertTone(value: unknown, field: string) {
  if (typeof value !== "string" || !statusToneIds.has(value)) {
    throw new Error(`Hub state field '${field}' must use a supported status tone`);
  }
}

function assertNullableString(value: unknown, field: string) {
  if (value !== null && typeof value !== "string") {
    throw new Error(`Hub state field '${field}' must be a string or null`);
  }
}

function assertNullableRecord(value: unknown, field: string) {
  if (value !== null) {
    assertRecord(value, field);
  }
}

function assertArray(value: unknown, field: string): unknown[] {
  if (!Array.isArray(value)) {
    throw new Error(`Hub state field '${field}' must be an array`);
  }
  return value;
}

function assertUniqueIds(value: unknown, field: string): Array<Record<string, unknown>> {
  const rows = assertArray(value, field);
  const ids = new Set<string>();
  return rows.map((item, index) => {
    const row = assertRecord(item, `${field}[${index}]`);
    assertString(row.id, `${field}[${index}].id`);
    const id = row.id as string;
    if (!id.trim()) {
      throw new Error(`Hub state field '${field}[${index}].id' must not be empty`);
    }
    if (ids.has(id)) {
      throw new Error(`Hub state field '${field}' ids must be unique`);
    }
    ids.add(id);
    return row;
  });
}

function assertTaskSummary(value: unknown) {
  const task = assertRecord(value, "taskSummary");
  for (const field of ["label", "detail", "operation"] as const) {
    assertString(task[field], `taskSummary.${field}`);
  }
  assertTone(task.tone, "taskSummary.tone");
  assertBoolean(task.running, "taskSummary.running");
  assertBoolean(task.cancellable, "taskSummary.cancellable");
  assertNullableString(task.recovery, "taskSummary.recovery");
  assertIntegerInRange(task.progressPercent, "taskSummary.progressPercent", 0, 100);
  assertNonNegativeInteger(task.taskId, "taskSummary.taskId");
  assertNonNegativeInteger(task.queued, "taskSummary.queued");
}

function assertStatusPills(value: unknown) {
  const rows = assertUniqueIds(value, "taskStatus");
  rows.forEach((row, index) => {
    assertString(row.label, `taskStatus[${index}].label`);
    assertTone(row.tone, `taskStatus[${index}].tone`);
  });
}

function assertSourceEngines(value: unknown) {
  const engines = assertUniqueIds(value, "sourceEngines");
  engines.forEach((engine, index) => {
    const historyField = `sourceEngines[${index}].buildHistory`;
    const history = assertUniqueIds(engine.buildHistory, historyField);
    history.forEach((build, buildIndex) => {
      assertTone(build.statusTone, `${historyField}[${buildIndex}].statusTone`);
      if (build.jobs !== null && build.jobs !== undefined) {
        assertNonNegativeInteger(build.jobs, `${historyField}[${buildIndex}].jobs`);
      }
    });
  });
}

function assertPlugins(value: unknown) {
  const plugins = assertUniqueIds(value, "plugins");
  plugins.forEach((plugin, index) => {
    assertTone(plugin.maturityTone, `plugins[${index}].maturityTone`);
    assertBoolean(plugin.editorScoped, `plugins[${index}].editorScoped`);
    assertNonNegativeInteger(plugin.moduleCount, `plugins[${index}].moduleCount`);
    const packaging = assertArray(plugin.defaultPackaging, `plugins[${index}].defaultPackaging`);
    packaging.forEach((entry, entryIndex) => assertString(entry, `plugins[${index}].defaultPackaging[${entryIndex}]`));
  });
}

function assertActionHistory(value: unknown) {
  const actions = assertUniqueIds(value, "actionHistory");
  actions.forEach((action, index) => {
    assertTone(action.tone, `actionHistory[${index}].tone`);
    assertNullableString(action.recovery, `actionHistory[${index}].recovery`);
    if (action.processId !== null && action.processId !== undefined) {
      assertNonNegativeInteger(action.processId, `actionHistory[${index}].processId`);
    }
    const detailRowsField = `actionHistory[${index}].detailRows`;
    const detailRows = assertUniqueIds(action.detailRows, detailRowsField);
    detailRows.forEach((row, rowIndex) => {
      assertString(row.title, `${detailRowsField}[${rowIndex}].title`);
      assertString(row.detail, `${detailRowsField}[${rowIndex}].detail`);
    });
  });
}

function assertTeam(value: unknown) {
  const team = assertRecord(value, "team");
  if (team.members !== undefined) {
    const members = assertUniqueIds(team.members, "team.members");
    members.forEach((member, index) => {
      assertNonNegativeInteger(member.commits, `team.members[${index}].commits`);
    });
  }
}

function assertSettings(value: unknown) {
  const settings = assertRecord(value, "settings");
  if (settings.health === undefined) {
    return;
  }
  const health = assertRecord(settings.health, "settings.health");
  assertTone(health.tone, "settings.health.tone");
  assertIntegerInRange(health.completion, "settings.health.completion", 0, 100);
  const rows = assertUniqueIds(health.rows, "settings.health.rows");
  rows.forEach((row, index) => {
    assertBoolean(row.selected, `settings.health.rows[${index}].selected`);
  });
}

function assertBoolean(value: unknown, field: string) {
  if (typeof value !== "boolean") {
    throw new Error(`Hub state field '${field}' must be a boolean`);
  }
}

function assertIntegerInRange(value: unknown, field: string, minimum: number, maximum: number) {
  if (!Number.isSafeInteger(value) || (value as number) < minimum || (value as number) > maximum) {
    throw new Error(`Hub state field '${field}' must be an integer between ${minimum} and ${maximum}`);
  }
}

function assertNonNegativeInteger(value: unknown, field: string) {
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw new Error(`Hub state field '${field}' must be a non-negative integer`);
  }
}

function assertNavigationItems(value: unknown) {
  const navigationItems = assertUniqueIds(value, "ui.shell.navItems");
  for (const [index, navItem] of navigationItems.entries()) {
    assertString(navItem.id, `ui.shell.navItems[${index}].id`);
    assertString(navItem.label, `ui.shell.navItems[${index}].label`);
    if (!hubPageIds.has(navItem.id as string)) {
      throw new Error(`Hub state field 'ui.shell.navItems[${index}].id' is not a supported Hub page`);
    }
  }
}
