import { existsSync, readFileSync } from "node:fs";
import {
  STRUCTURE_OVERLAY_BASELINE,
  STRUCTURE_OVERLAY_BASELINE_EXPECTED_LAYERS,
  STRUCTURE_OVERLAY_BASELINE_INVARIANTS,
  STRUCTURE_OVERLAY_BASELINE_REQUIRED_FIELDS,
} from "./board-registry.mjs";

export function validateStructureOverlayBaselineFile(filePath) {
  const baseline = readBaseline(filePath);
  for (const field of STRUCTURE_OVERLAY_BASELINE_REQUIRED_FIELDS) {
    if (!(field in baseline)) {
      throw new Error(`Structure overlay baseline missing required field: ${field}`);
    }
  }
  if (STRUCTURE_OVERLAY_BASELINE !== "docs/ui-and-layout/hub-design-board/structure-overlay-baseline.json") {
    throw new Error(`Unexpected structure overlay baseline path: ${STRUCTURE_OVERLAY_BASELINE}`);
  }
  if (baseline.source !== "docs/ui-and-layout/hub-design-board/index.html") {
    throw new Error(`Structure overlay baseline source is ${baseline.source}`);
  }
  if (baseline.review_focus !== "overlay-ownership-no-reflow") {
    throw new Error(`Structure overlay baseline review focus is ${baseline.review_focus}`);
  }
  validateOverlayLayers(baseline.overlay_layers);
  validateInvariants(baseline.invariants);
}

function readBaseline(filePath) {
  if (!existsSync(filePath)) {
    throw new Error(`Missing structure overlay baseline: ${filePath}`);
  }
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function validateOverlayLayers(actual) {
  if (!Array.isArray(actual) || actual.length !== STRUCTURE_OVERLAY_BASELINE_EXPECTED_LAYERS.length) {
    throw new Error(
      `Structure overlay baseline layer count is ${actual?.length}; expected ${STRUCTURE_OVERLAY_BASELINE_EXPECTED_LAYERS.length}`,
    );
  }
  for (let index = 0; index < STRUCTURE_OVERLAY_BASELINE_EXPECTED_LAYERS.length; index += 1) {
    const expected = STRUCTURE_OVERLAY_BASELINE_EXPECTED_LAYERS[index];
    const row = actual[index];
    for (const key of ["id", "owner", "primary_artifact", "anchor_region", "layout_effect"]) {
      if (row?.[key] !== expected[key]) {
        throw new Error(`Structure overlay baseline row ${index + 1} ${key} is ${row?.[key]}; expected ${expected[key]}`);
      }
    }
  }
}

function validateInvariants(actual) {
  if (!Array.isArray(actual) || actual.length !== STRUCTURE_OVERLAY_BASELINE_INVARIANTS.length) {
    throw new Error(
      `Structure overlay baseline invariant count is ${actual?.length}; expected ${STRUCTURE_OVERLAY_BASELINE_INVARIANTS.length}`,
    );
  }
  for (let index = 0; index < STRUCTURE_OVERLAY_BASELINE_INVARIANTS.length; index += 1) {
    if (actual[index] !== STRUCTURE_OVERLAY_BASELINE_INVARIANTS[index]) {
      throw new Error(
        `Structure overlay baseline invariant ${index + 1} is ${actual[index]}; expected ${STRUCTURE_OVERLAY_BASELINE_INVARIANTS[index]}`,
      );
    }
  }
}
