import { existsSync, readFileSync } from "node:fs";
import { DASHBOARD_CAPTURE_NAME, EXPORTS_LIST } from "../hub-web-reference/page-registry.mjs";
import {
  STRUCTURE_REFERENCE_ROUTE_BASELINE,
  STRUCTURE_REFERENCE_ROUTE_BASELINE_EXPECTED_ROUTES,
  STRUCTURE_REFERENCE_ROUTE_BASELINE_INVARIANTS,
  STRUCTURE_REFERENCE_ROUTE_BASELINE_REQUIRED_FIELDS,
} from "./board-registry.mjs";

const KNOWN_FINAL_REFERENCES = new Set([DASHBOARD_CAPTURE_NAME, ...EXPORTS_LIST.map(([, output]) => output)]);

export function validateStructureReferenceRouteBaselineFile(filePath) {
  const baseline = readBaseline(filePath);
  for (const field of STRUCTURE_REFERENCE_ROUTE_BASELINE_REQUIRED_FIELDS) {
    if (!(field in baseline)) {
      throw new Error(`Structure reference route baseline missing required field: ${field}`);
    }
  }
  if (STRUCTURE_REFERENCE_ROUTE_BASELINE !== "docs/ui-and-layout/hub-design-board/structure-reference-route-baseline.json") {
    throw new Error(`Unexpected structure reference route baseline path: ${STRUCTURE_REFERENCE_ROUTE_BASELINE}`);
  }
  if (baseline.source !== "docs/ui-and-layout/hub-design-board/STRUCTURE_TO_REFERENCE_MAP.md") {
    throw new Error(`Structure reference route baseline source is ${baseline.source}`);
  }
  if (baseline.reference_target !== "docs/ui-and-layout/hub.png") {
    throw new Error(`Structure reference route baseline reference target is ${baseline.reference_target}`);
  }
  if (baseline.review_focus !== "structure-to-final-reference-route") {
    throw new Error(`Structure reference route baseline review focus is ${baseline.review_focus}`);
  }
  validateRoutes(baseline.routes);
  validateInvariants(baseline.invariants);
}

function readBaseline(filePath) {
  if (!existsSync(filePath)) {
    throw new Error(`Missing structure reference route baseline: ${filePath}`);
  }
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function validateRoutes(actual) {
  if (!Array.isArray(actual) || actual.length !== STRUCTURE_REFERENCE_ROUTE_BASELINE_EXPECTED_ROUTES.length) {
    throw new Error(
      `Structure reference route baseline route count is ${actual?.length}; expected ${STRUCTURE_REFERENCE_ROUTE_BASELINE_EXPECTED_ROUTES.length}`,
    );
  }
  for (let index = 0; index < STRUCTURE_REFERENCE_ROUTE_BASELINE_EXPECTED_ROUTES.length; index += 1) {
    const expected = STRUCTURE_REFERENCE_ROUTE_BASELINE_EXPECTED_ROUTES[index];
    const row = actual[index];
    for (const key of ["id", "review_item", "design_artifact", "final_reference", "verify_rule"]) {
      if (row?.[key] !== expected[key]) {
        throw new Error(
          `Structure reference route baseline row ${index + 1} ${key} is ${row?.[key]}; expected ${expected[key]}`,
        );
      }
    }
    if (!KNOWN_FINAL_REFERENCES.has(row.final_reference)) {
      throw new Error(`Structure reference route baseline references unknown final PNG: ${row.final_reference}`);
    }
  }
}

function validateInvariants(actual) {
  if (!Array.isArray(actual) || actual.length !== STRUCTURE_REFERENCE_ROUTE_BASELINE_INVARIANTS.length) {
    throw new Error(
      `Structure reference route baseline invariant count is ${actual?.length}; expected ${STRUCTURE_REFERENCE_ROUTE_BASELINE_INVARIANTS.length}`,
    );
  }
  for (let index = 0; index < STRUCTURE_REFERENCE_ROUTE_BASELINE_INVARIANTS.length; index += 1) {
    if (actual[index] !== STRUCTURE_REFERENCE_ROUTE_BASELINE_INVARIANTS[index]) {
      throw new Error(
        `Structure reference route baseline invariant ${index + 1} is ${actual[index]}; expected ${STRUCTURE_REFERENCE_ROUTE_BASELINE_INVARIANTS[index]}`,
      );
    }
  }
}
