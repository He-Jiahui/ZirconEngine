import { existsSync, readFileSync } from "node:fs";
import {
  STRUCTURE_REVIEW_ROUTE_BASELINE,
  STRUCTURE_REVIEW_ROUTE_BASELINE_BLOCKING_RULES,
  STRUCTURE_REVIEW_ROUTE_BASELINE_EXPECTED_ORDER,
  STRUCTURE_REVIEW_ROUTE_BASELINE_REQUIRED_FIELDS,
} from "./board-registry.mjs";

export function validateStructureReviewRouteBaselineFile(filePath) {
  const baseline = readBaseline(filePath);
  for (const field of STRUCTURE_REVIEW_ROUTE_BASELINE_REQUIRED_FIELDS) {
    if (!(field in baseline)) {
      throw new Error(`Structure review route baseline missing required field: ${field}`);
    }
  }
  if (STRUCTURE_REVIEW_ROUTE_BASELINE !== "docs/ui-and-layout/hub-design-board/structure-review-route-baseline.json") {
    throw new Error(`Unexpected structure review route baseline path: ${STRUCTURE_REVIEW_ROUTE_BASELINE}`);
  }
  if (baseline.source !== "docs/ui-and-layout/hub-design-board/index.html") {
    throw new Error(`Structure review route baseline source is ${baseline.source}`);
  }
  if (baseline.review_focus !== "overall-structure-first-review-route") {
    throw new Error(`Structure review route baseline review focus is ${baseline.review_focus}`);
  }
  validateReviewOrder(baseline.review_order);
  validateBlockingRules(baseline.blocking_rules);
}

function readBaseline(filePath) {
  if (!existsSync(filePath)) {
    throw new Error(`Missing structure review route baseline: ${filePath}`);
  }
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function validateReviewOrder(actual) {
  if (!Array.isArray(actual) || actual.length !== STRUCTURE_REVIEW_ROUTE_BASELINE_EXPECTED_ORDER.length) {
    throw new Error(
      `Structure review route baseline review_order count is ${actual?.length}; expected ${STRUCTURE_REVIEW_ROUTE_BASELINE_EXPECTED_ORDER.length}`,
    );
  }
  for (let index = 0; index < STRUCTURE_REVIEW_ROUTE_BASELINE_EXPECTED_ORDER.length; index += 1) {
    const expected = STRUCTURE_REVIEW_ROUTE_BASELINE_EXPECTED_ORDER[index];
    const row = actual[index];
    for (const key of ["step", "artifact_id", "artifact", "focus", "decision_gate"]) {
      if (row?.[key] !== expected[key]) {
        throw new Error(`Structure review route baseline row ${index + 1} ${key} is ${row?.[key]}; expected ${expected[key]}`);
      }
    }
  }
  if (actual[0].artifact_id !== "structure" || actual.at(-1).artifact_id !== "details") {
    throw new Error("Structure review route baseline must put the structure board first and details board last");
  }
}

function validateBlockingRules(actual) {
  if (!Array.isArray(actual) || actual.length !== STRUCTURE_REVIEW_ROUTE_BASELINE_BLOCKING_RULES.length) {
    throw new Error(
      `Structure review route baseline blocking rule count is ${actual?.length}; expected ${STRUCTURE_REVIEW_ROUTE_BASELINE_BLOCKING_RULES.length}`,
    );
  }
  for (let index = 0; index < STRUCTURE_REVIEW_ROUTE_BASELINE_BLOCKING_RULES.length; index += 1) {
    if (actual[index] !== STRUCTURE_REVIEW_ROUTE_BASELINE_BLOCKING_RULES[index]) {
      throw new Error(
        `Structure review route baseline blocking rule ${index + 1} is ${actual[index]}; expected ${STRUCTURE_REVIEW_ROUTE_BASELINE_BLOCKING_RULES[index]}`,
      );
    }
  }
}
