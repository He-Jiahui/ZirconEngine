import type { HubShellState } from "../types/hub";

const requiredStrings = [
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

export function assertHubShellState(value: unknown): HubShellState {
  const state = assertRecord(value, "HubShellState");

  for (const field of requiredStrings) {
    assertString(state[field], field);
  }

  for (const field of requiredArrays) {
    assertArray(state[field], field);
  }

  assertNullableString(state.selectedProjectId, "selectedProjectId");
  assertNullableString(state.activeSourceEngineId, "activeSourceEngineId");
  assertRecord(state.taskSummary, "taskSummary");
  assertRecord(state.team, "team");
  assertRecord(state.settings, "settings");
  assertNullableRecord(state.selectedProject, "selectedProject");
  assertNullableRecord(state.settingsDraft, "settingsDraft");
  assertRecord(state.ui, "ui");
  const shell = assertRecord((state.ui as Record<string, unknown>).shell, "ui.shell");
  assertString(shell.demoModeBadge, "ui.shell.demoModeBadge");
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

function assertArray(value: unknown, field: string) {
  if (!Array.isArray(value)) {
    throw new Error(`Hub state field '${field}' must be an array`);
  }
}
