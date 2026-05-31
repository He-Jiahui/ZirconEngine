import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdtempSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { inflateSync } from "node:zlib";
import { CANVAS_HEIGHT, CANVAS_WIDTH, DASHBOARD_CAPTURE_NAME, EXPORTS_LIST } from "../hub-web-reference/page-registry.mjs";
import {
  DESIGN_BOARD_EXPORT_HASH_INPUTS,
  DESIGN_BOARD_EXPORT_METADATA,
  DESIGN_BOARD_REVIEW_INDEX,
  DESIGN_BOARD_REVIEW_INDEX_REQUIRED_TEXT,
  DESIGN_BOARD_LIST,
  DESIGN_BOARD_MANIFEST,
  DESIGN_BOARD_MANIFEST_SCHEMA,
  DESIGN_BOARD_SOURCE,
  HUB_REFERENCE_TARGET,
  REFERENCE_ALIGNMENT_MATRIX,
  REFERENCE_ALIGNMENT_MATRIX_REQUIRED_TEXT,
  REFERENCE_ALIGNMENT_CHECKS,
  STRUCTURE_SIGNOFF_CHECKLIST,
  STRUCTURE_SIGNOFF_CHECKLIST_REQUIRED_TEXT,
  STRUCTURE_DECISION_LOG,
  STRUCTURE_ACCEPTANCE_RECORD,
  STRUCTURE_ACCEPTANCE_RECORD_REQUIRED_TEXT,
  STRUCTURE_REVIEW_PACKET,
  STRUCTURE_REVIEW_PACKET_REQUIRED_FIELDS,
  STRUCTURE_REVIEW_PACKET_REQUIRED_COMMANDS,
  STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE,
  STRUCTURE_REVIEW_PACKET_SCHEMA,
  STRUCTURE_REVIEW_STATUS,
  STRUCTURE_REVIEW_STATUS_REQUIRED_TEXT,
  STRUCTURE_GEOMETRY_BASELINE,
  STRUCTURE_GEOMETRY_EVIDENCE,
  STRUCTURE_GEOMETRY_REQUIRED_TEXT,
  STRUCTURE_RESPONSIVE_BASELINE,
  STRUCTURE_REVIEW_ROUTE_BASELINE,
  STRUCTURE_OVERLAY_BASELINE,
  STRUCTURE_REFERENCE_ROUTE_BASELINE,
  STRUCTURE_COVERAGE_MATRIX,
  STRUCTURE_COVERAGE_EXPECTED_ROWS,
  STRUCTURE_COVERAGE_REQUIRED_ITEMS,
  STRUCTURE_REVIEW_CHECKLIST,
  STRUCTURE_REVIEW_GUIDE,
  STRUCTURE_REVIEW_GUIDE_REQUIRED_TEXT,
  STRUCTURE_REVIEW_REQUIRED_TEXT,
  STRUCTURE_TO_REFERENCE_MAP,
  STRUCTURE_TO_REFERENCE_REQUIRED_TEXT,
} from "./board-registry.mjs";
import {
  validateStructureGeometryBaselineAgainstMeasured,
  validateStructureGeometryBaselineFile,
} from "./structure-geometry-baseline-validator.mjs";
import { validateStructureDecisionLogDocument } from "./structure-decision-log-validator.mjs";
import { validateStructureResponsiveBaselineFile } from "./structure-responsive-baseline-validator.mjs";
import { validateStructureReviewRouteBaselineFile } from "./structure-review-route-baseline-validator.mjs";
import { validateStructureOverlayBaselineFile } from "./structure-overlay-baseline-validator.mjs";
import { validateStructureReferenceRouteBaselineFile } from "./structure-reference-route-baseline-validator.mjs";

const MIN_FILE_SIZE = 20_000;
const MIN_DYNAMIC_RANGE = 80;
const MIN_AVERAGE_LUMA = 8;
const MAX_AVERAGE_LUMA = 245;
const here = dirname(fileURLToPath(import.meta.url));
const outputDir = resolve(here, "..");
const exportMetadataPath = resolve(here, "export-metadata.json");
const exportsIndex = resolve(here, "EXPORTS.md");
const manifestPath = resolve(here, "manifest.json");
const manifestSchemaPath = resolve(here, "manifest.schema.json");
const referenceAlignmentMatrixPath = resolve(here, "REFERENCE_ALIGNMENT_MATRIX.md");
const reviewIndexPath = resolve(here, "REVIEW_INDEX.md");
const sourcePath = resolve(here, "index.html");
const structureAcceptanceRecordPath = resolve(here, "STRUCTURE_ACCEPTANCE_RECORD.md");
const structureCoveragePath = resolve(here, "STRUCTURE_COVERAGE_MATRIX.md");
const structureGeometryBaselinePath = resolve(here, "structure-geometry-baseline.json");
const structureGeometryPath = resolve(here, "STRUCTURE_GEOMETRY_EVIDENCE.md");
const structureResponsiveBaselinePath = resolve(here, "structure-responsive-baseline.json");
const structureReviewRouteBaselinePath = resolve(here, "structure-review-route-baseline.json");
const structureOverlayBaselinePath = resolve(here, "structure-overlay-baseline.json");
const structureReferenceRouteBaselinePath = resolve(here, "structure-reference-route-baseline.json");
const structureReviewPacketPath = resolve(here, "structure-review-packet.json");
const structureReviewPacketSchemaPath = resolve(here, "structure-review-packet.schema.json");
const structureReviewPath = resolve(here, "STRUCTURE_REVIEW.md");
const structureReviewGuidePath = resolve(here, "STRUCTURE_REVIEW_GUIDE.md");
const structureSignoffChecklistPath = resolve(here, "STRUCTURE_SIGNOFF_CHECKLIST.md");
const structureDecisionLogPath = resolve(here, "STRUCTURE_DECISION_LOG.md");
const structureReviewStatusPath = resolve(here, "STRUCTURE_REVIEW_STATUS.md");
const structureToReferenceMapPath = resolve(here, "STRUCTURE_TO_REFERENCE_MAP.md");

validateManifest();
validateManifestSchema();
validateStructureReview();
validateStructureCoverageMatrix();
validateStructureGeometryEvidenceDocument();
validateStructureGeometryBaselineFile(structureGeometryBaselinePath);
validateStructureResponsiveBaselineFile(structureResponsiveBaselinePath);
validateStructureReviewRouteBaselineFile(structureReviewRouteBaselinePath);
validateStructureOverlayBaselineFile(structureOverlayBaselinePath);
validateStructureReferenceRouteBaselineFile(structureReferenceRouteBaselinePath);
validateStructureReviewGuide();
validateStructureReviewStatus();
validateStructureAcceptanceRecord();
validateStructureReviewPacket();
validateStructureReviewPacketSchema();
validateJsonSchemaSubsetSelfTest();
validateReviewIndex();
validateStructureToReferenceMap();
validateReferenceAlignmentMatrix();
validateStructureSignoffChecklist();
validateStructureDecisionLogDocument(structureDecisionLogPath);
validateExportsIndex();
for (const { output } of DESIGN_BOARD_LIST) {
  validatePng(resolve(outputDir, output));
}
validateExportMetadata();
await validateBrowserBoards();

console.log(`validated ${DESIGN_BOARD_LIST.length} Hub design-board PNGs at ${CANVAS_WIDTH}x${CANVAS_HEIGHT}`);
console.log("validated design-board PNG dynamic range and brightness");
console.log("validated design-board manifest.json against board-registry.mjs");
console.log("validated design-board manifest.schema.json against board-registry.mjs");
console.log("validated Hub reference target PNG against board-registry.mjs");
console.log("validated Hub reference alignment checks against board-registry.mjs");
console.log("validated design-board EXPORTS.md against board-registry.mjs");
console.log("validated design-board export metadata hashes");
console.log("validated STRUCTURE_REVIEW.md against board-registry.mjs");
console.log("validated STRUCTURE_COVERAGE_MATRIX.md against board-registry.mjs");
console.log("validated STRUCTURE_GEOMETRY_EVIDENCE.md against board-registry.mjs");
console.log("validated structure-geometry-baseline.json against measured geometry");
console.log("validated structure-responsive-baseline.json against board-registry.mjs");
console.log("validated structure-review-route-baseline.json against board-registry.mjs");
console.log("validated structure-overlay-baseline.json against board-registry.mjs");
console.log("validated structure-reference-route-baseline.json against board-registry.mjs");
console.log("validated STRUCTURE_REVIEW_GUIDE.md against board-registry.mjs");
console.log("validated STRUCTURE_REVIEW_STATUS.md against board-registry.mjs");
console.log("validated STRUCTURE_ACCEPTANCE_RECORD.md against board-registry.mjs");
console.log("validated structure-review-packet.json against board-registry.mjs");
console.log("validated structure-review-packet.schema.json against board-registry.mjs");
console.log("validated JSON schema subset self-test");
console.log("validated REVIEW_INDEX.md against board-registry.mjs");
console.log("validated STRUCTURE_TO_REFERENCE_MAP.md against board-registry.mjs");
console.log("validated REFERENCE_ALIGNMENT_MATRIX.md against board-registry.mjs");
console.log("validated STRUCTURE_SIGNOFF_CHECKLIST.md against board-registry.mjs");
console.log("validated STRUCTURE_DECISION_LOG.md against board-registry.mjs");
console.log("validated design-board pages fit the fixed canvas without scroll overflow");
console.log("validated primary structure board geometry boundaries");
console.log("validated primary structure board key-label fit");

function validateManifest() {
  if (!existsSync(manifestPath)) {
    throw new Error(`Missing design board manifest: ${manifestPath}`);
  }

  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  if (DESIGN_BOARD_MANIFEST !== "docs/ui-and-layout/hub-design-board/manifest.json") {
    throw new Error(`Unexpected design board manifest path: ${DESIGN_BOARD_MANIFEST}`);
  }
  if (DESIGN_BOARD_MANIFEST_SCHEMA !== "docs/ui-and-layout/hub-design-board/manifest.schema.json") {
    throw new Error(`Unexpected design board manifest schema path: ${DESIGN_BOARD_MANIFEST_SCHEMA}`);
  }
  const schema = JSON.parse(readFileSync(manifestSchemaPath, "utf8"));
  validateJsonSchemaSubset(manifest, schema, "$manifest");
  if (manifest.schema !== DESIGN_BOARD_MANIFEST_SCHEMA) {
    throw new Error(`Design board manifest schema is ${manifest.schema}; expected ${DESIGN_BOARD_MANIFEST_SCHEMA}`);
  }
  if (manifest.source !== DESIGN_BOARD_SOURCE) {
    throw new Error(`Design board manifest source is ${manifest.source}; expected ${DESIGN_BOARD_SOURCE}`);
  }
  if (manifest.canvas?.width !== CANVAS_WIDTH || manifest.canvas?.height !== CANVAS_HEIGHT) {
    throw new Error(
      `Design board manifest canvas is ${manifest.canvas?.width}x${manifest.canvas?.height}; expected ${CANVAS_WIDTH}x${CANVAS_HEIGHT}`,
    );
  }
  if (manifest.review_priority !== "overall-interaction-structure-layout") {
    throw new Error(`Design board manifest review_priority is ${manifest.review_priority}`);
  }
  validateManualSignoff("Design board manifest manual_signoff", manifest.manual_signoff);
  validateReferenceTarget("Design board manifest reference_target", manifest.reference_target);
  validateReferenceAlignmentChecks("Design board manifest reference_alignment_checks", manifest.reference_alignment_checks);
  if (manifest.structure_review_checklist !== STRUCTURE_REVIEW_CHECKLIST) {
    throw new Error(
      `Design board manifest structure_review_checklist is ${manifest.structure_review_checklist}; expected ${STRUCTURE_REVIEW_CHECKLIST}`,
    );
  }
  if (manifest.structure_coverage_matrix !== STRUCTURE_COVERAGE_MATRIX) {
    throw new Error(
      `Design board manifest structure_coverage_matrix is ${manifest.structure_coverage_matrix}; expected ${STRUCTURE_COVERAGE_MATRIX}`,
    );
  }
  if (manifest.structure_geometry_evidence !== STRUCTURE_GEOMETRY_EVIDENCE) {
    throw new Error(
      `Design board manifest structure_geometry_evidence is ${manifest.structure_geometry_evidence}; expected ${STRUCTURE_GEOMETRY_EVIDENCE}`,
    );
  }
  if (manifest.structure_geometry_baseline !== STRUCTURE_GEOMETRY_BASELINE) {
    throw new Error(
      `Design board manifest structure_geometry_baseline is ${manifest.structure_geometry_baseline}; expected ${STRUCTURE_GEOMETRY_BASELINE}`,
    );
  }
  if (manifest.structure_responsive_baseline !== STRUCTURE_RESPONSIVE_BASELINE) {
    throw new Error(
      `Design board manifest structure_responsive_baseline is ${manifest.structure_responsive_baseline}; expected ${STRUCTURE_RESPONSIVE_BASELINE}`,
    );
  }
  if (manifest.structure_review_route_baseline !== STRUCTURE_REVIEW_ROUTE_BASELINE) {
    throw new Error(
      `Design board manifest structure_review_route_baseline is ${manifest.structure_review_route_baseline}; expected ${STRUCTURE_REVIEW_ROUTE_BASELINE}`,
    );
  }
  if (manifest.structure_overlay_baseline !== STRUCTURE_OVERLAY_BASELINE) {
    throw new Error(
      `Design board manifest structure_overlay_baseline is ${manifest.structure_overlay_baseline}; expected ${STRUCTURE_OVERLAY_BASELINE}`,
    );
  }
  if (manifest.structure_reference_route_baseline !== STRUCTURE_REFERENCE_ROUTE_BASELINE) {
    throw new Error(
      `Design board manifest structure_reference_route_baseline is ${manifest.structure_reference_route_baseline}; expected ${STRUCTURE_REFERENCE_ROUTE_BASELINE}`,
    );
  }
  if (manifest.structure_review_guide !== STRUCTURE_REVIEW_GUIDE) {
    throw new Error(
      `Design board manifest structure_review_guide is ${manifest.structure_review_guide}; expected ${STRUCTURE_REVIEW_GUIDE}`,
    );
  }
  if (manifest.review_index !== DESIGN_BOARD_REVIEW_INDEX) {
    throw new Error(`Design board manifest review_index is ${manifest.review_index}; expected ${DESIGN_BOARD_REVIEW_INDEX}`);
  }
  if (manifest.structure_to_reference_map !== STRUCTURE_TO_REFERENCE_MAP) {
    throw new Error(
      `Design board manifest structure_to_reference_map is ${manifest.structure_to_reference_map}; expected ${STRUCTURE_TO_REFERENCE_MAP}`,
    );
  }
  if (manifest.reference_alignment_matrix !== REFERENCE_ALIGNMENT_MATRIX) {
    throw new Error(
      `Design board manifest reference_alignment_matrix is ${manifest.reference_alignment_matrix}; expected ${REFERENCE_ALIGNMENT_MATRIX}`,
    );
  }
  if (manifest.structure_signoff_checklist !== STRUCTURE_SIGNOFF_CHECKLIST) {
    throw new Error(
      `Design board manifest structure_signoff_checklist is ${manifest.structure_signoff_checklist}; expected ${STRUCTURE_SIGNOFF_CHECKLIST}`,
    );
  }
  if (manifest.structure_decision_log !== STRUCTURE_DECISION_LOG) {
    throw new Error(
      `Design board manifest structure_decision_log is ${manifest.structure_decision_log}; expected ${STRUCTURE_DECISION_LOG}`,
    );
  }
  if (manifest.structure_review_status !== STRUCTURE_REVIEW_STATUS) {
    throw new Error(
      `Design board manifest structure_review_status is ${manifest.structure_review_status}; expected ${STRUCTURE_REVIEW_STATUS}`,
    );
  }
  if (manifest.structure_acceptance_record !== STRUCTURE_ACCEPTANCE_RECORD) {
    throw new Error(
      `Design board manifest structure_acceptance_record is ${manifest.structure_acceptance_record}; expected ${STRUCTURE_ACCEPTANCE_RECORD}`,
    );
  }
  if (manifest.structure_review_packet !== STRUCTURE_REVIEW_PACKET) {
    throw new Error(
      `Design board manifest structure_review_packet is ${manifest.structure_review_packet}; expected ${STRUCTURE_REVIEW_PACKET}`,
    );
  }
  if (manifest.structure_review_packet_schema !== STRUCTURE_REVIEW_PACKET_SCHEMA) {
    throw new Error(
      `Design board manifest structure_review_packet_schema is ${manifest.structure_review_packet_schema}; expected ${STRUCTURE_REVIEW_PACKET_SCHEMA}`,
    );
  }
  if (manifest.export_metadata !== DESIGN_BOARD_EXPORT_METADATA) {
    throw new Error(
      `Design board manifest export_metadata is ${manifest.export_metadata}; expected ${DESIGN_BOARD_EXPORT_METADATA}`,
    );
  }
  validateStringArray("Design board manifest support_documents", manifest.support_documents, manifestSupportDocuments());
  validateRepoRelativeFilesExist("Design board manifest support_documents", manifest.support_documents);
  validateStringArray("Design board manifest validation_commands", manifest.validation_commands, STRUCTURE_REVIEW_PACKET_REQUIRED_COMMANDS);
  validateAcceptanceBoundary("Design board manifest acceptance_boundary", manifest.acceptance_boundary);
  if (!Array.isArray(manifest.artifacts)) {
    throw new Error("Design board manifest artifacts must be an array");
  }
  if (manifest.artifacts.length !== DESIGN_BOARD_LIST.length) {
    throw new Error(`Design board manifest lists ${manifest.artifacts.length} artifacts; expected ${DESIGN_BOARD_LIST.length}`);
  }

  const primaryCount = manifest.artifacts.filter((artifact) => artifact.category === "primary-structure").length;
  const secondaryCount = manifest.artifacts.filter((artifact) => artifact.category === "secondary-functional-detail").length;
  if (primaryCount !== 2 || secondaryCount !== 1) {
    throw new Error(`Design board manifest category split is primary=${primaryCount}, secondary=${secondaryCount}; expected 2/1`);
  }

  for (let index = 0; index < DESIGN_BOARD_LIST.length; index += 1) {
    const expected = DESIGN_BOARD_LIST[index];
    const actual = manifest.artifacts[index];
    if (actual.id !== expected.id || actual.output !== expected.output || actual.category !== expected.category) {
      throw new Error(
        `Design board manifest artifact ${index + 1} is ${actual.id}/${actual.output}/${actual.category}; expected ${expected.id}/${expected.output}/${expected.category}`,
      );
    }
  }
  validateManifestReviewSequence(manifest.review_sequence);
  validateManualReviewItems("Design board manifest manual_review_items", manifest.manual_review_items);
}

function validateManifestSchema() {
  if (!existsSync(manifestSchemaPath)) {
    throw new Error(`Missing design board manifest schema: ${manifestSchemaPath}`);
  }
  const schema = JSON.parse(readFileSync(manifestSchemaPath, "utf8"));
  if (schema.$schema !== "https://json-schema.org/draft/2020-12/schema") {
    throw new Error(`Design board manifest schema draft is ${schema.$schema}`);
  }
  if (schema.$id !== DESIGN_BOARD_MANIFEST_SCHEMA) {
    throw new Error(`Design board manifest schema id is ${schema.$id}; expected ${DESIGN_BOARD_MANIFEST_SCHEMA}`);
  }
  if (schema.type !== "object" || schema.additionalProperties !== false) {
    throw new Error("Design board manifest schema must define a closed object");
  }
  validateStringArray("Design board manifest schema required fields", schema.required, manifestRequiredFields());
  if (schema.properties?.schema?.const !== DESIGN_BOARD_MANIFEST_SCHEMA) {
    throw new Error("Design board manifest schema must pin the manifest schema path");
  }
  if (schema.properties?.source?.const !== DESIGN_BOARD_SOURCE) {
    throw new Error("Design board manifest schema must pin the design-board source");
  }
  if (
    schema.properties?.canvas?.properties?.width?.const !== CANVAS_WIDTH ||
    schema.properties?.canvas?.properties?.height?.const !== CANVAS_HEIGHT
  ) {
    throw new Error("Design board manifest schema must pin the 1568x1003 canvas");
  }
  if (schema.properties?.artifacts?.minItems !== DESIGN_BOARD_LIST.length || schema.properties?.artifacts?.maxItems !== DESIGN_BOARD_LIST.length) {
    throw new Error("Design board manifest schema must pin the artifact count");
  }
  if (
    schema.properties?.review_sequence?.minItems !== STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length ||
    schema.properties?.review_sequence?.maxItems !== STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length
  ) {
    throw new Error("Design board manifest schema must pin the review sequence count");
  }
  if (
    schema.properties?.manual_review_items?.minItems !== STRUCTURE_COVERAGE_EXPECTED_ROWS.length ||
    schema.properties?.manual_review_items?.maxItems !== STRUCTURE_COVERAGE_EXPECTED_ROWS.length
  ) {
    throw new Error("Design board manifest schema must pin the manual review item count");
  }
  if (schema.properties?.reference_alignment_matrix?.const !== REFERENCE_ALIGNMENT_MATRIX) {
    throw new Error("Design board manifest schema must pin the reference alignment matrix");
  }
  if (schema.properties?.structure_signoff_checklist?.const !== STRUCTURE_SIGNOFF_CHECKLIST) {
    throw new Error("Design board manifest schema must pin the structure signoff checklist");
  }
  if (schema.properties?.structure_geometry_baseline?.const !== STRUCTURE_GEOMETRY_BASELINE) {
    throw new Error("Design board manifest schema must pin the structure geometry baseline");
  }
  if (schema.properties?.structure_responsive_baseline?.const !== STRUCTURE_RESPONSIVE_BASELINE) {
    throw new Error("Design board manifest schema must pin the structure responsive baseline");
  }
  if (schema.properties?.structure_review_route_baseline?.const !== STRUCTURE_REVIEW_ROUTE_BASELINE) {
    throw new Error("Design board manifest schema must pin the structure review route baseline");
  }
  if (schema.properties?.structure_overlay_baseline?.const !== STRUCTURE_OVERLAY_BASELINE) {
    throw new Error("Design board manifest schema must pin the structure overlay baseline");
  }
  if (schema.properties?.structure_reference_route_baseline?.const !== STRUCTURE_REFERENCE_ROUTE_BASELINE) {
    throw new Error("Design board manifest schema must pin the structure reference route baseline");
  }
  if (schema.properties?.structure_decision_log?.const !== STRUCTURE_DECISION_LOG) {
    throw new Error("Design board manifest schema must pin the structure decision log");
  }
  if (
    schema.properties?.reference_alignment_checks?.minItems !== REFERENCE_ALIGNMENT_CHECKS.length ||
    schema.properties?.reference_alignment_checks?.maxItems !== REFERENCE_ALIGNMENT_CHECKS.length
  ) {
    throw new Error("Design board manifest schema must pin reference alignment check count");
  }
  if (
    schema.properties?.support_documents?.minItems !== manifestSupportDocuments().length ||
    schema.properties?.support_documents?.maxItems !== manifestSupportDocuments().length
  ) {
    throw new Error("Design board manifest schema must pin support document count");
  }
  if (schema.properties?.validation_commands?.minItems !== STRUCTURE_REVIEW_PACKET_REQUIRED_COMMANDS.length) {
    throw new Error("Design board manifest schema must pin validation command count");
  }
  if (schema.properties?.manual_signoff?.properties?.status?.const !== "pending") {
    throw new Error("Design board manifest schema must keep manual signoff pending");
  }
  if (
    schema.properties?.manual_signoff?.properties?.required_primary_artifact?.const !==
    "docs/ui-and-layout/hub-design-structure-layout.png"
  ) {
    throw new Error("Design board manifest schema must pin the primary signoff artifact");
  }
  validateReferenceTargetSchema("Design board manifest schema reference_target", schema.properties?.reference_target);
  if (schema.properties?.acceptance_boundary?.minItems !== 3 || schema.properties?.acceptance_boundary?.maxItems !== 3) {
    throw new Error("Design board manifest schema must pin acceptance boundary count");
  }
}

function validateStructureReview() {
  if (!existsSync(structureReviewPath)) {
    throw new Error(`Missing structure review checklist: ${structureReviewPath}`);
  }

  const content = readFileSync(structureReviewPath, "utf8");
  for (const snippet of STRUCTURE_REVIEW_REQUIRED_TEXT) {
    if (!content.includes(snippet)) {
      throw new Error(`Structure review checklist missing required text: ${snippet}`);
    }
  }

  for (const { output } of DESIGN_BOARD_LIST) {
    const expectedPath = `docs/ui-and-layout/${output}`;
    if (!content.includes(expectedPath)) {
      throw new Error(`Structure review checklist must reference ${expectedPath}`);
    }
  }
  if (STRUCTURE_REVIEW_CHECKLIST !== "docs/ui-and-layout/hub-design-board/STRUCTURE_REVIEW.md") {
    throw new Error(`Unexpected structure review checklist path: ${STRUCTURE_REVIEW_CHECKLIST}`);
  }
}

function validateStructureCoverageMatrix() {
  if (!existsSync(structureCoveragePath)) {
    throw new Error(`Missing structure coverage matrix: ${structureCoveragePath}`);
  }

  const content = readFileSync(structureCoveragePath, "utf8");
  for (const snippet of STRUCTURE_COVERAGE_REQUIRED_ITEMS) {
    if (!content.includes(snippet)) {
      throw new Error(`Structure coverage matrix missing required item: ${snippet}`);
    }
  }
  for (const { output } of DESIGN_BOARD_LIST) {
    if (!content.includes(output)) {
      throw new Error(`Structure coverage matrix must reference ${output}`);
    }
  }

  const artifactCategories = new Map(DESIGN_BOARD_LIST.map((board) => [board.output, board.category]));
  const rows = parseCoverageRows(content);
  if (rows.length !== STRUCTURE_COVERAGE_EXPECTED_ROWS.length) {
    throw new Error(
      `Structure coverage matrix has ${rows.length} review rows; expected ${STRUCTURE_COVERAGE_EXPECTED_ROWS.length}`,
    );
  }
  for (let index = 0; index < STRUCTURE_COVERAGE_EXPECTED_ROWS.length; index += 1) {
    const expected = STRUCTURE_COVERAGE_EXPECTED_ROWS[index];
    const actual = rows[index];
    if (
      actual.reviewItem !== expected.reviewItem ||
      actual.primaryArtifact !== expected.primaryArtifact ||
      actual.secondaryArtifact !== expected.secondaryArtifact
    ) {
      throw new Error(
        `Structure coverage row ${index + 1} is ${actual.reviewItem}/${actual.primaryArtifact}/${actual.secondaryArtifact}; expected ${expected.reviewItem}/${expected.primaryArtifact}/${expected.secondaryArtifact}`,
      );
    }

    const primaryCategory = artifactCategories.get(actual.primaryArtifact);
    const secondaryCategory = artifactCategories.get(actual.secondaryArtifact);
    if (!primaryCategory) {
      throw new Error(`Structure coverage row ${actual.reviewItem} references unknown primary artifact ${actual.primaryArtifact}`);
    }
    if (!secondaryCategory) {
      throw new Error(`Structure coverage row ${actual.reviewItem} references unknown secondary artifact ${actual.secondaryArtifact}`);
    }
    if (actual.reviewItem === "Functional detail") {
      if (primaryCategory !== "secondary-functional-detail" || secondaryCategory !== "primary-structure") {
        throw new Error(
          `Functional detail row must keep functional content secondary to structural evidence; got primary=${primaryCategory}, secondary=${secondaryCategory}`,
        );
      }
    } else if (primaryCategory !== "primary-structure") {
      throw new Error(
        `Structure coverage row ${actual.reviewItem} must use a primary-structure artifact first; got ${primaryCategory}`,
      );
    }
  }
  if (STRUCTURE_COVERAGE_MATRIX !== "docs/ui-and-layout/hub-design-board/STRUCTURE_COVERAGE_MATRIX.md") {
    throw new Error(`Unexpected structure coverage matrix path: ${STRUCTURE_COVERAGE_MATRIX}`);
  }
}

function parseCoverageRows(content) {
  const rows = [];
  for (const line of content.split(/\r?\n/)) {
    if (!line.startsWith("| ")) {
      continue;
    }
    if (line.includes("---") || line.startsWith("| Review item ")) {
      continue;
    }
    const match = line.match(/^\| ([^|]+) \| `([^`]+)` \| `([^`]+)` \| ([^|]+) \|$/);
    if (!match) {
      continue;
    }
    rows.push({
      reviewItem: match[1].trim(),
      primaryArtifact: match[2].trim(),
      secondaryArtifact: match[3].trim(),
      inspectFor: match[4].trim(),
    });
  }
  return rows;
}

function validateStructureGeometryEvidenceDocument() {
  if (!existsSync(structureGeometryPath)) {
    throw new Error(`Missing structure geometry evidence: ${structureGeometryPath}`);
  }

  const content = readFileSync(structureGeometryPath, "utf8");
  for (const snippet of STRUCTURE_GEOMETRY_REQUIRED_TEXT) {
    if (!content.includes(snippet)) {
      throw new Error(`Structure geometry evidence missing required text: ${snippet}`);
    }
  }
  if (STRUCTURE_GEOMETRY_EVIDENCE !== "docs/ui-and-layout/hub-design-board/STRUCTURE_GEOMETRY_EVIDENCE.md") {
    throw new Error(`Unexpected structure geometry evidence path: ${STRUCTURE_GEOMETRY_EVIDENCE}`);
  }
}

function validateStructureToReferenceMap() {
  if (!existsSync(structureToReferenceMapPath)) {
    throw new Error(`Missing structure to reference map: ${structureToReferenceMapPath}`);
  }

  const content = readFileSync(structureToReferenceMapPath, "utf8");
  for (const snippet of STRUCTURE_TO_REFERENCE_REQUIRED_TEXT) {
    if (!content.includes(snippet)) {
      throw new Error(`Structure to reference map missing required text: ${snippet}`);
    }
  }
  for (const { output } of DESIGN_BOARD_LIST) {
    if (!content.includes(output)) {
      throw new Error(`Structure to reference map must reference design-board artifact ${output}`);
    }
  }

  const knownFinalOutputs = new Set([DASHBOARD_CAPTURE_NAME, ...EXPORTS_LIST.map(([, output]) => output)]);
  const referencedFinalOutputs = [...content.matchAll(/`(hub-[^`]+\.png)`/g)]
    .map((match) => match[1])
    .filter((output) => !DESIGN_BOARD_LIST.some((board) => board.output === output));
  for (const output of referencedFinalOutputs) {
    if (!knownFinalOutputs.has(output)) {
      throw new Error(`Structure to reference map references unknown final artifact ${output}`);
    }
  }
  if (STRUCTURE_TO_REFERENCE_MAP !== "docs/ui-and-layout/hub-design-board/STRUCTURE_TO_REFERENCE_MAP.md") {
    throw new Error(`Unexpected structure to reference map path: ${STRUCTURE_TO_REFERENCE_MAP}`);
  }
}

function validateReferenceAlignmentMatrix() {
  if (!existsSync(referenceAlignmentMatrixPath)) {
    throw new Error(`Missing reference alignment matrix: ${referenceAlignmentMatrixPath}`);
  }

  const content = readFileSync(referenceAlignmentMatrixPath, "utf8");
  for (const snippet of REFERENCE_ALIGNMENT_MATRIX_REQUIRED_TEXT) {
    if (!content.includes(snippet)) {
      throw new Error(`Reference alignment matrix missing required text: ${snippet}`);
    }
  }

  const rows = parseReferenceAlignmentRows(content);
  if (rows.length !== REFERENCE_ALIGNMENT_CHECKS.length) {
    throw new Error(`Reference alignment matrix has ${rows.length} rows; expected ${REFERENCE_ALIGNMENT_CHECKS.length}`);
  }
  for (let index = 0; index < REFERENCE_ALIGNMENT_CHECKS.length; index += 1) {
    const expected = REFERENCE_ALIGNMENT_CHECKS[index];
    const actual = rows[index];
    for (const key of ["id", "target_region", "primary_artifact", "review_focus", "acceptance_rule"]) {
      if (actual[key] !== expected[key]) {
        throw new Error(`Reference alignment matrix row ${index + 1} ${key} is ${actual[key]}; expected ${expected[key]}`);
      }
    }
  }
  if (REFERENCE_ALIGNMENT_MATRIX !== "docs/ui-and-layout/hub-design-board/REFERENCE_ALIGNMENT_MATRIX.md") {
    throw new Error(`Unexpected reference alignment matrix path: ${REFERENCE_ALIGNMENT_MATRIX}`);
  }
}

function parseReferenceAlignmentRows(content) {
  const rows = [];
  for (const line of content.split(/\r?\n/)) {
    if (!line.startsWith("| ")) {
      continue;
    }
    if (line.includes("---") || line.startsWith("| Check ")) {
      continue;
    }
    const match = line.match(/^\| `([^`]+)` \| ([^|]+) \| `([^`]+)` \| ([^|]+) \| ([^|]+) \|$/);
    if (!match) {
      continue;
    }
    rows.push({
      id: match[1].trim(),
      target_region: normalizeDocCell(match[2]),
      primary_artifact: match[3].trim(),
      review_focus: normalizeDocCell(match[4]),
      acceptance_rule: normalizeDocCell(match[5]),
    });
  }
  return rows;
}

function normalizeDocCell(value) {
  return value.replace(/`/g, "").trim();
}

function validateStructureSignoffChecklist() {
  if (!existsSync(structureSignoffChecklistPath)) {
    throw new Error(`Missing structure signoff checklist: ${structureSignoffChecklistPath}`);
  }

  const content = readFileSync(structureSignoffChecklistPath, "utf8");
  for (const snippet of STRUCTURE_SIGNOFF_CHECKLIST_REQUIRED_TEXT) {
    if (!content.includes(snippet)) {
      throw new Error(`Structure signoff checklist missing required text: ${snippet}`);
    }
  }

  const rows = parseStructureSignoffRows(content);
  if (rows.length !== REFERENCE_ALIGNMENT_CHECKS.length) {
    throw new Error(`Structure signoff checklist has ${rows.length} rows; expected ${REFERENCE_ALIGNMENT_CHECKS.length}`);
  }
  for (let index = 0; index < REFERENCE_ALIGNMENT_CHECKS.length; index += 1) {
    const expected = REFERENCE_ALIGNMENT_CHECKS[index];
    const actual = rows[index];
    if (
      actual.id !== expected.id ||
      actual.primary_artifact !== expected.primary_artifact ||
      actual.target_region !== expected.target_region ||
      actual.required_decision !== expected.acceptance_rule
    ) {
      throw new Error(
        `Structure signoff row ${index + 1} is ${actual.id}/${actual.primary_artifact}/${actual.target_region}/${actual.required_decision}; expected ${expected.id}/${expected.primary_artifact}/${expected.target_region}/${expected.acceptance_rule}`,
      );
    }
    if (actual.manual_status !== "pending") {
      throw new Error(`Structure signoff row ${index + 1} status is ${actual.manual_status}; expected pending`);
    }
  }
  if (STRUCTURE_SIGNOFF_CHECKLIST !== "docs/ui-and-layout/hub-design-board/STRUCTURE_SIGNOFF_CHECKLIST.md") {
    throw new Error(`Unexpected structure signoff checklist path: ${STRUCTURE_SIGNOFF_CHECKLIST}`);
  }
}

function parseStructureSignoffRows(content) {
  const rows = [];
  for (const line of content.split(/\r?\n/)) {
    if (!line.startsWith("| ")) {
      continue;
    }
    if (line.includes("---") || line.startsWith("| Check ")) {
      continue;
    }
    const match = line.match(/^\| `([^`]+)` \| `([^`]+)` \| ([^|]+) \| ([^|]+) \| ([^|]+) \|$/);
    if (!match) {
      continue;
    }
    rows.push({
      id: match[1].trim(),
      primary_artifact: match[2].trim(),
      target_region: normalizeDocCell(match[3]),
      required_decision: normalizeDocCell(match[4]),
      manual_status: normalizeDocCell(match[5]),
    });
  }
  return rows;
}

function validateReviewIndex() {
  if (!existsSync(reviewIndexPath)) {
    throw new Error(`Missing design board review index: ${reviewIndexPath}`);
  }

  const content = readFileSync(reviewIndexPath, "utf8");
  for (const snippet of DESIGN_BOARD_REVIEW_INDEX_REQUIRED_TEXT) {
    if (!content.includes(snippet)) {
      throw new Error(`Design board review index missing required text: ${snippet}`);
    }
  }
  for (const { output } of DESIGN_BOARD_LIST) {
    const expectedPath = `docs/ui-and-layout/${output}`;
    if (!content.includes(expectedPath)) {
      throw new Error(`Design board review index must reference ${expectedPath}`);
    }
  }
  for (const supportPath of [
    DESIGN_BOARD_MANIFEST,
    DESIGN_BOARD_MANIFEST_SCHEMA,
    STRUCTURE_REVIEW_CHECKLIST,
    STRUCTURE_COVERAGE_MATRIX,
    STRUCTURE_GEOMETRY_EVIDENCE,
    STRUCTURE_GEOMETRY_BASELINE,
    STRUCTURE_RESPONSIVE_BASELINE,
    STRUCTURE_REVIEW_ROUTE_BASELINE,
    STRUCTURE_OVERLAY_BASELINE,
    STRUCTURE_REFERENCE_ROUTE_BASELINE,
    STRUCTURE_REVIEW_GUIDE,
    STRUCTURE_TO_REFERENCE_MAP,
    REFERENCE_ALIGNMENT_MATRIX,
    STRUCTURE_SIGNOFF_CHECKLIST,
    STRUCTURE_DECISION_LOG,
    STRUCTURE_REVIEW_STATUS,
    STRUCTURE_ACCEPTANCE_RECORD,
    STRUCTURE_REVIEW_PACKET,
    STRUCTURE_REVIEW_PACKET_SCHEMA,
    DESIGN_BOARD_EXPORT_METADATA,
  ]) {
    const basename = supportPath.split("/").at(-1);
    if (!content.includes(basename)) {
      throw new Error(`Design board review index must reference ${basename}`);
    }
  }
  if (DESIGN_BOARD_REVIEW_INDEX !== "docs/ui-and-layout/hub-design-board/REVIEW_INDEX.md") {
    throw new Error(`Unexpected design board review index path: ${DESIGN_BOARD_REVIEW_INDEX}`);
  }
}

function validateStructureReviewGuide() {
  if (!existsSync(structureReviewGuidePath)) {
    throw new Error(`Missing structure review guide: ${structureReviewGuidePath}`);
  }

  const content = readFileSync(structureReviewGuidePath, "utf8");
  for (const snippet of STRUCTURE_REVIEW_GUIDE_REQUIRED_TEXT) {
    if (!content.includes(snippet)) {
      throw new Error(`Structure review guide missing required text: ${snippet}`);
    }
  }
  for (const { output } of DESIGN_BOARD_LIST) {
    if (!content.includes(output)) {
      throw new Error(`Structure review guide must reference ${output}`);
    }
  }
  for (const supportPath of [
    STRUCTURE_COVERAGE_MATRIX,
    STRUCTURE_GEOMETRY_EVIDENCE,
    STRUCTURE_GEOMETRY_BASELINE,
    STRUCTURE_RESPONSIVE_BASELINE,
    STRUCTURE_REVIEW_ROUTE_BASELINE,
    STRUCTURE_OVERLAY_BASELINE,
    STRUCTURE_REFERENCE_ROUTE_BASELINE,
  ]) {
    const basename = supportPath.split("/").at(-1);
    if (!content.includes(basename)) {
      throw new Error(`Structure review guide must reference ${basename}`);
    }
  }
  if (STRUCTURE_REVIEW_GUIDE !== "docs/ui-and-layout/hub-design-board/STRUCTURE_REVIEW_GUIDE.md") {
    throw new Error(`Unexpected structure review guide path: ${STRUCTURE_REVIEW_GUIDE}`);
  }
}

function validateStructureReviewStatus() {
  if (!existsSync(structureReviewStatusPath)) {
    throw new Error(`Missing structure review status: ${structureReviewStatusPath}`);
  }

  const content = readFileSync(structureReviewStatusPath, "utf8");
  for (const snippet of STRUCTURE_REVIEW_STATUS_REQUIRED_TEXT) {
    if (!content.includes(snippet)) {
      throw new Error(`Structure review status missing required text: ${snippet}`);
    }
  }
  for (const { output } of DESIGN_BOARD_LIST) {
    if (!content.includes(output)) {
      throw new Error(`Structure review status must reference ${output}`);
    }
  }
  for (const supportPath of [
    DESIGN_BOARD_MANIFEST,
    DESIGN_BOARD_MANIFEST_SCHEMA,
    STRUCTURE_COVERAGE_MATRIX,
    STRUCTURE_GEOMETRY_EVIDENCE,
    STRUCTURE_GEOMETRY_BASELINE,
    STRUCTURE_RESPONSIVE_BASELINE,
    STRUCTURE_REVIEW_ROUTE_BASELINE,
    STRUCTURE_OVERLAY_BASELINE,
    STRUCTURE_REFERENCE_ROUTE_BASELINE,
    STRUCTURE_TO_REFERENCE_MAP,
    REFERENCE_ALIGNMENT_MATRIX,
    STRUCTURE_SIGNOFF_CHECKLIST,
    STRUCTURE_DECISION_LOG,
    STRUCTURE_REVIEW_PACKET,
    STRUCTURE_REVIEW_PACKET_SCHEMA,
    DESIGN_BOARD_EXPORT_METADATA,
  ]) {
    const basename = supportPath.split("/").at(-1);
    if (!content.includes(basename)) {
      throw new Error(`Structure review status must reference ${basename}`);
    }
  }
  if (STRUCTURE_REVIEW_STATUS !== "docs/ui-and-layout/hub-design-board/STRUCTURE_REVIEW_STATUS.md") {
    throw new Error(`Unexpected structure review status path: ${STRUCTURE_REVIEW_STATUS}`);
  }
}

function validateStructureAcceptanceRecord() {
  if (!existsSync(structureAcceptanceRecordPath)) {
    throw new Error(`Missing structure acceptance record: ${structureAcceptanceRecordPath}`);
  }

  const content = readFileSync(structureAcceptanceRecordPath, "utf8");
  for (const snippet of STRUCTURE_ACCEPTANCE_RECORD_REQUIRED_TEXT) {
    if (!content.includes(snippet)) {
      throw new Error(`Structure acceptance record missing required text: ${snippet}`);
    }
  }
  for (const { output } of DESIGN_BOARD_LIST) {
    if (!content.includes(output)) {
      throw new Error(`Structure acceptance record must reference ${output}`);
    }
  }
  for (const supportPath of [
    DESIGN_BOARD_MANIFEST,
    DESIGN_BOARD_MANIFEST_SCHEMA,
    STRUCTURE_REVIEW_STATUS,
    STRUCTURE_REVIEW_PACKET,
    STRUCTURE_REVIEW_PACKET_SCHEMA,
    STRUCTURE_COVERAGE_MATRIX,
    STRUCTURE_GEOMETRY_EVIDENCE,
    STRUCTURE_GEOMETRY_BASELINE,
    STRUCTURE_RESPONSIVE_BASELINE,
    STRUCTURE_REVIEW_ROUTE_BASELINE,
    STRUCTURE_OVERLAY_BASELINE,
    STRUCTURE_REFERENCE_ROUTE_BASELINE,
    REFERENCE_ALIGNMENT_MATRIX,
    STRUCTURE_SIGNOFF_CHECKLIST,
    STRUCTURE_DECISION_LOG,
    DESIGN_BOARD_EXPORT_METADATA,
  ]) {
    const basename = supportPath.split("/").at(-1);
    if (!content.includes(basename)) {
      throw new Error(`Structure acceptance record must reference ${basename}`);
    }
  }
  if (STRUCTURE_ACCEPTANCE_RECORD !== "docs/ui-and-layout/hub-design-board/STRUCTURE_ACCEPTANCE_RECORD.md") {
    throw new Error(`Unexpected structure acceptance record path: ${STRUCTURE_ACCEPTANCE_RECORD}`);
  }
}

function validateStructureReviewPacket() {
  if (!existsSync(structureReviewPacketPath)) {
    throw new Error(`Missing structure review packet: ${structureReviewPacketPath}`);
  }
  if (STRUCTURE_REVIEW_PACKET !== "docs/ui-and-layout/hub-design-board/structure-review-packet.json") {
    throw new Error(`Unexpected structure review packet path: ${STRUCTURE_REVIEW_PACKET}`);
  }

  const packet = JSON.parse(readFileSync(structureReviewPacketPath, "utf8"));
  const schema = JSON.parse(readFileSync(structureReviewPacketSchemaPath, "utf8"));
  validateJsonSchemaSubset(packet, schema, "$");
  if (packet.schema !== STRUCTURE_REVIEW_PACKET_SCHEMA) {
    throw new Error(`Structure review packet schema is ${packet.schema}; expected ${STRUCTURE_REVIEW_PACKET_SCHEMA}`);
  }
  if (packet.source !== DESIGN_BOARD_SOURCE) {
    throw new Error(`Structure review packet source is ${packet.source}; expected ${DESIGN_BOARD_SOURCE}`);
  }
  if (packet.canvas?.width !== CANVAS_WIDTH || packet.canvas?.height !== CANVAS_HEIGHT) {
    throw new Error(
      `Structure review packet canvas is ${packet.canvas?.width}x${packet.canvas?.height}; expected ${CANVAS_WIDTH}x${CANVAS_HEIGHT}`,
    );
  }
  if (packet.review_priority !== "overall-interaction-structure-layout") {
    throw new Error(`Structure review packet review_priority is ${packet.review_priority}`);
  }
  validateManualSignoff("Structure review packet manual_signoff", packet.manual_signoff);
  validateReferenceTarget("Structure review packet reference_target", packet.reference_target);
  validateReferenceAlignmentChecks("Structure review packet reference_alignment_checks", packet.reference_alignment_checks);
  if (packet.reference_alignment_matrix !== REFERENCE_ALIGNMENT_MATRIX) {
    throw new Error(
      `Structure review packet reference_alignment_matrix is ${packet.reference_alignment_matrix}; expected ${REFERENCE_ALIGNMENT_MATRIX}`,
    );
  }
  if (packet.structure_geometry_baseline !== STRUCTURE_GEOMETRY_BASELINE) {
    throw new Error(
      `Structure review packet structure_geometry_baseline is ${packet.structure_geometry_baseline}; expected ${STRUCTURE_GEOMETRY_BASELINE}`,
    );
  }
  if (packet.structure_responsive_baseline !== STRUCTURE_RESPONSIVE_BASELINE) {
    throw new Error(
      `Structure review packet structure_responsive_baseline is ${packet.structure_responsive_baseline}; expected ${STRUCTURE_RESPONSIVE_BASELINE}`,
    );
  }
  if (packet.structure_review_route_baseline !== STRUCTURE_REVIEW_ROUTE_BASELINE) {
    throw new Error(
      `Structure review packet structure_review_route_baseline is ${packet.structure_review_route_baseline}; expected ${STRUCTURE_REVIEW_ROUTE_BASELINE}`,
    );
  }
  if (packet.structure_overlay_baseline !== STRUCTURE_OVERLAY_BASELINE) {
    throw new Error(
      `Structure review packet structure_overlay_baseline is ${packet.structure_overlay_baseline}; expected ${STRUCTURE_OVERLAY_BASELINE}`,
    );
  }
  if (packet.structure_reference_route_baseline !== STRUCTURE_REFERENCE_ROUTE_BASELINE) {
    throw new Error(
      `Structure review packet structure_reference_route_baseline is ${packet.structure_reference_route_baseline}; expected ${STRUCTURE_REFERENCE_ROUTE_BASELINE}`,
    );
  }
  if (packet.structure_signoff_checklist !== STRUCTURE_SIGNOFF_CHECKLIST) {
    throw new Error(
      `Structure review packet structure_signoff_checklist is ${packet.structure_signoff_checklist}; expected ${STRUCTURE_SIGNOFF_CHECKLIST}`,
    );
  }
  if (packet.structure_decision_log !== STRUCTURE_DECISION_LOG) {
    throw new Error(
      `Structure review packet structure_decision_log is ${packet.structure_decision_log}; expected ${STRUCTURE_DECISION_LOG}`,
    );
  }

  if (!Array.isArray(packet.artifacts) || packet.artifacts.length !== DESIGN_BOARD_LIST.length) {
    throw new Error(`Structure review packet must list ${DESIGN_BOARD_LIST.length} artifacts`);
  }
  for (let index = 0; index < DESIGN_BOARD_LIST.length; index += 1) {
    const expected = DESIGN_BOARD_LIST[index];
    const actual = packet.artifacts[index];
    const expectedPath = `docs/ui-and-layout/${expected.output}`;
    if (actual.id !== expected.id || actual.path !== expectedPath || actual.category !== expected.category) {
      throw new Error(
        `Structure review packet artifact ${index + 1} is ${actual.id}/${actual.path}/${actual.category}; expected ${expected.id}/${expectedPath}/${expected.category}`,
      );
    }
  }
  validateStructureReviewPacketReviewSequence(packet.review_sequence);
  validateManualReviewItems("Structure review packet manual_review_items", packet.manual_review_items);

  validateStringArray("Structure review packet support_documents", packet.support_documents, structureReviewPacketSupportDocuments());
  validateRepoRelativeFilesExist("Structure review packet support_documents", packet.support_documents);
  validateStringArray("Structure review packet validation_commands", packet.validation_commands, STRUCTURE_REVIEW_PACKET_REQUIRED_COMMANDS);
  validateAcceptanceBoundary("Structure review packet acceptance_boundary", packet.acceptance_boundary);
}

function manifestRequiredFields() {
  return [
    "name",
    "schema",
    "source",
    "canvas",
    "review_priority",
    "manual_signoff",
    "reference_target",
    "reference_alignment_checks",
    "structure_review_checklist",
    "structure_coverage_matrix",
    "structure_geometry_evidence",
    "structure_geometry_baseline",
    "structure_responsive_baseline",
    "structure_review_route_baseline",
    "structure_overlay_baseline",
    "structure_reference_route_baseline",
    "structure_review_guide",
    "review_index",
    "structure_to_reference_map",
    "reference_alignment_matrix",
    "structure_signoff_checklist",
    "structure_decision_log",
    "structure_review_status",
    "structure_acceptance_record",
    "structure_review_packet",
    "structure_review_packet_schema",
    "export_metadata",
    "support_documents",
    "validation_commands",
    "acceptance_boundary",
    "artifacts",
    "review_sequence",
    "manual_review_items",
  ];
}

function manifestSupportDocuments() {
  return [
    DESIGN_BOARD_REVIEW_INDEX,
    DESIGN_BOARD_MANIFEST_SCHEMA,
    STRUCTURE_REVIEW_CHECKLIST,
    STRUCTURE_COVERAGE_MATRIX,
    STRUCTURE_GEOMETRY_EVIDENCE,
    STRUCTURE_GEOMETRY_BASELINE,
    STRUCTURE_RESPONSIVE_BASELINE,
    STRUCTURE_REVIEW_ROUTE_BASELINE,
    STRUCTURE_OVERLAY_BASELINE,
    STRUCTURE_REFERENCE_ROUTE_BASELINE,
    STRUCTURE_REVIEW_GUIDE,
    STRUCTURE_TO_REFERENCE_MAP,
    REFERENCE_ALIGNMENT_MATRIX,
    STRUCTURE_SIGNOFF_CHECKLIST,
    STRUCTURE_DECISION_LOG,
    STRUCTURE_REVIEW_STATUS,
    STRUCTURE_ACCEPTANCE_RECORD,
    STRUCTURE_REVIEW_PACKET,
    STRUCTURE_REVIEW_PACKET_SCHEMA,
    DESIGN_BOARD_EXPORT_METADATA,
  ];
}

function structureReviewPacketSupportDocuments() {
  return [
    DESIGN_BOARD_REVIEW_INDEX,
    DESIGN_BOARD_MANIFEST,
    DESIGN_BOARD_MANIFEST_SCHEMA,
    STRUCTURE_REVIEW_CHECKLIST,
    STRUCTURE_COVERAGE_MATRIX,
    STRUCTURE_GEOMETRY_EVIDENCE,
    STRUCTURE_GEOMETRY_BASELINE,
    STRUCTURE_RESPONSIVE_BASELINE,
    STRUCTURE_REVIEW_ROUTE_BASELINE,
    STRUCTURE_OVERLAY_BASELINE,
    STRUCTURE_REFERENCE_ROUTE_BASELINE,
    STRUCTURE_REVIEW_GUIDE,
    STRUCTURE_TO_REFERENCE_MAP,
    REFERENCE_ALIGNMENT_MATRIX,
    STRUCTURE_SIGNOFF_CHECKLIST,
    STRUCTURE_DECISION_LOG,
    STRUCTURE_REVIEW_STATUS,
    STRUCTURE_ACCEPTANCE_RECORD,
    STRUCTURE_REVIEW_PACKET_SCHEMA,
    DESIGN_BOARD_EXPORT_METADATA,
  ];
}

function validateManualSignoff(label, actual) {
  if (actual?.status !== "pending") {
    throw new Error(`${label} must remain pending until user review; got ${actual?.status}`);
  }
  const primaryArtifact = "docs/ui-and-layout/hub-design-structure-layout.png";
  if (actual?.required_primary_artifact !== primaryArtifact) {
    throw new Error(`${label} primary artifact is ${actual?.required_primary_artifact}; expected ${primaryArtifact}`);
  }
}

function validateReferenceTarget(label, actual) {
  if (actual?.path !== HUB_REFERENCE_TARGET) {
    throw new Error(`${label} path is ${actual?.path}; expected ${HUB_REFERENCE_TARGET}`);
  }
  if (actual?.role !== "target-ui-reference") {
    throw new Error(`${label} role is ${actual?.role}; expected target-ui-reference`);
  }
  if (actual?.canvas?.width !== CANVAS_WIDTH || actual?.canvas?.height !== CANVAS_HEIGHT) {
    throw new Error(
      `${label} canvas is ${actual?.canvas?.width}x${actual?.canvas?.height}; expected ${CANVAS_WIDTH}x${CANVAS_HEIGHT}`,
    );
  }
  validatePng(resolve(here, "../../..", HUB_REFERENCE_TARGET));
}

function validateReferenceTargetSchema(label, schema) {
  const properties = schema?.properties;
  if (schema?.type !== "object" || schema?.additionalProperties !== false) {
    throw new Error(`${label} must define a closed object`);
  }
  if (properties?.path?.const !== HUB_REFERENCE_TARGET) {
    throw new Error(`${label} must pin ${HUB_REFERENCE_TARGET}`);
  }
  if (properties?.role?.const !== "target-ui-reference") {
    throw new Error(`${label} must pin target-ui-reference role`);
  }
  if (
    properties?.canvas?.properties?.width?.const !== CANVAS_WIDTH ||
    properties?.canvas?.properties?.height?.const !== CANVAS_HEIGHT
  ) {
    throw new Error(`${label} must pin the ${CANVAS_WIDTH}x${CANVAS_HEIGHT} canvas`);
  }
}

function validateReferenceAlignmentChecks(label, actual) {
  if (!Array.isArray(actual) || actual.length !== REFERENCE_ALIGNMENT_CHECKS.length) {
    throw new Error(`${label} must list ${REFERENCE_ALIGNMENT_CHECKS.length} entries`);
  }
  for (let index = 0; index < REFERENCE_ALIGNMENT_CHECKS.length; index += 1) {
    const expected = REFERENCE_ALIGNMENT_CHECKS[index];
    const row = actual[index];
    for (const key of ["id", "target_region", "primary_artifact", "review_focus", "acceptance_rule"]) {
      if (row?.[key] !== expected[key]) {
        throw new Error(`${label} ${index + 1} ${key} is ${row?.[key]}; expected ${expected[key]}`);
      }
    }
  }
}

function validateAcceptanceBoundary(label, actual) {
  validateStringArray(label, actual, [
    "design-board-and-web-reference-only",
    "manual-signoff-required",
    "runtime-behavior-out-of-scope",
  ]);
}

function validateStructureReviewPacketSchema() {
  if (!existsSync(structureReviewPacketSchemaPath)) {
    throw new Error(`Missing structure review packet schema: ${structureReviewPacketSchemaPath}`);
  }
  if (STRUCTURE_REVIEW_PACKET_SCHEMA !== "docs/ui-and-layout/hub-design-board/structure-review-packet.schema.json") {
    throw new Error(`Unexpected structure review packet schema path: ${STRUCTURE_REVIEW_PACKET_SCHEMA}`);
  }

  const schema = JSON.parse(readFileSync(structureReviewPacketSchemaPath, "utf8"));
  if (schema.$schema !== "https://json-schema.org/draft/2020-12/schema") {
    throw new Error(`Structure review packet schema draft is ${schema.$schema}`);
  }
  if (schema.$id !== STRUCTURE_REVIEW_PACKET_SCHEMA) {
    throw new Error(`Structure review packet schema id is ${schema.$id}; expected ${STRUCTURE_REVIEW_PACKET_SCHEMA}`);
  }
  if (schema.type !== "object" || schema.additionalProperties !== false) {
    throw new Error("Structure review packet schema must define a closed object");
  }
  validateStringArray("Structure review packet schema required fields", schema.required, STRUCTURE_REVIEW_PACKET_REQUIRED_FIELDS);
  if (schema.properties?.schema?.const !== STRUCTURE_REVIEW_PACKET_SCHEMA) {
    throw new Error("Structure review packet schema must pin the packet schema path");
  }
  if (schema.properties?.source?.const !== DESIGN_BOARD_SOURCE) {
    throw new Error("Structure review packet schema must pin the design-board source");
  }
  if (
    schema.properties?.canvas?.properties?.width?.const !== CANVAS_WIDTH ||
    schema.properties?.canvas?.properties?.height?.const !== CANVAS_HEIGHT
  ) {
    throw new Error("Structure review packet schema must pin the 1568x1003 canvas");
  }
  if (schema.properties?.review_priority?.const !== "overall-interaction-structure-layout") {
    throw new Error("Structure review packet schema must pin structure-first review priority");
  }
  if (schema.properties?.manual_signoff?.properties?.status?.const !== "pending") {
    throw new Error("Structure review packet schema must keep manual signoff pending");
  }
  if (
    schema.properties?.manual_signoff?.properties?.required_primary_artifact?.const !==
    "docs/ui-and-layout/hub-design-structure-layout.png"
  ) {
    throw new Error("Structure review packet schema must pin the primary signoff artifact");
  }
  validateReferenceTargetSchema("Structure review packet schema reference_target", schema.properties?.reference_target);
  if (schema.properties?.reference_alignment_matrix?.const !== REFERENCE_ALIGNMENT_MATRIX) {
    throw new Error("Structure review packet schema must pin the reference alignment matrix");
  }
  if (schema.properties?.structure_geometry_baseline?.const !== STRUCTURE_GEOMETRY_BASELINE) {
    throw new Error("Structure review packet schema must pin the structure geometry baseline");
  }
  if (schema.properties?.structure_responsive_baseline?.const !== STRUCTURE_RESPONSIVE_BASELINE) {
    throw new Error("Structure review packet schema must pin the structure responsive baseline");
  }
  if (schema.properties?.structure_review_route_baseline?.const !== STRUCTURE_REVIEW_ROUTE_BASELINE) {
    throw new Error("Structure review packet schema must pin the structure review route baseline");
  }
  if (schema.properties?.structure_overlay_baseline?.const !== STRUCTURE_OVERLAY_BASELINE) {
    throw new Error("Structure review packet schema must pin the structure overlay baseline");
  }
  if (schema.properties?.structure_reference_route_baseline?.const !== STRUCTURE_REFERENCE_ROUTE_BASELINE) {
    throw new Error("Structure review packet schema must pin the structure reference route baseline");
  }
  if (schema.properties?.structure_signoff_checklist?.const !== STRUCTURE_SIGNOFF_CHECKLIST) {
    throw new Error("Structure review packet schema must pin the structure signoff checklist");
  }
  if (schema.properties?.structure_decision_log?.const !== STRUCTURE_DECISION_LOG) {
    throw new Error("Structure review packet schema must pin the structure decision log");
  }
  if (
    schema.properties?.reference_alignment_checks?.minItems !== REFERENCE_ALIGNMENT_CHECKS.length ||
    schema.properties?.reference_alignment_checks?.maxItems !== REFERENCE_ALIGNMENT_CHECKS.length
  ) {
    throw new Error("Structure review packet schema must pin reference alignment check count");
  }
  if (schema.properties?.artifacts?.minItems !== DESIGN_BOARD_LIST.length || schema.properties?.artifacts?.maxItems !== DESIGN_BOARD_LIST.length) {
    throw new Error("Structure review packet schema must pin the artifact count");
  }
  if (
    schema.properties?.review_sequence?.minItems !== STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length ||
    schema.properties?.review_sequence?.maxItems !== STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length
  ) {
    throw new Error("Structure review packet schema must pin the review sequence count");
  }
  const sequenceProperties = schema.properties?.review_sequence?.items?.properties;
  if (!Array.isArray(sequenceProperties?.step?.enum) || sequenceProperties.step.enum.length !== STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length) {
    throw new Error("Structure review packet schema must enumerate review sequence steps");
  }
  if (!Array.isArray(sequenceProperties?.artifact_id?.enum) || sequenceProperties.artifact_id.enum.length !== DESIGN_BOARD_LIST.length) {
    throw new Error("Structure review packet schema must enumerate review sequence artifact ids");
  }
  if (!Array.isArray(sequenceProperties?.focus?.enum) || sequenceProperties.focus.enum.length !== STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length) {
    throw new Error("Structure review packet schema must enumerate review sequence focus values");
  }
  if (
    schema.properties?.manual_review_items?.minItems !== STRUCTURE_COVERAGE_EXPECTED_ROWS.length ||
    schema.properties?.manual_review_items?.maxItems !== STRUCTURE_COVERAGE_EXPECTED_ROWS.length
  ) {
    throw new Error("Structure review packet schema must pin the manual review item count");
  }
  const manualReviewProperties = schema.properties?.manual_review_items?.items?.properties;
  if (!Array.isArray(manualReviewProperties?.item?.enum) || manualReviewProperties.item.enum.length !== STRUCTURE_COVERAGE_EXPECTED_ROWS.length) {
    throw new Error("Structure review packet schema must enumerate manual review items");
  }
  if (
    schema.properties?.support_documents?.minItems !== structureReviewPacketSupportDocuments().length ||
    schema.properties?.support_documents?.maxItems !== structureReviewPacketSupportDocuments().length
  ) {
    throw new Error("Structure review packet schema must include every support document, including manifest schema");
  }
  if (schema.properties?.validation_commands?.minItems !== STRUCTURE_REVIEW_PACKET_REQUIRED_COMMANDS.length) {
    throw new Error("Structure review packet schema must pin validation command count");
  }
  if (schema.properties?.acceptance_boundary?.minItems !== 3 || schema.properties?.acceptance_boundary?.maxItems !== 3) {
    throw new Error("Structure review packet schema must pin acceptance boundary count");
  }
}

function validateStructureReviewPacketReviewSequence(actual) {
  if (!Array.isArray(actual) || actual.length !== STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length) {
    throw new Error(`Structure review packet review_sequence must list ${STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length} entries`);
  }
  for (let index = 0; index < STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length; index += 1) {
    const expected = STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE[index];
    const row = actual[index];
    for (const key of ["step", "artifact_id", "focus", "decision_scope"]) {
      if (row?.[key] !== expected[key]) {
        throw new Error(
          `Structure review packet review_sequence ${index + 1} ${key} is ${row?.[key]}; expected ${expected[key]}`,
        );
      }
    }
  }
  if (actual[0].artifact_id !== "structure" || actual[2].artifact_id !== "details") {
    throw new Error("Structure review packet review_sequence must put overall structure first and functional detail last");
  }
}

function validateManualReviewItems(label, actual) {
  if (!Array.isArray(actual) || actual.length !== STRUCTURE_COVERAGE_EXPECTED_ROWS.length) {
    throw new Error(`${label} must list ${STRUCTURE_COVERAGE_EXPECTED_ROWS.length} entries`);
  }
  for (let index = 0; index < STRUCTURE_COVERAGE_EXPECTED_ROWS.length; index += 1) {
    const expected = STRUCTURE_COVERAGE_EXPECTED_ROWS[index];
    const row = actual[index];
    if (
      row?.item !== expected.reviewItem ||
      row?.primary_artifact !== expected.primaryArtifact ||
      row?.secondary_artifact !== expected.secondaryArtifact
    ) {
      throw new Error(
        `${label} ${index + 1} is ${row?.item}/${row?.primary_artifact}/${row?.secondary_artifact}; expected ${expected.reviewItem}/${expected.primaryArtifact}/${expected.secondaryArtifact}`,
      );
    }
  }
}

function validateManifestReviewSequence(actual) {
  if (!Array.isArray(actual) || actual.length !== STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length) {
    throw new Error(`Design board manifest review_sequence must list ${STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length} entries`);
  }
  for (let index = 0; index < STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE.length; index += 1) {
    const expected = STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE[index];
    const row = actual[index];
    if (row?.step !== expected.step || row?.artifact_id !== expected.artifact_id || row?.focus !== expected.focus) {
      throw new Error(
        `Design board manifest review_sequence ${index + 1} is ${row?.step}/${row?.artifact_id}/${row?.focus}; expected ${expected.step}/${expected.artifact_id}/${expected.focus}`,
      );
    }
  }
}

function validateJsonSchemaSubsetSelfTest() {
  const schema = {
    type: "object",
    additionalProperties: false,
    required: ["kind", "count", "mode", "items", "path"],
    properties: {
      kind: { type: "string", const: "packet" },
      count: { type: "number", const: 3 },
      mode: { type: "string", enum: ["pending", "accepted"] },
      items: {
        type: "array",
        minItems: 2,
        maxItems: 2,
        items: {
          type: "object",
          additionalProperties: false,
          required: ["id"],
          properties: {
            id: { type: "string", enum: ["structure", "flow"] },
          },
        },
      },
      path: { type: "string", pattern: "^docs/.+\\.json$" },
    },
  };
  const valid = {
    kind: "packet",
    count: 3,
    mode: "pending",
    items: [{ id: "structure" }, { id: "flow" }],
    path: "docs/ui-and-layout/hub-design-board/structure-review-packet.json",
  };

  validateJsonSchemaSubset(valid, schema, "$self.valid");

  expectJsonSchemaSubsetFailure("missing-required", {
    kind: "packet",
    count: 3,
    mode: "pending",
    items: [{ id: "structure" }, { id: "flow" }],
  }, schema);
  expectJsonSchemaSubsetFailure("unexpected-property", { ...valid, extra: true }, schema);
  expectJsonSchemaSubsetFailure("wrong-const", { ...valid, kind: "other" }, schema);
  expectJsonSchemaSubsetFailure("wrong-enum", { ...valid, mode: "draft" }, schema);
  expectJsonSchemaSubsetFailure("too-few-items", { ...valid, items: [{ id: "structure" }] }, schema);
  expectJsonSchemaSubsetFailure("too-many-items", {
    ...valid,
    items: [{ id: "structure" }, { id: "flow" }, { id: "structure" }],
  }, schema);
  expectJsonSchemaSubsetFailure("bad-nested-enum", { ...valid, items: [{ id: "structure" }, { id: "details" }] }, schema);
  expectJsonSchemaSubsetFailure("bad-pattern", { ...valid, path: "docs/ui-and-layout/hub-design-board/packet.txt" }, schema);
  expectJsonSchemaSubsetFailure("wrong-type", { ...valid, count: "3" }, schema);
}

function expectJsonSchemaSubsetFailure(label, value, schema) {
  let rejected = false;
  try {
    validateJsonSchemaSubset(value, schema, `$self.${label}`);
  } catch (_) {
    rejected = true;
  }
  if (!rejected) {
    throw new Error(`JSON schema subset self-test did not reject ${label}`);
  }
}

function validateJsonSchemaSubset(value, schema, path) {
  if (schema.const !== undefined && !jsonEqual(value, schema.const)) {
    throw new Error(`${path} must equal ${JSON.stringify(schema.const)}; got ${JSON.stringify(value)}`);
  }
  if (schema.enum && !schema.enum.some((item) => jsonEqual(value, item))) {
    throw new Error(`${path} must be one of ${schema.enum.map((item) => JSON.stringify(item)).join(", ")}; got ${JSON.stringify(value)}`);
  }

  if (schema.type) {
    validateJsonType(value, schema.type, path);
  }

  if (schema.type === "object") {
    const required = schema.required ?? [];
    for (const key of required) {
      if (!Object.hasOwn(value, key)) {
        throw new Error(`${path} is missing required field ${key}`);
      }
    }

    const properties = schema.properties ?? {};
    if (schema.additionalProperties === false) {
      for (const key of Object.keys(value)) {
        if (!Object.hasOwn(properties, key)) {
          throw new Error(`${path} has unexpected field ${key}`);
        }
      }
    }

    for (const [key, childSchema] of Object.entries(properties)) {
      if (Object.hasOwn(value, key)) {
        validateJsonSchemaSubset(value[key], childSchema, `${path}.${key}`);
      }
    }
  }

  if (schema.type === "array") {
    if (schema.minItems !== undefined && value.length < schema.minItems) {
      throw new Error(`${path} has ${value.length} items; expected at least ${schema.minItems}`);
    }
    if (schema.maxItems !== undefined && value.length > schema.maxItems) {
      throw new Error(`${path} has ${value.length} items; expected at most ${schema.maxItems}`);
    }
    if (schema.items) {
      for (let index = 0; index < value.length; index += 1) {
        validateJsonSchemaSubset(value[index], schema.items, `${path}[${index}]`);
      }
    }
  }

  if (schema.type === "string" && schema.pattern) {
    const regex = new RegExp(schema.pattern);
    if (!regex.test(value)) {
      throw new Error(`${path} must match ${schema.pattern}; got ${value}`);
    }
  }
}

function validateJsonType(value, type, path) {
  const actualType = Array.isArray(value) ? "array" : value === null ? "null" : typeof value;
  if (actualType !== type) {
    throw new Error(`${path} must be ${type}; got ${actualType}`);
  }
}

function jsonEqual(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function validateStringArray(label, actual, expected) {
  if (!Array.isArray(actual) || actual.length !== expected.length) {
    throw new Error(`${label} must list ${expected.length} entries`);
  }
  for (let index = 0; index < expected.length; index += 1) {
    if (actual[index] !== expected[index]) {
      throw new Error(`${label} entry ${index + 1} is ${actual[index]}; expected ${expected[index]}`);
    }
  }
}

function validateRepoRelativeFilesExist(label, paths) {
  for (const relativePath of paths) {
    const filePath = resolve(here, "../../..", relativePath);
    if (!existsSync(filePath)) {
      throw new Error(`${label} references missing file: ${relativePath}`);
    }
  }
}

function validateExportsIndex() {
  if (!existsSync(exportsIndex)) {
    throw new Error(`Missing design board export index: ${exportsIndex}`);
  }

  const content = readFileSync(exportsIndex, "utf8");
  if (!content.includes(`Source page: ${DESIGN_BOARD_SOURCE}`)) {
    throw new Error(`Design board export index must list source page ${DESIGN_BOARD_SOURCE}`);
  }
  for (const snippet of [
    "Review order:",
    DESIGN_BOARD_REVIEW_INDEX,
    "1. hub-design-structure-layout.png",
    "2. hub-design-structure-supplement.png",
    "3. hub-design-functional-details.png",
    "Review support:",
    DESIGN_BOARD_MANIFEST,
    DESIGN_BOARD_MANIFEST_SCHEMA,
    STRUCTURE_REVIEW_CHECKLIST,
    STRUCTURE_COVERAGE_MATRIX,
    STRUCTURE_GEOMETRY_EVIDENCE,
    STRUCTURE_GEOMETRY_BASELINE,
    STRUCTURE_RESPONSIVE_BASELINE,
    STRUCTURE_REVIEW_ROUTE_BASELINE,
    STRUCTURE_OVERLAY_BASELINE,
    STRUCTURE_REFERENCE_ROUTE_BASELINE,
    STRUCTURE_REVIEW_GUIDE,
    STRUCTURE_TO_REFERENCE_MAP,
    REFERENCE_ALIGNMENT_MATRIX,
    STRUCTURE_SIGNOFF_CHECKLIST,
    STRUCTURE_DECISION_LOG,
    STRUCTURE_REVIEW_STATUS,
    STRUCTURE_ACCEPTANCE_RECORD,
    STRUCTURE_REVIEW_PACKET,
    STRUCTURE_REVIEW_PACKET_SCHEMA,
    `Export metadata: ${DESIGN_BOARD_EXPORT_METADATA}`,
  ]) {
    if (!content.includes(snippet)) {
      throw new Error(`Design board EXPORTS.md missing review guidance: ${snippet}`);
    }
  }

  const rows = [...content.matchAll(/^- ([^:\r\n]+): ([^\r\n]+)$/gm)].map((match) => [match[2], match[1]]);
  if (rows.length !== DESIGN_BOARD_LIST.length) {
    throw new Error(`Design board EXPORTS.md lists ${rows.length} rows; expected ${DESIGN_BOARD_LIST.length}`);
  }

  for (let index = 0; index < DESIGN_BOARD_LIST.length; index += 1) {
    const expected = DESIGN_BOARD_LIST[index];
    const [actualId, actualOutput] = rows[index];
    if (actualId !== expected.id || actualOutput !== expected.output) {
      throw new Error(
        `Design board EXPORTS.md row ${index + 1} is ${actualOutput}: ${actualId}; expected ${expected.output}: ${expected.id}`,
      );
    }
  }
}

function validateExportMetadata() {
  if (!existsSync(exportMetadataPath)) {
    throw new Error(`Missing design board export metadata: ${exportMetadataPath}`);
  }
  if (DESIGN_BOARD_EXPORT_METADATA !== "docs/ui-and-layout/hub-design-board/export-metadata.json") {
    throw new Error(`Unexpected design board export metadata path: ${DESIGN_BOARD_EXPORT_METADATA}`);
  }

  const metadata = JSON.parse(readFileSync(exportMetadataPath, "utf8"));
  if (metadata.source !== DESIGN_BOARD_SOURCE) {
    throw new Error(`Design board export metadata source is ${metadata.source}; expected ${DESIGN_BOARD_SOURCE}`);
  }
  if (metadata.canvas?.width !== CANVAS_WIDTH || metadata.canvas?.height !== CANVAS_HEIGHT) {
    throw new Error(
      `Design board export metadata canvas is ${metadata.canvas?.width}x${metadata.canvas?.height}; expected ${CANVAS_WIDTH}x${CANVAS_HEIGHT}`,
    );
  }
  if (metadata.hash_algorithm !== "sha256") {
    throw new Error(`Design board export metadata hash_algorithm is ${metadata.hash_algorithm}; expected sha256`);
  }
  if (!Array.isArray(metadata.source_inputs)) {
    throw new Error("Design board export metadata source_inputs must be an array");
  }
  if (metadata.source_inputs.length !== DESIGN_BOARD_EXPORT_HASH_INPUTS.length) {
    throw new Error(
      `Design board export metadata lists ${metadata.source_inputs.length} source inputs; expected ${DESIGN_BOARD_EXPORT_HASH_INPUTS.length}`,
    );
  }

  for (let index = 0; index < DESIGN_BOARD_EXPORT_HASH_INPUTS.length; index += 1) {
    const expectedPath = DESIGN_BOARD_EXPORT_HASH_INPUTS[index];
    const actual = metadata.source_inputs[index];
    if (actual.path !== expectedPath) {
      throw new Error(`Design board export metadata source input ${index + 1} is ${actual.path}; expected ${expectedPath}`);
    }
    const actualHash = sha256File(resolve(here, "../../..", expectedPath));
    if (actual.sha256 !== actualHash) {
      throw new Error(`Design board export metadata source input hash mismatch for ${expectedPath}`);
    }
  }

  if (!Array.isArray(metadata.artifacts)) {
    throw new Error("Design board export metadata artifacts must be an array");
  }
  if (metadata.artifacts.length !== DESIGN_BOARD_LIST.length) {
    throw new Error(`Design board export metadata lists ${metadata.artifacts.length} artifacts; expected ${DESIGN_BOARD_LIST.length}`);
  }

  for (let index = 0; index < DESIGN_BOARD_LIST.length; index += 1) {
    const expected = DESIGN_BOARD_LIST[index];
    const actual = metadata.artifacts[index];
    if (actual.id !== expected.id || actual.output !== expected.output || actual.category !== expected.category) {
      throw new Error(
        `Design board export metadata artifact ${index + 1} is ${actual.id}/${actual.output}/${actual.category}; expected ${expected.id}/${expected.output}/${expected.category}`,
      );
    }
    const artifactPath = resolve(outputDir, expected.output);
    const info = statSync(artifactPath);
    if (actual.bytes !== info.size) {
      throw new Error(`Design board export metadata byte size mismatch for ${expected.output}: ${actual.bytes} vs ${info.size}`);
    }
    const actualHash = sha256File(artifactPath);
    if (actual.sha256 !== actualHash) {
      throw new Error(`Design board export metadata artifact hash mismatch for ${expected.output}`);
    }
  }
}

function sha256File(filePath) {
  return createHash("sha256").update(readFileSync(filePath)).digest("hex");
}

function validatePng(filePath) {
  if (!existsSync(filePath)) {
    throw new Error(`Missing design board PNG: ${filePath}`);
  }
  const size = statSync(filePath).size;
  if (size <= MIN_FILE_SIZE) {
    throw new Error(`Design board PNG is suspiciously small: ${filePath} (${size} bytes)`);
  }
  const image = decodePng(filePath);
  if (image.width !== CANVAS_WIDTH || image.height !== CANVAS_HEIGHT) {
    throw new Error(`${filePath} has size ${image.width}x${image.height}; expected ${CANVAS_WIDTH}x${CANVAS_HEIGHT}`);
  }
  const stats = computeImageStats(image);
  if (stats.dynamicRange < MIN_DYNAMIC_RANGE) {
    throw new Error(`${filePath} has low dynamic range ${stats.dynamicRange}; expected at least ${MIN_DYNAMIC_RANGE}`);
  }
  if (stats.averageLuma < MIN_AVERAGE_LUMA || stats.averageLuma > MAX_AVERAGE_LUMA) {
    throw new Error(
      `${filePath} average luma is ${stats.averageLuma.toFixed(2)}; expected ${MIN_AVERAGE_LUMA}..${MAX_AVERAGE_LUMA}`,
    );
  }
}

async function validateBrowserBoards() {
  const edge = [
    "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
    "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
  ].find((candidate) => existsSync(candidate));
  if (!edge) {
    throw new Error("Microsoft Edge executable not found.");
  }

  const port = Number.parseInt(
    process.env.ZIRCON_HUB_DESIGN_BOARD_CDP_PORT ?? String(10_433 + Math.floor(Math.random() * 500)),
    10,
  );
  const profile = mkdtempSync(join(tmpdir(), "zircon-hub-design-board-cdp-"));
  const baseUrl = pathToFileURL(sourcePath).href;
  const browser = spawn(
    edge,
    [
      "--headless=new",
      "--disable-gpu",
      "--hide-scrollbars",
      "--allow-file-access-from-files",
      `--remote-debugging-port=${port}`,
      `--user-data-dir=${profile}`,
      "about:blank",
    ],
    { stdio: "ignore" },
  );

  try {
    const list = await waitForJson(`http://127.0.0.1:${port}/json/list`);
    const target = list.find((item) => item.type === "page") ?? list[0];
    const cdp = await connect(target.webSocketDebuggerUrl);
    await cdp.send("Page.enable");
    await cdp.send("Runtime.enable");
    await cdp.send("Emulation.setDeviceMetricsOverride", {
      width: CANVAS_WIDTH,
      height: CANVAS_HEIGHT,
      deviceScaleFactor: 1,
      mobile: false,
    });

    for (const board of DESIGN_BOARD_LIST) {
      await cdp.send("Page.navigate", { url: `${baseUrl}?board=${encodeURIComponent(board.id)}` });
      await waitForBoardReady(cdp, board.id);
      const state = await evaluate(
        cdp,
        `(() => {
          const shell = document.querySelector(".design-shell");
          const box = shell.getBoundingClientRect();
          return JSON.stringify({
            text: document.body.innerText,
            width: shell.clientWidth,
            height: shell.clientHeight,
            boxWidth: box.width,
            boxHeight: box.height,
            scrollWidth: shell.scrollWidth,
            scrollHeight: shell.scrollHeight,
            bodyScrollWidth: document.body.scrollWidth,
            bodyScrollHeight: document.body.scrollHeight,
            activeBoard: document.documentElement.dataset.board
          });
        })()`,
      );
      const parsed = JSON.parse(state);
      if (parsed.activeBoard !== board.id) {
        throw new Error(`Design board ${board.id} did not activate; got ${parsed.activeBoard}`);
      }
      if (parsed.boxWidth !== CANVAS_WIDTH || parsed.boxHeight !== CANVAS_HEIGHT) {
        throw new Error(
          `Design board ${board.id} shell border box is ${parsed.boxWidth}x${parsed.boxHeight}; expected ${CANVAS_WIDTH}x${CANVAS_HEIGHT}`,
        );
      }
      if (parsed.scrollHeight > CANVAS_HEIGHT || parsed.bodyScrollHeight > CANVAS_HEIGHT) {
        throw new Error(
          `Design board ${board.id} overflows vertically: shell ${parsed.scrollHeight}, body ${parsed.bodyScrollHeight}, canvas ${CANVAS_HEIGHT}`,
        );
      }
      if (parsed.scrollWidth > CANVAS_WIDTH || parsed.bodyScrollWidth > CANVAS_WIDTH) {
        throw new Error(
          `Design board ${board.id} overflows horizontally: shell ${parsed.scrollWidth}, body ${parsed.bodyScrollWidth}, canvas ${CANVAS_WIDTH}`,
        );
      }
      for (const snippet of board.requiredText) {
        if (!parsed.text.includes(snippet)) {
          throw new Error(`Design board ${board.id} missing required text: ${snippet}`);
        }
      }
      if (board.id === "structure") {
        await validateStructureGeometry(cdp);
        await validateStructureLabelFit(cdp);
      }
    }

    cdp.close();
  } finally {
    browser.kill();
    await waitForExit(browser);
    for (let attempt = 0; attempt < 8; attempt += 1) {
      try {
        rmSync(profile, { recursive: true, force: true });
        return;
      } catch (_) {
        await delay(250);
      }
    }
  }
}

async function validateStructureGeometry(cdp) {
  const geometry = JSON.parse(
    await evaluate(
      cdp,
      `(() => {
        const frame = document.querySelector(".hub-frame");
        const frameRect = frame.getBoundingClientRect();
        const rectFor = (selector) => {
          const rect = document.querySelector(selector).getBoundingClientRect();
          return {
            left: Math.round(rect.left - frameRect.left),
            top: Math.round(rect.top - frameRect.top),
            right: Math.round(rect.right - frameRect.left),
            bottom: Math.round(rect.bottom - frameRect.top),
            width: Math.round(rect.width),
            height: Math.round(rect.height)
          };
        };
        return JSON.stringify({
          frame: { width: Math.round(frameRect.width), height: Math.round(frameRect.height) },
          scaleStripHeight: Math.round(document.querySelector(".frame-scale-strip").getBoundingClientRect().height),
          topbar: rectFor(".wf-topbar"),
          sidebar: rectFor(".wf-sidebar"),
          workspace: rectFor(".wf-workspace"),
          bottom: rectFor(".wf-bottom"),
          sourceOverlay: rectFor(".pop-source"),
          accountOverlay: rectFor(".pop-menu")
        });
      })()`,
    ),
  );

  assertBetween("structure frame width", geometry.frame.width, 1_095, 1_110);
  assertBetween("structure frame height", geometry.frame.height, 610, 614);
  assertBetween("structure scale strip height", geometry.scaleStripHeight, 30, 34);

  assertBetween("topbar top", geometry.topbar.top, 0, 2);
  assertBetween("topbar height", geometry.topbar.height, 57, 60);
  assertBetween("topbar width", geometry.topbar.width, geometry.frame.width - 3, geometry.frame.width + 1);

  assertBetween("sidebar left", geometry.sidebar.left, 0, 2);
  assertBetween("sidebar top", geometry.sidebar.top, geometry.topbar.bottom - 1, geometry.topbar.bottom + 1);
  assertBetween("sidebar width", geometry.sidebar.width, 177, 181);
  assertBetween("sidebar bottom", geometry.sidebar.bottom, geometry.frame.height - 2, geometry.frame.height + 1);

  assertBetween("workspace left", geometry.workspace.left, geometry.sidebar.right - 1, geometry.sidebar.right + 1);
  assertBetween("workspace top", geometry.workspace.top, geometry.topbar.bottom - 1, geometry.topbar.bottom + 1);
  assertBetween("workspace right", geometry.workspace.right, geometry.frame.width - 2, geometry.frame.width + 1);
  if (geometry.workspace.bottom > geometry.bottom.top + 1) {
    throw new Error(
      `Workspace must stay above Bottom strip; got workspace bottom ${geometry.workspace.bottom}, bottom top ${geometry.bottom.top}`,
    );
  }

  assertBetween("bottom left", geometry.bottom.left, geometry.sidebar.right - 1, geometry.sidebar.right + 1);
  assertBetween("bottom height", geometry.bottom.height, 92, 95);
  assertBetween("bottom bottom", geometry.bottom.bottom, geometry.frame.height - 2, geometry.frame.height + 1);

  if (geometry.sourceOverlay.left <= geometry.sidebar.right || geometry.accountOverlay.left <= geometry.sidebar.right) {
    throw new Error("Structure overlays must float over the workspace/header area, not inside the sidebar.");
  }
  if (geometry.sourceOverlay.top < geometry.topbar.bottom - 1 || geometry.accountOverlay.top < geometry.topbar.bottom - 1) {
    throw new Error("Structure overlays must appear below the top header boundary.");
  }
  if (geometry.sourceOverlay.bottom >= geometry.workspace.bottom || geometry.accountOverlay.bottom >= geometry.workspace.bottom) {
    throw new Error("Structure overlays must stay inside the workspace vertical span.");
  }
  validateStructureGeometryBaselineAgainstMeasured(structureGeometryBaselinePath, geometry);
  validateGeometryEvidenceNumbers(geometry);
}

async function validateStructureLabelFit(cdp) {
  const clipped = JSON.parse(
    await evaluate(
      cdp,
      `(() => {
        const selectors = [
          ".frame-scale-strip span",
          ".board-tabs a",
          ".wf-topbar strong",
          ".wf-sidebar strong",
          ".wf-bottom strong",
          ".route-flow span",
          ".check-panel li strong"
        ];
        const items = selectors.flatMap((selector) =>
          [...document.querySelectorAll(selector)].map((element) => ({
            selector,
            text: element.textContent.trim(),
            clientWidth: Math.ceil(element.clientWidth),
            scrollWidth: Math.ceil(element.scrollWidth),
            clientHeight: Math.ceil(element.clientHeight),
            scrollHeight: Math.ceil(element.scrollHeight)
          }))
        );
        return JSON.stringify(items.filter((item) => item.scrollWidth > item.clientWidth + 1 || item.scrollHeight > item.clientHeight + 1));
      })()`,
    ),
  );
  if (clipped.length > 0) {
    const summary = clipped.map((item) => `${item.selector} "${item.text}" ${item.scrollWidth}x${item.scrollHeight} > ${item.clientWidth}x${item.clientHeight}`).join("; ");
    throw new Error(`Structure board has clipped key labels: ${summary}`);
  }
}

function assertBetween(label, actual, min, max) {
  if (actual < min || actual > max) {
    throw new Error(`${label} is ${actual}; expected ${min}..${max}`);
  }
}

function validateGeometryEvidenceNumbers(geometry) {
  const content = readFileSync(structureGeometryPath, "utf8");
  const expectedRows = [
    ["Hub frame", 0, 0, geometry.frame.width, geometry.frame.height, geometry.frame.width, geometry.frame.height],
    ["Topbar", geometry.topbar.left, geometry.topbar.top, geometry.topbar.width, geometry.topbar.height, geometry.topbar.right, geometry.topbar.bottom],
    [
      "Sidebar",
      geometry.sidebar.left,
      geometry.sidebar.top,
      geometry.sidebar.width,
      geometry.sidebar.height,
      geometry.sidebar.right,
      geometry.sidebar.bottom,
    ],
    [
      "Workspace",
      geometry.workspace.left,
      geometry.workspace.top,
      geometry.workspace.width,
      geometry.workspace.height,
      geometry.workspace.right,
      geometry.workspace.bottom,
    ],
    ["Bottom strip", geometry.bottom.left, geometry.bottom.top, geometry.bottom.width, geometry.bottom.height, geometry.bottom.right, geometry.bottom.bottom],
    [
      "Source Engine overlay",
      geometry.sourceOverlay.left,
      geometry.sourceOverlay.top,
      geometry.sourceOverlay.width,
      geometry.sourceOverlay.height,
      geometry.sourceOverlay.right,
      geometry.sourceOverlay.bottom,
    ],
    [
      "Account overlay",
      geometry.accountOverlay.left,
      geometry.accountOverlay.top,
      geometry.accountOverlay.width,
      geometry.accountOverlay.height,
      geometry.accountOverlay.right,
      geometry.accountOverlay.bottom,
    ],
  ];

  for (const row of expectedRows) {
    const expected = `| ${row[0]} | ${row.slice(1).join(" | ")} |`;
    if (!content.includes(expected)) {
      throw new Error(`Structure geometry evidence must include measured row: ${expected}`);
    }
  }
  const dimensionStrip = `| Dimension strip | n/a | n/a | n/a | ${geometry.scaleStripHeight} | n/a | n/a |`;
  if (!content.includes(dimensionStrip)) {
    throw new Error(`Structure geometry evidence must include measured row: ${dimensionStrip}`);
  }
}

function delay(ms) {
  return new Promise((resolveDelay) => setTimeout(resolveDelay, ms));
}

async function waitForJson(url, attempts = 80) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return response.json();
      }
    } catch (_) {
      // The debug endpoint takes a moment to open.
    }
    await delay(100);
  }
  throw new Error(`Timed out waiting for ${url}.`);
}

function connect(wsUrl) {
  return new Promise((resolveConnect, rejectConnect) => {
    const ws = new WebSocket(wsUrl);
    const pending = new Map();
    let nextId = 1;

    ws.addEventListener("open", () => {
      resolveConnect({
        send(method, params = {}) {
          const id = nextId;
          nextId += 1;
          ws.send(JSON.stringify({ id, method, params }));
          return new Promise((resolveSend, rejectSend) => {
            pending.set(id, { method, resolveSend, rejectSend });
          });
        },
        close() {
          ws.close();
        },
      });
    });

    ws.addEventListener("message", (event) => {
      const message = JSON.parse(event.data);
      if (!message.id || !pending.has(message.id)) {
        return;
      }
      const request = pending.get(message.id);
      pending.delete(message.id);
      if (message.error) {
        request.rejectSend(new Error(`${request.method}: ${message.error.message}`));
      } else {
        request.resolveSend(message.result);
      }
    });

    ws.addEventListener("error", rejectConnect);
  });
}

async function waitForBoardReady(cdp, boardId, attempts = 80) {
  const expectedBoard = JSON.stringify(boardId);
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    const ready = await evaluate(
      cdp,
      `(() => {
        const shell = document.querySelector(".design-shell");
        return document.readyState !== "loading"
          && document.documentElement.dataset.board === ${expectedBoard}
          && Boolean(shell);
      })()`,
    );
    if (ready) {
      return;
    }
    await delay(100);
  }

  const diagnostics = await evaluate(
    cdp,
    `JSON.stringify({
      readyState: document.readyState,
      activeBoard: document.documentElement.dataset.board ?? "",
      hasShell: Boolean(document.querySelector(".design-shell")),
      bodyText: document.body?.innerText?.slice(0, 240) ?? ""
    })`,
  );
  throw new Error(`Timed out waiting for design board ${boardId} shell: ${diagnostics}`);
}

async function evaluate(cdp, expression) {
  const result = await cdp.send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (result.exceptionDetails) {
    throw new Error(
      result.exceptionDetails.exception?.description
        ?? result.exceptionDetails.text
        ?? "Runtime.evaluate exception.",
    );
  }
  return result.result.value;
}

async function waitForExit(child, timeoutMs = 3000) {
  if (child.exitCode !== null || child.signalCode !== null) {
    return;
  }
  await new Promise((resolveExit) => {
    const timer = setTimeout(resolveExit, timeoutMs);
    child.once("exit", () => {
      clearTimeout(timer);
      resolveExit();
    });
  });
}

function decodePng(filePath) {
  const bytes = readFileSync(filePath);
  const signature = "89504e470d0a1a0a";
  if (bytes.subarray(0, 8).toString("hex") !== signature) {
    throw new Error(`${filePath} is not a PNG file`);
  }

  let offset = 8;
  let header = null;
  const chunks = [];
  while (offset < bytes.length) {
    const length = bytes.readUInt32BE(offset);
    const type = bytes.subarray(offset + 4, offset + 8).toString("ascii");
    const data = bytes.subarray(offset + 8, offset + 8 + length);
    offset += 12 + length;
    if (type === "IHDR") {
      header = {
        width: data.readUInt32BE(0),
        height: data.readUInt32BE(4),
        bitDepth: data[8],
        colorType: data[9],
        interlace: data[12],
      };
    } else if (type === "IDAT") {
      chunks.push(data);
    } else if (type === "IEND") {
      break;
    }
  }

  if (!header) {
    throw new Error(`${filePath} is missing IHDR`);
  }
  if (header.bitDepth !== 8 || header.interlace !== 0 || ![0, 2, 4, 6].includes(header.colorType)) {
    throw new Error(`${filePath} uses unsupported PNG settings`);
  }

  return {
    ...header,
    pixels: inflateSync(Buffer.concat(chunks)),
  };
}

function computeImageStats(image) {
  const channels = image.colorType === 6 ? 4 : image.colorType === 2 ? 3 : image.colorType === 4 ? 2 : 1;
  const bytesPerPixel = channels;
  const stride = image.width * bytesPerPixel;
  let sourceOffset = 0;
  let previous = Buffer.alloc(stride);
  let minLuma = 255;
  let maxLuma = 0;
  let totalLuma = 0;
  let sampleCount = 0;

  for (let y = 0; y < image.height; y += 1) {
    const filter = image.pixels[sourceOffset];
    sourceOffset += 1;
    const current = Buffer.from(image.pixels.subarray(sourceOffset, sourceOffset + stride));
    sourceOffset += stride;
    unfilterScanline(current, previous, filter, bytesPerPixel);

    for (let x = 0; x < image.width; x += 4) {
      const index = x * bytesPerPixel;
      const [r, g, b] = colorAt(current, index, image.colorType);
      const luma = Math.round(0.2126 * r + 0.7152 * g + 0.0722 * b);
      minLuma = Math.min(minLuma, luma);
      maxLuma = Math.max(maxLuma, luma);
      totalLuma += luma;
      sampleCount += 1;
    }

    previous = current;
  }

  return {
    dynamicRange: maxLuma - minLuma,
    averageLuma: totalLuma / sampleCount,
  };
}

function colorAt(scanline, index, colorType) {
  if (colorType === 6 || colorType === 2) {
    return [scanline[index], scanline[index + 1], scanline[index + 2]];
  }
  const gray = scanline[index];
  return [gray, gray, gray];
}

function unfilterScanline(scanline, previous, filter, bytesPerPixel) {
  if (filter === 0) {
    return;
  }
  for (let index = 0; index < scanline.length; index += 1) {
    const left = index >= bytesPerPixel ? scanline[index - bytesPerPixel] : 0;
    const up = previous[index] ?? 0;
    const upLeft = index >= bytesPerPixel ? previous[index - bytesPerPixel] ?? 0 : 0;
    if (filter === 1) {
      scanline[index] = (scanline[index] + left) & 0xff;
    } else if (filter === 2) {
      scanline[index] = (scanline[index] + up) & 0xff;
    } else if (filter === 3) {
      scanline[index] = (scanline[index] + Math.floor((left + up) / 2)) & 0xff;
    } else if (filter === 4) {
      scanline[index] = (scanline[index] + paeth(left, up, upLeft)) & 0xff;
    } else {
      throw new Error(`Unsupported PNG filter ${filter}`);
    }
  }
}

function paeth(left, up, upLeft) {
  const prediction = left + up - upLeft;
  const leftDistance = Math.abs(prediction - left);
  const upDistance = Math.abs(prediction - up);
  const upLeftDistance = Math.abs(prediction - upLeft);
  if (leftDistance <= upDistance && leftDistance <= upLeftDistance) {
    return left;
  }
  if (upDistance <= upLeftDistance) {
    return up;
  }
  return upLeft;
}
