import { inflateSync } from "node:zlib";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { CANVAS_HEIGHT, CANVAS_WIDTH, DASHBOARD_CAPTURE_NAME, EXPORTS_LIST } from "./page-registry.mjs";

const MIN_FILE_SIZE = 20_000;
const MIN_DYNAMIC_RANGE = 20;

const here = dirname(fileURLToPath(import.meta.url));
const outputDir = resolve(here, "..");
const repoRoot = resolve(here, "../../..");
const sourceReference = resolve(outputDir, "hub.png");
const dashboardCapture = resolve(here, DASHBOARD_CAPTURE_NAME);
const exportsIndex = resolve(here, "EXPORTS.md");
const spotChecksPath = resolve(here, "SPOT_CHECKS.md");
const acceptanceEvidencePath = resolve(here, "ACCEPTANCE_EVIDENCE.md");
const aiManifest = resolve(outputDir, "hub-ai-reference-manifest.json");
const aiManifestSchema = resolve(outputDir, "hub-ai-reference-manifest.schema.json");
const aiDraftRoot = resolve(outputDir, "hub-ai-drafts");
const docsIndex = resolve(outputDir, "index.md");
const supplementalDesignArtifacts = [
  "hub-design-structure-layout.png",
  "hub-design-structure-supplement.png",
  "hub-design-functional-details.png",
];
const spotCheckArtifacts = [
  ["hub.png", "projects-dashboard"],
  ["hub-editor.png", "hub-editor"],
  ["hub-assets.png", "hub-assets"],
  ["hub-projects-browser.png", "hub-projects-browser"],
  ["hub-projects-detail-delete-confirm.png", "hub-projects-detail-delete-confirm"],
  ["hub-source-engine-popup.png", "hub-source-engine-popup"],
  ["hub-state-empty.png", "hub-state-empty"],
  ["hub-state-error.png", "hub-state-error"],
];

const regions = [
  ["full", null, 9.75, 27.63],
  ["topbar", [0, 0, 1568, 74], 10.55, 30.38],
  ["workspace", [223, 74, 1568, 866], 9.35, 27.88],
  ["project_cards", [253, 252, 1502, 505], 12.24, 27.49],
  ["quick_panel", [1071, 526, 1535, 839], 12.16, 35.15],
  ["bottom_strip", [0, 866, 1568, 1003], 11.00, 23.62],
];

for (const [, outputName] of EXPORTS_LIST) {
  validateExport(resolve(outputDir, outputName));
}
validateRootPngInventory();
validateOldGeneratorRetired();
validateExportsIndex();
validateSpotChecks();
validateDocsIndex();
validateReadme();
validateAcceptanceEvidence();
const aiDraftReport = validateAiManifest();

const source = decodePng(sourceReference);
const capture = decodePng(dashboardCapture);
assertSize(sourceReference, source, CANVAS_WIDTH, CANVAS_HEIGHT);
assertSize(dashboardCapture, capture, CANVAS_WIDTH, CANVAS_HEIGHT);

const comparisons = regions.map(([name, box, expectedMean, expectedRms]) => {
  const result = compareRegion(source, capture, box);
  assertWithin(name, "mean", result.mean, expectedMean, 0.04);
  assertWithin(name, "rms", result.rms, expectedRms, 0.04);
  return `${name}: mean=${result.mean.toFixed(2)} rms=${result.rms.toFixed(2)}`;
});

console.log(`validated ${EXPORTS_LIST.length} exported PNGs at ${CANVAS_WIDTH}x${CANVAS_HEIGHT}`);
console.log("validated docs/ui-and-layout root Hub PNG inventory");
console.log("validated retired Hub deterministic generator is absent");
console.log("validated EXPORTS.md against page-registry.mjs");
console.log("validated Hub web-reference spot-check artifact list");
console.log("validated docs/ui-and-layout/index.md Hub artifact matrix coverage");
console.log("validated Hub web-reference README policy text");
console.log("validated Hub web-reference acceptance evidence ledger");
console.log(
  `validated hub-ai-reference-manifest.json against page-registry.mjs (${aiDraftReport.validatedCount}/${EXPORTS_LIST.length} required AI drafts checked; no orphaned AI draft PNGs or stray draft files)`,
);
console.log(comparisons.join("\n"));

function validateExport(filePath) {
  if (!existsSync(filePath)) {
    throw new Error(`Missing exported PNG: ${filePath}`);
  }
  const size = statSync(filePath).size;
  if (size <= MIN_FILE_SIZE) {
    throw new Error(`Exported PNG is suspiciously small: ${filePath} (${size} bytes)`);
  }

  const image = decodePng(filePath);
  assertSize(filePath, image, CANVAS_WIDTH, CANVAS_HEIGHT);
  const dynamicRange = maxDynamicRange(image.rgb);
  if (dynamicRange <= MIN_DYNAMIC_RANGE) {
    throw new Error(`Exported PNG has low dynamic range: ${filePath} (${dynamicRange})`);
  }
}

function validateRootPngInventory() {
  const expected = new Set(["hub.png", ...EXPORTS_LIST.map(([, outputName]) => outputName), ...supplementalDesignArtifacts]);
  const actual = readdirSync(outputDir)
    .filter((fileName) => fileName.startsWith("hub") && fileName.endsWith(".png"))
    .sort();
  const missing = [...expected].filter((fileName) => !actual.includes(fileName)).sort();
  const orphaned = actual.filter((fileName) => !expected.has(fileName));

  if (missing.length > 0 || orphaned.length > 0) {
    throw new Error(
      `docs/ui-and-layout Hub PNG inventory mismatch. Missing: ${missing.join(", ") || "<none>"}. Orphaned: ${orphaned.join(", ") || "<none>"}.`,
    );
  }
}

function validateOldGeneratorRetired() {
  const retiredGenerator = resolve(repoRoot, "tools/generate-hub-design-assets.py");
  if (existsSync(retiredGenerator)) {
    throw new Error(
      `${retiredGenerator} must not exist; Hub design PNGs are now AI-directed and HTML/CSS-finalized through docs/ui-and-layout/hub-web-reference/export-pages.mjs.`,
    );
  }
}

function validateExportsIndex() {
  if (!existsSync(exportsIndex)) {
    throw new Error(`Missing export index: ${exportsIndex}`);
  }

  const rows = [
    ...readFileSync(exportsIndex, "utf8").matchAll(/^- ([^:]+): ([^ ]+) \(([^)]+)\)$/gm),
  ].map((match) => [match[2], match[1], match[3]]);
  if (rows.length !== EXPORTS_LIST.length) {
    throw new Error(`EXPORTS.md lists ${rows.length} rows; expected ${EXPORTS_LIST.length}`);
  }

  for (let index = 0; index < EXPORTS_LIST.length; index += 1) {
    const [expectedPageId, expectedOutputName] = EXPORTS_LIST[index];
    const [actualPageId, actualOutputName, actualReplayPath] = rows[index];
    if (actualPageId !== expectedPageId || actualOutputName !== expectedOutputName) {
      throw new Error(
        `EXPORTS.md row ${index + 1} is ${actualOutputName}: ${actualPageId}; expected ${expectedOutputName}: ${expectedPageId}`,
      );
    }
    const expectedReplayPath = `docs/ui-and-layout/hub-web-reference/index.html?page=${encodeURIComponent(expectedPageId)}`;
    if (actualReplayPath !== expectedReplayPath) {
      throw new Error(
        `EXPORTS.md row ${index + 1} replay path is ${actualReplayPath}; expected ${expectedReplayPath}`,
      );
    }
  }
}

function validateSpotChecks() {
  if (!existsSync(spotChecksPath)) {
    throw new Error(`Missing Hub web-reference spot-check list: ${spotChecksPath}`);
  }

  const source = readFileSync(spotChecksPath, "utf8");
  for (const snippet of [
    "Hub Web Reference Spot Checks",
    "AI-directed, HTML/CSS-finalized Hub reference PNGs",
    "19-page export validation",
    "against `docs/ui-and-layout/hub.png`",
    "Latest review: 2026-05-30",
    "no clipped text",
    "no overlap",
    "matching density",
    "consistent button/badge",
  ]) {
    if (!source.includes(snippet)) {
      throw new Error(`Hub web-reference SPOT_CHECKS.md is missing required text: ${snippet}`);
    }
  }

  const rows = [...source.matchAll(/^\| `([^`]+)` \| `([^`]+)` \| ([^|]+) \| ([^|]+) \| ([^|]+) \|$/gm)].map((match) => ({
    artifact: match[1],
    pageId: match[2],
    inspectFor: match[3],
    result: match[4].trim(),
    evidence: match[5].trim(),
  }));
  if (rows.length !== spotCheckArtifacts.length) {
    throw new Error(`SPOT_CHECKS.md lists ${rows.length} artifact rows; expected ${spotCheckArtifacts.length}`);
  }

  const knownPageIds = new Set([["projects-dashboard"], ...EXPORTS_LIST.map(([pageId]) => [pageId])].flat());
  for (let index = 0; index < spotCheckArtifacts.length; index += 1) {
    const [expectedArtifact, expectedPageId] = spotCheckArtifacts[index];
    const row = rows[index];
    if (row.artifact !== expectedArtifact || row.pageId !== expectedPageId) {
      throw new Error(
        `SPOT_CHECKS.md row ${index + 1} is ${row.artifact}/${row.pageId}; expected ${expectedArtifact}/${expectedPageId}`,
      );
    }
    if (!knownPageIds.has(row.pageId)) {
      throw new Error(`SPOT_CHECKS.md row ${row.artifact} uses unknown page id ${row.pageId}`);
    }
    validateExport(resolve(outputDir, row.artifact));
    if (row.inspectFor.length < 32) {
      throw new Error(`SPOT_CHECKS.md row ${row.artifact} must include a concrete inspection reason`);
    }
    if (row.result !== "Pass") {
      throw new Error(`SPOT_CHECKS.md row ${row.artifact} must record a Pass result; found ${row.result}`);
    }
    if (!row.evidence.includes("index.html?page=") && !row.evidence.includes("validate-visuals.mjs")) {
      throw new Error(`SPOT_CHECKS.md row ${row.artifact} must cite replay or validation evidence`);
    }
  }
}

function validateDocsIndex() {
  if (!existsSync(docsIndex)) {
    throw new Error(`Missing UI layout docs index: ${docsIndex}`);
  }

  const source = readFileSync(docsIndex, "utf8");
  for (const snippet of [
    "## Hub Visual Design Artifacts",
    "### Hub Visual Artifact Matrix",
    "docs/ui-and-layout/hub-ai-reference-manifest.json",
    "docs/ui-and-layout/hub-ai-reference-manifest.schema.json",
    "schema subset validation",
    "negative schema self-tests",
    "docs/ui-and-layout/hub-web-reference/export-pages.mjs",
    "docs/ui-and-layout/hub-web-reference/index.html?page=<page_id>",
    "falls back to a free local port",
    "ZIRCON_HUB_WEB_REFERENCE_PORT",
    "docs/ui-and-layout/hub-web-reference/SPOT_CHECKS.md",
    "docs/ui-and-layout/hub-web-reference/ACCEPTANCE_EVIDENCE.md",
    "Representative manual visual spot checks",
    "generated 19-file final PNG inventory",
    "export port fallback evidence",
    "interaction temp-profile cleanup evidence",
    "known optional `cargo check` timeout",
    "retired `tools/generate-hub-design-assets.py` Pillow generator",
    "must not be restored as the authoritative Hub PNG source",
  ]) {
    if (!source.includes(snippet)) {
      throw new Error(`docs/ui-and-layout/index.md is missing Hub visual workflow text: ${snippet}`);
    }
  }

  const matrixRows = [...source.matchAll(/^\| `([^`]+)` \|/gm)].map((match) => match[1]);
  const expectedArtifacts = ["hub.png", ...EXPORTS_LIST.map(([, outputName]) => outputName)];
  const missingArtifacts = expectedArtifacts.filter((artifact) => !matrixRows.includes(artifact));
  if (missingArtifacts.length > 0) {
    throw new Error(`docs/ui-and-layout/index.md Hub artifact matrix is missing: ${missingArtifacts.join(", ")}`);
  }

  for (const [pageId, outputName] of EXPORTS_LIST) {
    const expectedReplayPath = `docs/ui-and-layout/hub-web-reference/index.html?page=${pageId}`;
    if (!source.includes(outputName) || !source.includes(pageId)) {
      throw new Error(`docs/ui-and-layout/index.md must document ${outputName} with page id ${pageId}`);
    }
    if (!source.includes(expectedReplayPath) && !source.includes("index.html?page=<page_id>")) {
      throw new Error(`docs/ui-and-layout/index.md must document replay path ${expectedReplayPath}`);
    }
  }
}

function validateReadme() {
  const readme = resolve(here, "README.md");
  if (!existsSync(readme)) {
    throw new Error(`Missing Hub web-reference README: ${readme}`);
  }

  const source = readFileSync(readme, "utf8");
  for (const snippet of [
    "AI structure drafts are not final acceptance evidence",
    "all 19 page/state",
    "drafts must be present",
    "the HTML/CSS exported PNGs are accepted",
    "`docs/ui-and-layout/index.md` artifact matrix coverage",
    "opens every replay path listed in `EXPORTS.md`",
    "Unknown `--only` page ids fail before capture starts",
    "available page ids from `page-registry.mjs`",
    "automatically falls back to a",
    "`ZIRCON_HUB_WEB_REFERENCE_PORT` remains strict",
    "old `tools/generate-hub-design-assets.py` generator is retired",
    "AI-directed, HTML/CSS-finalized workflow",
    "`SPOT_CHECKS.md` records the representative manual visual inspection set",
    "`ACCEPTANCE_EVIDENCE.md` records the export command",
    "`cover-rendering.js` maps project ids to local reference cover images",
    "`covers.css` owns project cover image treatment",
    "`responsive.css` owns live-window responsive overrides",
    "`fullscreen-preview.css` makes normal browser preview fill the viewport",
    "`LAYOUT_MODEL.md` records the CSS region model",
    "`validate-responsive.mjs` checks all browser-openable pages",
    "`640x640`",
    "five live resize steps",
    "Runtime preview does not depend on third-party UI libraries or screenshot",
    "6 applied state interactions",
    "project cover assets",
    "`../hub-ai-reference-manifest.schema.json` defines the required manifest",
    "19 reference rows",
    "evidence links.",
    "exactly the 19 manifest-listed PNGs",
    "no orphaned AI draft PNGs or stray draft files",
    "no orphaned AI draft PNGs, non-PNG files, or nested",
    "exact AI draft directory inventory with no orphaned PNGs or stray files",
    "focused Cargo contract result",
    "optional `cargo check` timeout",
    "dashboard, Editor, Assets, Project Detail delete confirmation, Source Engine",
    "popup, Empty, and Error",
  ]) {
    if (!source.includes(snippet)) {
      throw new Error(`Hub web-reference README is missing validation policy text: ${snippet}`);
    }
  }
}

function validateAcceptanceEvidence() {
  if (!existsSync(acceptanceEvidencePath)) {
    throw new Error(`Missing Hub web-reference acceptance evidence ledger: ${acceptanceEvidencePath}`);
  }

  const source = readFileSync(acceptanceEvidencePath, "utf8");
  for (const snippet of [
    "Hub Web Reference Acceptance Evidence",
    "Date: 2026-05-30",
    "Canvas: `1568x1003`",
    "`docs/ui-and-layout/hub.png` remains the fixed Projects Dashboard source",
    "AI structure drafts and design-board screenshots are review support, not final acceptance evidence",
    "node docs/ui-and-layout/hub-web-reference/export-pages.mjs",
    "Default port `5198` falls back to a free local port",
    "explicit `ZIRCON_HUB_WEB_REFERENCE_PORT` remains strict",
    "node docs/ui-and-layout/hub-web-reference/validate-visuals.mjs",
    "node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs",
    "node docs/ui-and-layout/hub-web-reference/validate-responsive.mjs",
    "docs/ui-and-layout/hub-ai-reference-manifest.schema.json",
    "node docs/ui-and-layout/hub-design-board/validate-design-board.mjs",
    "cargo test --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1",
    "--test ui_visual_standard_contract",
    "Generated Final PNG Inventory",
    "Automated Gate Results",
    "Web reference export | Pass",
    "default port fallback and strict explicit-port failure were both exercised",
    "Web reference visual validation | Pass",
    "AI manifest schema, schema subset validation, negative schema self-tests",
    "19 required AI drafts at `1024x1024`",
    "exact AI draft directory inventory with no orphaned AI draft PNGs or stray draft files",
    "Web reference interaction validation | Pass",
    "6 applied state interactions",
    "left no `zircon-hub-cdp-*` temp profile",
    "Web reference responsive validation | Pass",
    "validated the dashboard plus all 19 exported pages across `1568x1003`, `1920x1080`, `1915x508`, `1600x1024`, `1280x900`, `1024x720`, `900x720`, `760x680`, and `640x640`",
    "large-preview dashboard card expansion",
    "7 representative pages through 5 live resize steps",
    "required `.project-cover-image` reference cover elements on project pages",
    "Projects Browser header/every-row column alignment",
    "`zircon_hub/assets/covers/reference/*` local cover images",
    "Responsive browser preview is accepted through DOM geometry validation and",
    "Design-board validation | Pass",
    "Focused Rust visual contract | Pass",
    "8 passed, 0 failed, 0 ignored",
    "optional `cargo check` timeout",
    "not acceptance evidence for or against this visual-reference slice",
    "AI draft inventory is still enforced",
    "must contain exactly the 19 manifest-listed PNGs and no stray draft files",
    "Accept only the final web-reference PNGs and their static validation package",
  ]) {
    if (!source.includes(snippet)) {
      throw new Error(`Hub web-reference ACCEPTANCE_EVIDENCE.md is missing required text: ${snippet}`);
    }
  }

  for (const [, outputName] of EXPORTS_LIST) {
    if (!source.includes(`- \`${outputName}\``)) {
      throw new Error(`Hub web-reference ACCEPTANCE_EVIDENCE.md is missing generated PNG ${outputName}`);
    }
  }
}

function validateAiManifest() {
  if (!existsSync(aiManifest)) {
    throw new Error(`Missing AI reference manifest: ${aiManifest}`);
  }

  const manifest = JSON.parse(readFileSync(aiManifest, "utf8"));
  validateAiManifestSchema(manifest);
  if (manifest.source_reference !== "docs/ui-and-layout/hub.png") {
    throw new Error(`Manifest source_reference is ${manifest.source_reference}; expected docs/ui-and-layout/hub.png`);
  }
  if (manifest.final_source !== "docs/ui-and-layout/hub-web-reference/index.html") {
    throw new Error(
      `Manifest final_source is ${manifest.final_source}; expected docs/ui-and-layout/hub-web-reference/index.html`,
    );
  }
  if (manifest.export_command !== "node docs/ui-and-layout/hub-web-reference/export-pages.mjs") {
    throw new Error(
      `Manifest export_command is ${manifest.export_command}; expected node docs/ui-and-layout/hub-web-reference/export-pages.mjs`,
    );
  }
  validateManifestAcceptanceEvidence(manifest);
  if (manifest.canvas?.width !== CANVAS_WIDTH || manifest.canvas?.height !== CANVAS_HEIGHT) {
    throw new Error(
      `Manifest canvas is ${manifest.canvas?.width}x${manifest.canvas?.height}; expected ${CANVAS_WIDTH}x${CANVAS_HEIGHT}`,
    );
  }
  if (manifest.draft_kind !== "overall-interaction-structure-layout") {
    throw new Error(`Manifest draft_kind is ${manifest.draft_kind}; expected overall-interaction-structure-layout`);
  }
  if (manifest.ai_draft_root !== "docs/ui-and-layout/hub-ai-drafts") {
    throw new Error(
      `Manifest ai_draft_root is ${manifest.ai_draft_root}; expected docs/ui-and-layout/hub-ai-drafts`,
    );
  }
  for (const snippet of ["structure-layout direction images only", "not acceptance evidence", "partial image"]) {
    const haystack = `${manifest.draft_status ?? ""} ${manifest.draft_usage ?? ""} ${manifest.streaming_generation?.fallback ?? ""}`;
    if (!haystack.includes(snippet)) {
      throw new Error(`Manifest must document AI draft usage/fallback; missing ${snippet}`);
    }
  }
  if (!Array.isArray(manifest.references)) {
    throw new Error("Manifest references must be an array");
  }
  if (manifest.references.length !== EXPORTS_LIST.length) {
    throw new Error(`Manifest lists ${manifest.references.length} references; expected ${EXPORTS_LIST.length}`);
  }
  validateSupplementalDesignManifest(manifest);

  let validatedDrafts = 0;
  for (let index = 0; index < EXPORTS_LIST.length; index += 1) {
    const [expectedPageId, expectedOutputName] = EXPORTS_LIST[index];
    const reference = manifest.references[index];
    if (reference.page_id !== expectedPageId || reference.output !== expectedOutputName) {
      throw new Error(
        `Manifest reference ${index + 1} is ${reference.page_id}: ${reference.output}; expected ${expectedPageId}: ${expectedOutputName}`,
      );
    }
    const expectedDraft = `docs/ui-and-layout/hub-ai-drafts/${expectedPageId}.png`;
    if (reference.ai_draft !== expectedDraft) {
      throw new Error(
        `Manifest reference ${index + 1} has ai_draft ${reference.ai_draft}; expected ${expectedDraft}`,
      );
    }
    const draftPath = resolve(aiDraftRoot, `${expectedPageId}.png`);
    if (existsSync(draftPath)) {
      validateAiDraft(draftPath);
      validatedDrafts += 1;
    } else {
      throw new Error(`Missing AI structure-layout draft: ${draftPath}`);
    }
  }
  validateAiDraftInventory();
  return { validatedCount: validatedDrafts };
}

function validateAiDraftInventory() {
  if (!existsSync(aiDraftRoot)) {
    throw new Error(`Missing AI draft directory: ${aiDraftRoot}`);
  }

  const entries = readdirSync(aiDraftRoot, { withFileTypes: true });
  const strayEntries = entries
    .filter((entry) => !entry.isFile() || !entry.name.endsWith(".png"))
    .map((entry) => entry.isDirectory() ? `${entry.name}/` : entry.name)
    .sort();
  if (strayEntries.length > 0) {
    throw new Error(`AI draft directory contains non-PNG or nested entries: ${strayEntries.join(", ")}`);
  }

  const expected = EXPORTS_LIST.map(([pageId]) => `${pageId}.png`).sort();
  const actual = entries.map((entry) => entry.name).sort();
  const missing = expected.filter((fileName) => !actual.includes(fileName));
  const orphaned = actual.filter((fileName) => !expected.includes(fileName));
  if (missing.length > 0 || orphaned.length > 0) {
    throw new Error(
      `AI draft PNG inventory mismatch. Missing: ${missing.join(", ") || "<none>"}. Orphaned: ${orphaned.join(", ") || "<none>"}.`,
    );
  }
}

function validateAiManifestSchema(manifest) {
  if (!existsSync(aiManifestSchema)) {
    throw new Error(`Missing AI reference manifest schema: ${aiManifestSchema}`);
  }

  const schema = JSON.parse(readFileSync(aiManifestSchema, "utf8"));
  if (manifest.$schema !== "docs/ui-and-layout/hub-ai-reference-manifest.schema.json") {
    throw new Error(
      `Manifest $schema is ${manifest.$schema}; expected docs/ui-and-layout/hub-ai-reference-manifest.schema.json`,
    );
  }
  if (schema.$id !== "docs/ui-and-layout/hub-ai-reference-manifest.schema.json") {
    throw new Error(`AI manifest schema $id is ${schema.$id}; expected docs/ui-and-layout/hub-ai-reference-manifest.schema.json`);
  }
  if (schema.additionalProperties !== false) {
    throw new Error("AI manifest schema must reject additional top-level properties");
  }

  requireSchemaEntries(schema.required, [
    "$schema",
    "source_reference",
    "prompt_family",
    "draft_kind",
    "streaming_generation",
    "supplemental_design_mode_artifacts",
    "design_board_workflow",
    "final_source",
    "export_command",
    "acceptance_evidence",
    "canvas",
    "references",
  ], "AI manifest schema top-level required");

  const references = schema.properties?.references;
  if (references?.minItems !== EXPORTS_LIST.length || references?.maxItems !== EXPORTS_LIST.length) {
    throw new Error(
      `AI manifest schema references count is ${references?.minItems}/${references?.maxItems}; expected ${EXPORTS_LIST.length}`,
    );
  }
  requireSchemaEntries(references.items?.required, ["page_id", "output", "ai_draft", "selected_draft", "prompt_focus"], "AI manifest schema reference item required");

  const supplemental = schema.properties?.supplemental_design_mode_artifacts;
  if (supplemental?.minItems !== supplementalDesignArtifacts.length || supplemental?.maxItems !== supplementalDesignArtifacts.length) {
    throw new Error(
      `AI manifest schema supplemental count is ${supplemental?.minItems}/${supplemental?.maxItems}; expected ${supplementalDesignArtifacts.length}`,
    );
  }

  requireSchemaEntries(schema.properties?.acceptance_evidence?.required, [
    "ledger",
    "final_png_inventory",
    "spot_checks",
    "web_exports",
    "visual_validation",
    "interaction_validation",
    "design_board_validation",
    "rust_contract",
    "known_limit",
  ], "AI manifest schema acceptance evidence required");
  if (schema.properties?.canvas?.properties?.width?.const !== CANVAS_WIDTH || schema.properties?.canvas?.properties?.height?.const !== CANVAS_HEIGHT) {
    throw new Error("AI manifest schema canvas constants must match the Hub reference canvas");
  }
  validateJsonSchemaSubset(manifest, schema, "hub-ai-reference-manifest.json");
  validateJsonSchemaSubsetSelfTest(manifest, schema);
}

function requireSchemaEntries(actual, expected, label) {
  if (!Array.isArray(actual)) {
    throw new Error(`${label} must be an array`);
  }
  const missing = expected.filter((entry) => !actual.includes(entry));
  if (missing.length > 0) {
    throw new Error(`${label} is missing: ${missing.join(", ")}`);
  }
}

function validateJsonSchemaSubset(value, schema, path) {
  if (!schema || typeof schema !== "object" || Array.isArray(schema)) {
    throw new Error(`Invalid JSON schema node at ${path}`);
  }

  if (schema.const !== undefined && !jsonEqual(value, schema.const)) {
    throw new Error(`${path} must equal ${JSON.stringify(schema.const)}; got ${JSON.stringify(value)}`);
  }

  if (schema.type) {
    validateSchemaType(value, schema.type, path);
  }

  if (schema.minLength !== undefined && typeof value === "string" && value.length < schema.minLength) {
    throw new Error(`${path} length ${value.length} is below minLength ${schema.minLength}`);
  }

  if (schema.pattern !== undefined && typeof value === "string" && !new RegExp(schema.pattern).test(value)) {
    throw new Error(`${path} does not match schema pattern ${schema.pattern}`);
  }

  if (Array.isArray(value)) {
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

  if (isPlainObject(value)) {
    const properties = schema.properties ?? {};
    if (schema.additionalProperties === false) {
      const unknown = Object.keys(value).filter((key) => !hasOwn(properties, key));
      if (unknown.length > 0) {
        throw new Error(`${path} has unknown schema properties: ${unknown.join(", ")}`);
      }
    }
    if (Array.isArray(schema.required)) {
      const missing = schema.required.filter((key) => !hasOwn(value, key));
      if (missing.length > 0) {
        throw new Error(`${path} is missing required schema properties: ${missing.join(", ")}`);
      }
    }
    for (const [key, propertySchema] of Object.entries(properties)) {
      if (hasOwn(value, key)) {
        validateJsonSchemaSubset(value[key], propertySchema, `${path}.${key}`);
      }
    }
  }
}

function validateJsonSchemaSubsetSelfTest(manifest, schema) {
  const cases = [
    {
      name: "rejects unknown top-level properties",
      mutate: (copy) => {
        copy.untracked_note = "must fail";
      },
      expectedMessage: "unknown schema properties",
    },
    {
      name: "rejects missing reference output",
      mutate: (copy) => {
        delete copy.references[0].output;
      },
      expectedMessage: "missing required schema properties",
    },
    {
      name: "rejects short reference inventory",
      mutate: (copy) => {
        copy.references = copy.references.slice(0, -1);
      },
      expectedMessage: "expected at least 19",
    },
    {
      name: "rejects malformed AI draft path",
      mutate: (copy) => {
        copy.references[0].ai_draft = "docs/ui-and-layout/hub-ai-drafts/not-a-png.txt";
      },
      expectedMessage: "schema pattern",
    },
    {
      name: "rejects wrong canvas width",
      mutate: (copy) => {
        copy.canvas.width = 1200;
      },
      expectedMessage: "must equal 1568",
    },
  ];

  for (const testCase of cases) {
    expectSchemaFailure(testCase.name, testCase.mutate, testCase.expectedMessage, manifest, schema);
  }
}

function expectSchemaFailure(name, mutate, expectedMessage, manifest, schema) {
  const invalidManifest = cloneJson(manifest);
  mutate(invalidManifest);
  try {
    validateJsonSchemaSubset(invalidManifest, schema, `schema self-test ${name}`);
  } catch (error) {
    const message = String(error?.message ?? error);
    if (!message.includes(expectedMessage)) {
      throw new Error(`Schema self-test "${name}" failed with unexpected message: ${message}`);
    }
    return;
  }
  throw new Error(`Schema self-test "${name}" did not reject the malformed manifest`);
}

function validateSchemaType(value, expectedType, path) {
  const valid =
    (expectedType === "object" && isPlainObject(value)) ||
    (expectedType === "array" && Array.isArray(value)) ||
    (expectedType === "string" && typeof value === "string") ||
    (expectedType === "number" && typeof value === "number");
  if (!valid) {
    throw new Error(`${path} must be schema type ${expectedType}; got ${Array.isArray(value) ? "array" : typeof value}`);
  }
}

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function hasOwn(value, key) {
  return Object.prototype.hasOwnProperty.call(value, key);
}

function jsonEqual(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function cloneJson(value) {
  return JSON.parse(JSON.stringify(value));
}

function validateManifestAcceptanceEvidence(manifest) {
  const evidence = manifest.acceptance_evidence;
  if (!evidence || typeof evidence !== "object") {
    throw new Error("Manifest must include acceptance_evidence");
  }

  const expectedEntries = {
    ledger: "docs/ui-and-layout/hub-web-reference/ACCEPTANCE_EVIDENCE.md",
    final_png_inventory: "docs/ui-and-layout/hub-web-reference/ACCEPTANCE_EVIDENCE.md#generated-final-png-inventory",
    spot_checks: "docs/ui-and-layout/hub-web-reference/SPOT_CHECKS.md",
    web_exports: "docs/ui-and-layout/hub-web-reference/EXPORTS.md",
    visual_validation: "node docs/ui-and-layout/hub-web-reference/validate-visuals.mjs",
    interaction_validation: "node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs",
    design_board_validation: "node docs/ui-and-layout/hub-design-board/validate-design-board.mjs",
    rust_contract:
      "cargo test --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1 --test ui_visual_standard_contract",
    known_limit:
      "AI drafts and design-board screenshots are not final acceptance evidence; optional cargo check may time out under concurrent Hub library compilation.",
  };

  for (const [key, expectedValue] of Object.entries(expectedEntries)) {
    if (evidence[key] !== expectedValue) {
      throw new Error(`Manifest acceptance_evidence.${key} is ${evidence[key]}; expected ${expectedValue}`);
    }
  }
}

function validateSupplementalDesignManifest(manifest) {
  const supplemental = manifest.supplemental_design_mode_artifacts;
  if (!Array.isArray(supplemental) || supplemental.length !== supplementalDesignArtifacts.length) {
    throw new Error(
      `Manifest supplemental design artifacts count is ${supplemental?.length}; expected ${supplementalDesignArtifacts.length}`,
    );
  }

  for (const outputName of supplementalDesignArtifacts) {
    const expectedFile = `docs/ui-and-layout/${outputName}`;
    const entry = supplemental.find((item) => item.file === expectedFile);
    if (!entry) {
      throw new Error(`Manifest is missing supplemental design artifact ${expectedFile}`);
    }
    if (entry.acceptance !== "Not final acceptance evidence.") {
      throw new Error(`Supplemental design artifact ${expectedFile} must not be acceptance evidence`);
    }
    validateSupplementalDesign(resolve(outputDir, outputName));
  }
}

function validateAiDraft(filePath) {
  const size = statSync(filePath).size;
  if (size <= MIN_FILE_SIZE) {
    throw new Error(`AI structure draft is suspiciously small: ${filePath} (${size} bytes)`);
  }
  const image = decodePng(filePath);
  assertSize(filePath, image, 1024, 1024);
  const dynamicRange = maxDynamicRange(image.rgb);
  if (dynamicRange <= MIN_DYNAMIC_RANGE) {
    throw new Error(`AI structure draft has low dynamic range: ${filePath} (${dynamicRange})`);
  }
}

function validateSupplementalDesign(filePath) {
  if (!existsSync(filePath)) {
    throw new Error(`Missing supplemental design-mode artifact: ${filePath}`);
  }
  const size = statSync(filePath).size;
  if (size <= MIN_FILE_SIZE) {
    throw new Error(`Supplemental design-mode artifact is suspiciously small: ${filePath} (${size} bytes)`);
  }
  const image = decodePng(filePath);
  assertSize(filePath, image, CANVAS_WIDTH, CANVAS_HEIGHT);
  const dynamicRange = maxDynamicRange(image.rgb);
  if (dynamicRange <= MIN_DYNAMIC_RANGE) {
    throw new Error(`Supplemental design-mode artifact has low dynamic range: ${filePath} (${dynamicRange})`);
  }
}

function assertSize(filePath, image, width, height) {
  if (image.width !== width || image.height !== height) {
    throw new Error(`${filePath} has size ${image.width}x${image.height}; expected ${width}x${height}`);
  }
}

function assertWithin(regionName, metricName, actual, expected, tolerance) {
  if (Math.abs(actual - expected) > tolerance) {
    throw new Error(
      `${regionName} ${metricName} ${actual.toFixed(2)} differs from expected ${expected.toFixed(2)} by more than ${tolerance}`,
    );
  }
}

function compareRegion(a, b, box) {
  const [x0, y0, x1, y1] = box ?? [0, 0, a.width, a.height];
  let sum = 0;
  let squared = 0;
  let count = 0;

  for (let y = y0; y < y1; y += 1) {
    for (let x = x0; x < x1; x += 1) {
      const offset = (y * a.width + x) * 3;
      for (let channel = 0; channel < 3; channel += 1) {
        const diff = Math.abs(a.rgb[offset + channel] - b.rgb[offset + channel]);
        sum += diff;
        squared += diff * diff;
        count += 1;
      }
    }
  }

  return {
    mean: sum / count,
    rms: Math.sqrt(squared / count),
  };
}

function maxDynamicRange(rgb) {
  const min = [255, 255, 255];
  const max = [0, 0, 0];
  for (let offset = 0; offset < rgb.length; offset += 3) {
    for (let channel = 0; channel < 3; channel += 1) {
      const value = rgb[offset + channel];
      min[channel] = Math.min(min[channel], value);
      max[channel] = Math.max(max[channel], value);
    }
  }
  return Math.max(max[0] - min[0], max[1] - min[1], max[2] - min[2]);
}

function decodePng(filePath) {
  const bytes = readFileSync(filePath);
  const signature = "89504e470d0a1a0a";
  if (bytes.subarray(0, 8).toString("hex") !== signature) {
    throw new Error(`${filePath} is not a PNG file`);
  }

  let offset = 8;
  let header = null;
  const idatChunks = [];

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
      idatChunks.push(data);
    } else if (type === "IEND") {
      break;
    }
  }

  if (!header) {
    throw new Error(`${filePath} is missing IHDR`);
  }
  if (header.bitDepth !== 8 || header.interlace !== 0) {
    throw new Error(`${filePath} uses unsupported PNG settings`);
  }

  const channels = channelsForColorType(filePath, header.colorType);
  const inflated = inflateSync(Buffer.concat(idatChunks));
  const stride = header.width * channels;
  const raw = new Uint8Array(header.height * stride);
  let inputOffset = 0;

  for (let y = 0; y < header.height; y += 1) {
    const filter = inflated[inputOffset];
    inputOffset += 1;
    const rowStart = y * stride;
    for (let x = 0; x < stride; x += 1) {
      const rawValue = inflated[inputOffset];
      inputOffset += 1;
      const left = x >= channels ? raw[rowStart + x - channels] : 0;
      const up = y > 0 ? raw[rowStart + x - stride] : 0;
      const upLeft = y > 0 && x >= channels ? raw[rowStart + x - stride - channels] : 0;
      raw[rowStart + x] = (rawValue + predictor(filter, left, up, upLeft)) & 0xff;
    }
  }

  return {
    width: header.width,
    height: header.height,
    rgb: toRgb(filePath, raw, header.colorType, channels),
  };
}

function channelsForColorType(filePath, colorType) {
  if (colorType === 0) return 1;
  if (colorType === 2) return 3;
  if (colorType === 4) return 2;
  if (colorType === 6) return 4;
  throw new Error(`${filePath} uses unsupported PNG color type ${colorType}`);
}

function predictor(filter, left, up, upLeft) {
  if (filter === 0) return 0;
  if (filter === 1) return left;
  if (filter === 2) return up;
  if (filter === 3) return Math.floor((left + up) / 2);
  if (filter === 4) return paeth(left, up, upLeft);
  throw new Error(`Unsupported PNG filter ${filter}`);
}

function paeth(left, up, upLeft) {
  const estimate = left + up - upLeft;
  const leftDistance = Math.abs(estimate - left);
  const upDistance = Math.abs(estimate - up);
  const upLeftDistance = Math.abs(estimate - upLeft);
  if (leftDistance <= upDistance && leftDistance <= upLeftDistance) return left;
  if (upDistance <= upLeftDistance) return up;
  return upLeft;
}

function toRgb(filePath, raw, colorType, channels) {
  const pixels = raw.length / channels;
  const rgb = new Uint8Array(pixels * 3);

  for (let pixel = 0; pixel < pixels; pixel += 1) {
    const sourceOffset = pixel * channels;
    const targetOffset = pixel * 3;
    if (colorType === 0 || colorType === 4) {
      const gray = raw[sourceOffset];
      rgb[targetOffset] = gray;
      rgb[targetOffset + 1] = gray;
      rgb[targetOffset + 2] = gray;
    } else if (colorType === 2 || colorType === 6) {
      rgb[targetOffset] = raw[sourceOffset];
      rgb[targetOffset + 1] = raw[sourceOffset + 1];
      rgb[targetOffset + 2] = raw[sourceOffset + 2];
    } else {
      throw new Error(`${filePath} uses unsupported PNG color type ${colorType}`);
    }
  }

  return rgb;
}
