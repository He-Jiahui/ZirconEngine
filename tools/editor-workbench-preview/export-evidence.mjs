import { createHash } from "node:crypto";
import { readdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

export const EXPORT_EVIDENCE_FILENAME = "EXPORT-EVIDENCE.json";
export const SCREENSHOT_CHANNEL = "msedge";
export const SCREENSHOT_SELECTOR = "#design-root > *";
export const SCREENSHOT_WAIT_MS = 250;
export const SCREENSHOT_TIMEOUT_MS = 120000;

const SOURCE_FILES = [
  "tools/editor-workbench-preview/design.html",
  "tools/editor-workbench-preview/design.css",
  "tools/editor-workbench-preview/design.js",
  "tools/editor-workbench-preview/design-manifest.mjs",
  "tools/editor-workbench-preview/export-designs.mjs",
  "tools/editor-workbench-preview/export-evidence.mjs",
  "tools/editor-workbench-preview/export-options.mjs",
  "tools/editor-workbench-preview/package-lock.json",
  "tools/editor-workbench-preview/package.json",
  "tools/editor-workbench-preview/preview-sheet.js",
  "tools/editor-workbench-preview/server.mjs",
];

const SOURCE_DIRECTORIES = [
  "zircon_editor/assets/icons/ionicons",
  "zircon_editor/fixtures/workbench",
];

export async function buildExportEvidence({ rootDir, outputDir, outputNames, width, height }) {
  const sourcePaths = [...SOURCE_FILES];
  for (const directory of SOURCE_DIRECTORIES) {
    sourcePaths.push(...(await listFiles(rootDir, directory)));
  }
  sourcePaths.sort();

  const outputs = await Promise.all(
    [...outputNames].sort().map((filename) => hashEntry(outputDir, filename)),
  );

  return {
    schemaVersion: 1,
    capture: {
      channel: SCREENSHOT_CHANNEL,
      height,
      selector: SCREENSHOT_SELECTOR,
      timeoutMs: SCREENSHOT_TIMEOUT_MS,
      waitMs: SCREENSHOT_WAIT_MS,
      width,
    },
    sources: await Promise.all(sourcePaths.map((path) => hashEntry(rootDir, path))),
    outputs,
    styleNotes: await hashEntry(outputDir, "STYLE-NOTES.md"),
  };
}

export async function writeExportEvidence(options) {
  const evidence = await buildExportEvidence(options);
  await writeFile(
    resolve(options.outputDir, EXPORT_EVIDENCE_FILENAME),
    `${JSON.stringify(evidence, null, 2)}\n`,
    "utf8",
  );
}

async function listFiles(rootDir, directory) {
  const entries = await readdir(resolve(rootDir, directory), { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const path = `${directory}/${entry.name}`;
    if (entry.isDirectory()) {
      files.push(...(await listFiles(rootDir, path)));
    } else if (entry.isFile()) {
      files.push(path);
    }
  }
  return files;
}

async function hashEntry(baseDir, path) {
  const buffer = await readFile(resolve(baseDir, path));
  const canonicalBuffer = isTextPath(path)
    ? Buffer.from(buffer.toString("utf8").replace(/\r\n?/gu, "\n"), "utf8")
    : buffer;
  return {
    path,
    bytes: canonicalBuffer.length,
    sha256: createHash("sha256").update(canonicalBuffer).digest("hex").toUpperCase(),
  };
}

function isTextPath(path) {
  return /\.(?:css|html|js|json|md|mjs|svg)$/iu.test(path);
}
