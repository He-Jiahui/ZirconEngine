import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFile, readdir, stat } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { inflateSync } from "node:zlib";
import { ALL_DESIGNS } from "./design.js";
import { DESIGNS, HEIGHT, OUTPUT_DIR, WIDTH } from "./design-manifest.mjs";
import {
  npmConfigDesignArgs,
  parseDesignSelection,
  selectDesigns,
  shouldCapturePreviewSheet,
} from "./export-options.mjs";
import { previewSheetCoverageIds, previewSheetEntries } from "./preview-sheet.js";

const rootDir = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const outputDir = resolve(rootDir, OUTPUT_DIR);
const runtimeReferenceOverride =
  process.env.ZIRCON_WORKBENCH_REFERENCE_NEGATIVE_GUARD === "1"
    ? process.env.ZIRCON_WORKBENCH_RUNTIME_REFERENCE_OVERRIDE
    : null;
const referenceBaselines = [
  {
    label: "docs workbench.png",
    path: "docs/ui-and-layout/workbench.png",
  },
  {
    label: "editor reference workbench.png",
    path: runtimeReferenceOverride || "zircon_editor/assets/ui/editor/reference/workbench.png",
  },
];
const referenceSha256 = "4AD7706C08138EF422802C0C46B5DE4775237021F474200EFC100DAD577C02D0";
const referenceByteLength = 1526533;
const expected = [...DESIGNS.map((design) => design.output), "preview-sheet.png"];
const expectedSet = new Set(expected);
const failures = [];

await validateReferenceBaseline();
validateManifest();
validateRendererRegistry();
validatePreviewSheetCoverage();
validateExportOptions();
await validatePackageScripts();
await validateOutputDirectory();
await validateDocumentationBaselines();
validateNoStalePreviewProcesses();
await validatePreviewSheetFreshness();
await validateStyleNotesFreshness();
await validateDesignPngFreshness();

for (const filename of expected) {
  const file = resolve(outputDir, filename);
  try {
    const info = await stat(file);
    if (info.size <= 0) {
      failures.push(`${filename}: file is empty`);
      continue;
    }
    const header = await readFile(file);
    const size = readPngSize(header);
    if (!size) {
      failures.push(`${filename}: not a PNG`);
      continue;
    }
    if (size.width !== WIDTH || size.height !== HEIGHT) {
      failures.push(`${filename}: expected ${WIDTH}x${HEIGHT}, got ${size.width}x${size.height}`);
    }
    validateVisualProfile(filename, header, size);
  } catch (error) {
    failures.push(`${filename}: ${error.message}`);
  }
}

await validateStyleNotes();

if (failures.length) {
  console.error("Design verification failed:");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log(`Verified ${expected.length} PNG files at ${WIDTH}x${HEIGHT} in ${OUTPUT_DIR}.`);

async function validateReferenceBaseline() {
  for (const baseline of referenceBaselines) {
    await validateReferencePng(baseline);
  }
  await validateReferenceByteIdentity();
}

async function validateReferencePng(baseline) {
  try {
    const buffer = await readFile(resolve(rootDir, baseline.path));
    const actualSha256 = createHash("sha256").update(buffer).digest("hex").toUpperCase();
    if (actualSha256 !== referenceSha256) {
      failures.push(`${baseline.label}: SHA-256 ${actualSha256} != ${referenceSha256}`);
    }
    if (buffer.length !== referenceByteLength) {
      failures.push(`${baseline.label}: byte length ${buffer.length} != ${referenceByteLength}`);
    }

    const size = readPngSize(buffer);
    if (!size) {
      failures.push(`${baseline.label}: not a PNG`);
      return;
    }
    if (size.width !== WIDTH || size.height !== HEIGHT) {
      failures.push(`${baseline.label}: expected ${WIDTH}x${HEIGHT}, got ${size.width}x${size.height}`);
    }
    if (!readRgbPngVisualMetrics(buffer, size)) {
      failures.push(`${baseline.label}: expected non-interlaced 8-bit RGB PNG content`);
    }
  } catch (error) {
    failures.push(`${baseline.label}: ${error.message}`);
  }
}

async function validateReferenceByteIdentity() {
  try {
    const [source, runtime] = await Promise.all(
      referenceBaselines.map((baseline) => readFile(resolve(rootDir, baseline.path))),
    );
    if (!source.equals(runtime)) {
      failures.push(
        `${referenceBaselines[1].label}: not byte-identical to ${referenceBaselines[0].label}`,
      );
    }
  } catch (error) {
    failures.push(`reference byte identity: ${error.message}`);
  }
}

function validateManifest() {
  const duplicateIds = duplicates(DESIGNS.map((design) => design.id));
  const duplicateOutputs = duplicates(DESIGNS.map((design) => design.output));
  for (const id of duplicateIds) failures.push(`manifest duplicate id: ${id}`);
  for (const output of duplicateOutputs) failures.push(`manifest duplicate output: ${output}`);

  DESIGNS.forEach((design, index) => {
    if (!design.id || typeof design.id !== "string") {
      failures.push(`manifest entry ${index}: missing string id`);
    }
    if (!design.kind || typeof design.kind !== "string") {
      failures.push(`manifest entry ${index}: missing string kind`);
    }
    if (!design.output || typeof design.output !== "string") {
      failures.push(`manifest entry ${index}: missing string output`);
    } else if (!design.output.endsWith(".png")) {
      failures.push(`${design.output}: manifest output must end with .png`);
    }
  });
}

function validateRendererRegistry() {
  const duplicateRendererIds = duplicates(ALL_DESIGNS.map((design) => design.id));
  const duplicateRendererOutputs = duplicates(ALL_DESIGNS.map((design) => design.output));
  for (const id of duplicateRendererIds) failures.push(`renderer duplicate id: ${id}`);
  for (const output of duplicateRendererOutputs) {
    failures.push(`renderer duplicate output: ${output}`);
  }

  ALL_DESIGNS.forEach((design, index) => {
    if (!design.id || typeof design.id !== "string") {
      failures.push(`renderer entry ${index}: missing string id`);
    }
    if (!design.output || typeof design.output !== "string") {
      failures.push(`renderer entry ${index}: missing string output`);
    } else if (!design.output.endsWith(".png")) {
      failures.push(`${design.output}: renderer output must end with .png`);
    }
  });

  const manifestIds = new Set(DESIGNS.map((design) => design.id));
  const manifestById = new Map(DESIGNS.map((design) => [design.id, design]));
  const rendererIds = new Set(ALL_DESIGNS.map((design) => design.id));
  const rendererById = new Map(ALL_DESIGNS.map((design) => [design.id, design]));
  for (const id of [...manifestIds].sort()) {
    if (!rendererIds.has(id)) {
      failures.push(`${id}: manifest design id is not rendered by design.js`);
      continue;
    }
    const manifest = manifestById.get(id);
    const renderer = rendererById.get(id);
    if (renderer.output !== manifest.output) {
      failures.push(
        `${id}: manifest output ${manifest.output} does not match renderer output ${renderer.output}`,
      );
    }
  }
  for (const id of [...rendererIds].sort()) {
    if (!manifestIds.has(id)) {
      failures.push(`${id}: renderer design id is missing from design-manifest.mjs`);
    }
  }
}

function validatePreviewSheetCoverage() {
  const coverageIds = previewSheetCoverageIds(ALL_DESIGNS);
  const duplicateCoverageIds = duplicates(coverageIds);
  for (const id of duplicateCoverageIds) failures.push(`preview sheet duplicate design id: ${id}`);

  const manifestIds = new Set(DESIGNS.map((design) => design.id));
  const coverageIdSet = new Set(coverageIds);
  for (const id of [...manifestIds].sort()) {
    if (!coverageIdSet.has(id)) {
      failures.push(`${id}: preview sheet is missing manifest design id`);
    }
  }
  for (const id of [...coverageIdSet].sort()) {
    if (!manifestIds.has(id)) {
      failures.push(`${id}: preview sheet includes non-manifest design id`);
    }
  }

  const entries = previewSheetEntries(ALL_DESIGNS);
  if (entries.length !== DESIGNS.length) {
    failures.push(`preview sheet entry count ${entries.length} != manifest count ${DESIGNS.length}`);
  }
  for (const entry of entries) {
    if (!entry.output.endsWith(".png")) {
      failures.push(`${entry.id}: preview sheet output ${entry.output} must end with .png`);
    }
    if (!entry.title || !entry.description) {
      failures.push(`${entry.id}: preview sheet entry is missing title or description`);
    }
  }
}

function validateExportOptions() {
  const cases = [
    {
      name: "all without sheet",
      args: ["--all", "--no-sheet"],
      env: {},
      ids: null,
      captureSheet: false,
      selected: DESIGNS.length,
    },
    {
      name: "space joined positional ids",
      args: ["sound-cue-workbench audio-mixer-workbench", "--no-sheet"],
      env: {},
      ids: ["audio-mixer-workbench", "sound-cue-workbench"],
      captureSheet: false,
      selected: 2,
    },
    {
      name: "comma ids option",
      args: ["--ids=sound-cue-workbench,audio-mixer-workbench", "--no-sheet"],
      env: {},
      ids: ["audio-mixer-workbench", "sound-cue-workbench"],
      captureSheet: false,
      selected: 2,
    },
    {
      name: "separate ids option",
      args: ["--ids", "audio-mixer-workbench"],
      env: {},
      ids: ["audio-mixer-workbench"],
      captureSheet: false,
      selected: 1,
    },
    {
      name: "output filename selection",
      args: ["audio-mixer-workbench.png", "--no-sheet"],
      env: {},
      ids: ["audio-mixer-workbench.png"],
      captureSheet: false,
      selected: 1,
    },
    {
      name: "npm config ids without sheet empty value",
      args: [],
      env: { npm_config_ids: "sound-cue-workbench", npm_config_sheet: "" },
      ids: ["sound-cue-workbench"],
      captureSheet: false,
      selected: 1,
    },
    {
      name: "sheet only",
      args: ["sheet"],
      env: {},
      ids: ["sheet"],
      captureSheet: true,
      selected: 0,
    },
    {
      name: "sheet plus one design",
      args: ["--ids=sheet,sound-cue-workbench"],
      env: {},
      ids: ["sheet", "sound-cue-workbench"],
      captureSheet: true,
      selected: 1,
    },
  ];

  for (const testCase of cases) {
    const selection = parseDesignSelection([...npmConfigDesignArgs(testCase.env), ...testCase.args]);
    const actualIds = selection.ids ? [...selection.ids].sort() : null;
    if (!sameJson(actualIds, testCase.ids)) {
      failures.push(
        `export options ${testCase.name}: ids ${JSON.stringify(actualIds)} != ${JSON.stringify(
          testCase.ids,
        )}`,
      );
    }

    const actualCaptureSheet = shouldCapturePreviewSheet(selection);
    if (actualCaptureSheet !== testCase.captureSheet) {
      failures.push(
        `export options ${testCase.name}: capture sheet ${actualCaptureSheet} != ${testCase.captureSheet}`,
      );
    }

    const selected = selectDesigns(DESIGNS, selection.ids);
    if (selected.length !== testCase.selected) {
      failures.push(
        `export options ${testCase.name}: selected ${selected.length} != ${testCase.selected}`,
      );
    }
  }

  try {
    selectDesigns(DESIGNS, new Set(["missing-workbench"]));
    failures.push("export options unknown id: expected selectDesigns to throw");
  } catch (error) {
    if (!error.message.includes("missing-workbench")) {
      failures.push(`export options unknown id: unexpected error ${error.message}`);
    }
  }
}

async function validatePackageScripts() {
  const filename = "tools/editor-workbench-preview/package.json";
  let packageJson;
  try {
    packageJson = JSON.parse(await readFile(resolve(rootDir, filename), "utf8"));
  } catch (error) {
    failures.push(`${filename}: ${error.message}`);
    return;
  }

  const expectedScripts = {
    "design:verify": "node verify-designs.mjs",
    "design:verify:reference-negative": "node verify-reference-negative-guard.mjs",
  };
  for (const [script, command] of Object.entries(expectedScripts)) {
    if (packageJson.scripts?.[script] !== command) {
      failures.push(
        `${filename}: script ${script} must be ${JSON.stringify(command)}, got ${JSON.stringify(
          packageJson.scripts?.[script],
        )}`,
      );
    }
  }
}

async function validateOutputDirectory() {
  let files;
  try {
    files = await readdir(outputDir);
  } catch (error) {
    failures.push(`${OUTPUT_DIR}: ${error.message}`);
    return;
  }

  const unexpectedPngs = files
    .filter((file) => file.toLowerCase().endsWith(".png"))
    .filter((file) => !expectedSet.has(file));
  for (const filename of unexpectedPngs) {
    failures.push(`${filename}: unexpected PNG not listed in design manifest`);
  }
}

async function validateStyleNotes() {
  const filename = "STYLE-NOTES.md";
  const file = resolve(outputDir, filename);
  try {
    const info = await stat(file);
    if (info.size <= 0) {
      failures.push(`${filename}: file is empty`);
      return;
    }
    const content = await readFile(file, "utf8");
    for (const design of DESIGNS) {
      if (!content.includes(`\`${design.output}\``)) {
        failures.push(`${filename}: missing ${design.output}`);
      }
    }
    if (!content.includes("`preview-sheet.png`")) {
      failures.push(`${filename}: missing preview-sheet.png`);
    }
  } catch (error) {
    failures.push(`${filename}: ${error.message}`);
  }
}

async function validateDocumentationBaselines() {
  const docs = [
    {
      filename: ".codex/plans/Editor Workbench PNG Design Plan.md",
      required: [
        `${DESIGNS.length} design entries`,
        `${expected.length} PNG files`,
        "zircon_editor/assets/ui/editor/reference/workbench.png",
        "byte-identical",
        "direct byte comparison",
        "Negative guard",
        "design:verify:reference-negative",
        "ZIRCON_WORKBENCH_REFERENCE_NEGATIVE_GUARD=1",
        "package script definitions",
        "per-design PNG freshness",
        "world streaming pages",
        "LiveOps pages",
      ],
    },
    {
      filename: "docs/ui-and-layout/editor-workbench-design-export.md",
      required: [
        `contain ${DESIGNS.length} design entries`,
        `expects ${expected.length} PNG files`,
        "zircon_editor/assets/ui/editor/reference/workbench.png",
        "tools/editor-workbench-preview/package.json",
        "zero-drift byte match",
        "direct byte comparison",
        "negative guard",
        "design:verify:reference-negative",
        "guarded runtime-reference override behavior",
        "package script definitions",
        "exact pixel identity belongs to the pinned runtime reference asset",
        "Every manifest single-page PNG",
        "world streaming editor-page batch",
        "LiveOps editor-page batch",
      ],
    },
    {
      filename: "docs/ui-and-layout/index.md",
      required: [
        "World streaming pages",
        "world-partition-workbench.png",
        "LiveOps pages",
        "feature-flags-workbench.png",
        "zircon_editor/assets/ui/editor/reference/workbench.png",
        "tools/editor-workbench-preview/verify-reference-negative-guard.mjs",
        "npm --prefix tools/editor-workbench-preview run design:verify:reference-negative",
      ],
    },
    {
      filename: "docs/ui-and-layout/editor-workbench-designs/STYLE-NOTES.md",
      required: [
        "`preview-sheet.png`",
        "World streaming editor-page PNGs",
        "world-partition-workbench.png",
        "LiveOps editor-page PNGs",
        "feature-flags-workbench.png",
      ],
    },
  ];

  for (const doc of docs) {
    try {
      const content = await readFile(resolve(rootDir, doc.filename), "utf8");
      for (const marker of doc.required) {
        if (!content.includes(marker)) {
          failures.push(`${doc.filename}: missing documentation marker ${marker}`);
        }
      }
    } catch (error) {
      failures.push(`${doc.filename}: ${error.message}`);
    }
  }
}

function validateNoStalePreviewProcesses() {
  if (process.platform !== "win32") {
    return;
  }

  const result = spawnSync(
    "powershell.exe",
    [
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-Command",
      [
        "Get-CimInstance Win32_Process",
        "| Where-Object {",
        "($_.Name -match 'node|msedge|playwright') -and",
        "($_.CommandLine -match 'editor-workbench-preview|export-designs\\.mjs|server\\.mjs|design\\.html\\?design=') -and",
        "($_.ProcessId -ne $PID) -and",
        "($_.CommandLine -notmatch 'verify-designs\\.mjs')",
        "-and ($_.CommandLine -notmatch 'run design:verify')",
        "}",
        "| ForEach-Object { \"$($_.ProcessId) $($_.Name) $($_.CommandLine)\" }",
      ].join(" "),
    ],
    { cwd: rootDir, encoding: "utf8" },
  );

  if (result.error) {
    failures.push(`preview process scan failed: ${result.error.message}`);
    return;
  }
  if (result.status !== 0) {
    failures.push(`preview process scan exited with ${result.status}: ${result.stderr.trim()}`);
    return;
  }

  const staleProcesses = result.stdout
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  for (const line of staleProcesses) {
    failures.push(`stale preview process: ${line}`);
  }
}

async function validatePreviewSheetFreshness() {
  const previewSheet = await stat(resolve(outputDir, "preview-sheet.png"));
  const sources = [
    "tools/editor-workbench-preview/design.html",
    "tools/editor-workbench-preview/design.css",
    "tools/editor-workbench-preview/design.js",
    "tools/editor-workbench-preview/design-manifest.mjs",
    "tools/editor-workbench-preview/preview-sheet.js",
  ];

  for (const source of sources) {
    try {
      const sourceInfo = await stat(resolve(rootDir, source));
      if (previewSheet.mtimeMs + 1000 < sourceInfo.mtimeMs) {
        failures.push(`preview-sheet.png: older than ${source}`);
      }
    } catch (error) {
      failures.push(`${source}: ${error.message}`);
    }
  }
}

async function validateStyleNotesFreshness() {
  const styleNotes = await stat(resolve(outputDir, "STYLE-NOTES.md"));
  const sources = [
    "tools/editor-workbench-preview/design-manifest.mjs",
    "tools/editor-workbench-preview/export-designs.mjs",
  ];

  for (const source of sources) {
    try {
      const sourceInfo = await stat(resolve(rootDir, source));
      if (styleNotes.mtimeMs + 1000 < sourceInfo.mtimeMs) {
        failures.push(`STYLE-NOTES.md: older than ${source}`);
      }
    } catch (error) {
      failures.push(`${source}: ${error.message}`);
    }
  }
}

async function validateDesignPngFreshness() {
  const sources = [
    "tools/editor-workbench-preview/design.html",
    "tools/editor-workbench-preview/design.css",
    "tools/editor-workbench-preview/design.js",
    "tools/editor-workbench-preview/design-manifest.mjs",
  ];
  const sourceInfos = [];

  for (const source of sources) {
    try {
      sourceInfos.push([source, await stat(resolve(rootDir, source))]);
    } catch (error) {
      failures.push(`${source}: ${error.message}`);
    }
  }

  for (const design of DESIGNS) {
    let designInfo;
    try {
      designInfo = await stat(resolve(outputDir, design.output));
    } catch (error) {
      failures.push(`${design.output}: ${error.message}`);
      continue;
    }

    for (const [source, sourceInfo] of sourceInfos) {
      if (designInfo.mtimeMs + 1000 < sourceInfo.mtimeMs) {
        failures.push(`${design.output}: older than ${source}`);
      }
    }
  }
}

function duplicates(values) {
  const seen = new Set();
  const duplicateValues = new Set();
  for (const value of values) {
    if (seen.has(value)) duplicateValues.add(value);
    else seen.add(value);
  }
  return [...duplicateValues].sort();
}

function sameJson(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function readPngSize(buffer) {
  const png = parsePng(buffer);
  if (!png) return null;
  return {
    width: png.width,
    height: png.height,
  };
}

function validateVisualProfile(filename, buffer, size) {
  const metrics = readRgbPngVisualMetrics(buffer, size);
  if (!metrics) {
    failures.push(`${filename}: expected non-interlaced 8-bit RGB PNG content`);
    return;
  }

  if (metrics.averageLuma < 12 || metrics.averageLuma > 55) {
    failures.push(`${filename}: average luminance ${metrics.averageLuma.toFixed(2)} is outside workbench dark range`);
  }
  if (metrics.darkRatio < 0.82) {
    failures.push(`${filename}: dark pixel ratio ${metrics.darkRatio.toFixed(4)} is below workbench density floor`);
  }
  if (metrics.brightRatio > 0.10) {
    failures.push(`${filename}: bright pixel ratio ${metrics.brightRatio.toFixed(4)} is too high for dark chrome`);
  }
  if (metrics.tealRatio < 0.00008) {
    failures.push(`${filename}: teal accent ratio ${metrics.tealRatio.toFixed(6)} is too low`);
  }
  if (metrics.uniqueSampleColors < 32) {
    failures.push(`${filename}: sampled color variety ${metrics.uniqueSampleColors} is too low`);
  }
}

function readRgbPngVisualMetrics(buffer, size) {
  const png = parsePng(buffer);
  if (!png || png.bitDepth !== 8 || png.colorType !== 2 || png.interlace !== 0) {
    return null;
  }

  const bytesPerPixel = 3;
  const stride = size.width * bytesPerPixel;
  const data = inflateSync(png.imageData);
  let source = 0;
  let previousRow = Buffer.alloc(stride);
  let currentRow = Buffer.alloc(stride);
  let lumaSum = 0;
  let darkPixels = 0;
  let brightPixels = 0;
  let tealPixels = 0;
  let sampledPixels = 0;
  const uniqueSampleColors = new Set();

  for (let y = 0; y < size.height; y += 1) {
    const filter = data[source];
    source += 1;
    for (let i = 0; i < stride; i += 1) {
      const raw = data[source];
      source += 1;
      const left = i >= bytesPerPixel ? currentRow[i - bytesPerPixel] : 0;
      const up = previousRow[i];
      const upLeft = i >= bytesPerPixel ? previousRow[i - bytesPerPixel] : 0;
      currentRow[i] = reconstructPngByte(filter, raw, left, up, upLeft);
    }

    if (y % 4 === 0) {
      for (let x = 0; x < size.width; x += 4) {
        const index = x * bytesPerPixel;
        const red = currentRow[index];
        const green = currentRow[index + 1];
        const blue = currentRow[index + 2];
        const luma = 0.2126 * red + 0.7152 * green + 0.0722 * blue;
        lumaSum += luma;
        sampledPixels += 1;
        if (luma < 80) darkPixels += 1;
        if (luma > 170) brightPixels += 1;
        if (red < 100 && green >= 120 && blue >= 120 && green - red >= 35 && blue - red >= 35) {
          tealPixels += 1;
        }
        if (x % 20 === 0) {
          uniqueSampleColors.add((red << 16) | (green << 8) | blue);
        }
      }
    }

    [previousRow, currentRow] = [Buffer.from(currentRow), previousRow];
  }

  return {
    averageLuma: lumaSum / sampledPixels,
    darkRatio: darkPixels / sampledPixels,
    brightRatio: brightPixels / sampledPixels,
    tealRatio: tealPixels / sampledPixels,
    uniqueSampleColors: uniqueSampleColors.size,
  };
}

function parsePng(buffer) {
  const signature = "89504e470d0a1a0a";
  if (buffer.subarray(0, 8).toString("hex") !== signature) return null;
  const idatChunks = [];
  const header = {};
  let offset = 8;
  while (offset < buffer.length) {
    const length = buffer.readUInt32BE(offset);
    const type = buffer.subarray(offset + 4, offset + 8).toString("ascii");
    const data = buffer.subarray(offset + 8, offset + 8 + length);
    if (type === "IHDR") {
      header.width = data.readUInt32BE(0);
      header.height = data.readUInt32BE(4);
      header.bitDepth = data[8];
      header.colorType = data[9];
      header.interlace = data[12];
    } else if (type === "IDAT") {
      idatChunks.push(data);
    }
    offset += 12 + length;
    if (type === "IEND") break;
  }
  return {
    ...header,
    imageData: Buffer.concat(idatChunks),
  };
}

function reconstructPngByte(filter, raw, left, up, upLeft) {
  if (filter === 0) return raw;
  if (filter === 1) return (raw + left) & 255;
  if (filter === 2) return (raw + up) & 255;
  if (filter === 3) return (raw + Math.floor((left + up) / 2)) & 255;
  if (filter === 4) return (raw + paeth(left, up, upLeft)) & 255;
  throw new Error(`unsupported PNG filter ${filter}`);
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
