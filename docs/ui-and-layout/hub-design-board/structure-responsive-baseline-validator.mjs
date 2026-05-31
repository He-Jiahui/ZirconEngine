import { existsSync, readFileSync } from "node:fs";
import {
  STRUCTURE_RESPONSIVE_BASELINE,
  STRUCTURE_RESPONSIVE_BASELINE_EXPECTED_BREAKPOINTS,
  STRUCTURE_RESPONSIVE_BASELINE_REQUIRED_FIELDS,
} from "./board-registry.mjs";

const REQUIRED_INVARIANTS = [
  "Topbar owns global source, task status, account, and window controls.",
  "Sidebar owns primary navigation and the Engine Status card.",
  "Workspace owns page title, toolbar, cards, tables, and page content.",
  "Bottom strip owns button-state visual samples.",
  "Overlays float above the shell and do not consume layout space.",
];

export function validateStructureResponsiveBaselineFile(filePath) {
  const baseline = readBaseline(filePath);
  for (const field of STRUCTURE_RESPONSIVE_BASELINE_REQUIRED_FIELDS) {
    if (!(field in baseline)) {
      throw new Error(`Structure responsive baseline missing required field: ${field}`);
    }
  }
  if (STRUCTURE_RESPONSIVE_BASELINE !== "docs/ui-and-layout/hub-design-board/structure-responsive-baseline.json") {
    throw new Error(`Unexpected structure responsive baseline path: ${STRUCTURE_RESPONSIVE_BASELINE}`);
  }
  if (baseline.source !== "docs/ui-and-layout/hub-design-board/index.html?board=flow") {
    throw new Error(`Structure responsive baseline source is ${baseline.source}`);
  }
  if (baseline.artifact !== "docs/ui-and-layout/hub-design-structure-supplement.png") {
    throw new Error(`Structure responsive baseline artifact is ${baseline.artifact}`);
  }
  if (baseline.review_focus !== "route-and-responsive-structure") {
    throw new Error(`Structure responsive baseline review focus is ${baseline.review_focus}`);
  }
  validateBreakpoints(baseline.breakpoints);
  validateInvariants(baseline.invariants);
}

function readBaseline(filePath) {
  if (!existsSync(filePath)) {
    throw new Error(`Missing structure responsive baseline: ${filePath}`);
  }
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function validateBreakpoints(actual) {
  if (!Array.isArray(actual) || actual.length !== STRUCTURE_RESPONSIVE_BASELINE_EXPECTED_BREAKPOINTS.length) {
    throw new Error(
      `Structure responsive baseline breakpoint count is ${actual?.length}; expected ${STRUCTURE_RESPONSIVE_BASELINE_EXPECTED_BREAKPOINTS.length}`,
    );
  }
  for (let index = 0; index < STRUCTURE_RESPONSIVE_BASELINE_EXPECTED_BREAKPOINTS.length; index += 1) {
    const expected = STRUCTURE_RESPONSIVE_BASELINE_EXPECTED_BREAKPOINTS[index];
    const row = actual[index];
    for (const key of ["id", "label", "sidebar_mode", "workspace_rule"]) {
      if (row?.[key] !== expected[key]) {
        throw new Error(`Structure responsive baseline row ${index + 1} ${key} is ${row?.[key]}; expected ${expected[key]}`);
      }
    }
    for (const owner of ["topbar-global-commands", "sidebar-navigation-owner", "workspace-page-owner", "bottom-state-strip-owner"]) {
      if (!row.must_preserve?.includes(owner)) {
        throw new Error(`Structure responsive baseline row ${index + 1} must preserve ${owner}`);
      }
    }
  }
}

function validateInvariants(actual) {
  if (!Array.isArray(actual) || actual.length !== REQUIRED_INVARIANTS.length) {
    throw new Error(`Structure responsive baseline invariant count is ${actual?.length}; expected ${REQUIRED_INVARIANTS.length}`);
  }
  for (let index = 0; index < REQUIRED_INVARIANTS.length; index += 1) {
    if (actual[index] !== REQUIRED_INVARIANTS[index]) {
      throw new Error(`Structure responsive baseline invariant ${index + 1} is ${actual[index]}; expected ${REQUIRED_INVARIANTS[index]}`);
    }
  }
}
