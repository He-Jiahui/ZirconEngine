import { existsSync, readFileSync } from "node:fs";
import { CANVAS_HEIGHT, CANVAS_WIDTH } from "../hub-web-reference/page-registry.mjs";
import {
  STRUCTURE_GEOMETRY_BASELINE,
  STRUCTURE_GEOMETRY_BASELINE_EXPECTED_REGIONS,
  STRUCTURE_GEOMETRY_BASELINE_REQUIRED_FIELDS,
} from "./board-registry.mjs";

const REGION_KEYS = ["left", "top", "width", "height", "right", "bottom"];

export function validateStructureGeometryBaselineFile(filePath) {
  const baseline = readBaseline(filePath);
  validateBaselineShape(baseline);
  validateExpectedRegionRows(baseline);
}

export function validateStructureGeometryBaselineAgainstMeasured(filePath, geometry) {
  const baseline = readBaseline(filePath);
  const measured = measuredRegionRows(geometry);

  if (baseline.dimension_strip?.height !== geometry.scaleStripHeight) {
    throw new Error(
      `Structure geometry baseline dimension strip height is ${baseline.dimension_strip?.height}; expected ${geometry.scaleStripHeight}`,
    );
  }

  for (let index = 0; index < measured.length; index += 1) {
    const expected = measured[index];
    const actual = baseline.regions[index];
    for (const key of ["id", "label", ...REGION_KEYS]) {
      if (actual?.[key] !== expected[key]) {
        throw new Error(
          `Structure geometry baseline ${expected.id} ${key} is ${actual?.[key]}; expected ${expected[key]}`,
        );
      }
    }
  }
}

function readBaseline(filePath) {
  if (!existsSync(filePath)) {
    throw new Error(`Missing structure geometry baseline: ${filePath}`);
  }
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function validateBaselineShape(baseline) {
  for (const field of STRUCTURE_GEOMETRY_BASELINE_REQUIRED_FIELDS) {
    if (!(field in baseline)) {
      throw new Error(`Structure geometry baseline missing required field: ${field}`);
    }
  }
  if (STRUCTURE_GEOMETRY_BASELINE !== "docs/ui-and-layout/hub-design-board/structure-geometry-baseline.json") {
    throw new Error(`Unexpected structure geometry baseline path: ${STRUCTURE_GEOMETRY_BASELINE}`);
  }
  if (baseline.source !== "docs/ui-and-layout/hub-design-board/index.html?board=structure") {
    throw new Error(`Structure geometry baseline source is ${baseline.source}`);
  }
  if (baseline.artifact !== "docs/ui-and-layout/hub-design-structure-layout.png") {
    throw new Error(`Structure geometry baseline artifact is ${baseline.artifact}`);
  }
  if (baseline.canvas?.width !== CANVAS_WIDTH || baseline.canvas?.height !== CANVAS_HEIGHT) {
    throw new Error(`Structure geometry baseline canvas is ${baseline.canvas?.width}x${baseline.canvas?.height}`);
  }
  if (baseline.measurement_space !== ".hub-frame rounded CSS pixels") {
    throw new Error(`Structure geometry baseline measurement space is ${baseline.measurement_space}`);
  }
  if (baseline.dimension_strip?.height !== 32) {
    throw new Error(`Structure geometry baseline dimension strip height is ${baseline.dimension_strip?.height}`);
  }
  if (!Array.isArray(baseline.regions) || baseline.regions.length !== STRUCTURE_GEOMETRY_BASELINE_EXPECTED_REGIONS.length) {
    throw new Error(
      `Structure geometry baseline region count is ${baseline.regions?.length}; expected ${STRUCTURE_GEOMETRY_BASELINE_EXPECTED_REGIONS.length}`,
    );
  }
  if (!Array.isArray(baseline.relationships) || baseline.relationships.length < 5) {
    throw new Error("Structure geometry baseline must list shell relationship rules");
  }
}

function validateExpectedRegionRows(baseline) {
  for (let index = 0; index < STRUCTURE_GEOMETRY_BASELINE_EXPECTED_REGIONS.length; index += 1) {
    const expected = STRUCTURE_GEOMETRY_BASELINE_EXPECTED_REGIONS[index];
    const actual = baseline.regions[index];
    for (const key of ["id", "label", ...REGION_KEYS]) {
      if (actual?.[key] !== expected[key]) {
        throw new Error(
          `Structure geometry baseline region ${index + 1} ${key} is ${actual?.[key]}; expected ${expected[key]}`,
        );
      }
    }
  }
}

function measuredRegionRows(geometry) {
  return [
    { id: "hub-frame", label: "Hub frame", ...geometry.frame, left: 0, top: 0, right: geometry.frame.width, bottom: geometry.frame.height },
    { id: "topbar", label: "Topbar", ...geometry.topbar },
    { id: "sidebar", label: "Sidebar", ...geometry.sidebar },
    { id: "workspace", label: "Workspace", ...geometry.workspace },
    { id: "bottom-strip", label: "Bottom strip", ...geometry.bottom },
    { id: "source-engine-overlay", label: "Source Engine overlay", ...geometry.sourceOverlay },
    { id: "account-overlay", label: "Account overlay", ...geometry.accountOverlay },
  ];
}
