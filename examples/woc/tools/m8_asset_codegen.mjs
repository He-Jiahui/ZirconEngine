import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const toolDir = dirname(fileURLToPath(import.meta.url));
const wocRoot = resolve(toolDir, "..");
const repoRoot = resolve(wocRoot, "..", "..");
const referenceRoot = resolve(repoRoot, "dev", "world-of-claudecraft");
const selectionPath = resolve(wocRoot, "contracts", "m8_asset_selection.json");
const catalogPath = resolve(wocRoot, "reference", "asset_catalog.json");
const manifestPath = resolve(wocRoot, "contracts", "m8_assets.json");
const checkOnly = process.argv.includes("--check");

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function normalizePath(value, label) {
  if (typeof value !== "string" || value.length === 0 || value.includes("\\")) {
    throw new Error(`${label} must be a non-empty forward-slash path`);
  }
  const segments = value.split("/");
  if (segments.some((segment) => segment === "" || segment === "." || segment === "..")) {
    throw new Error(`${label} contains an invalid path segment: ${value}`);
  }
  return value;
}

function outputFor(sourcePath, explicitOutput) {
  if (explicitOutput !== undefined) {
    return normalizePath(explicitOutput, "output_path");
  }
  if (!sourcePath.startsWith("public/")) {
    throw new Error(`non-public asset requires output_path: ${sourcePath}`);
  }
  return `assets/m8/${sourcePath.slice("public/".length)}`;
}

function readBlob(commit, sourcePath) {
  const result = spawnSync(
    "git",
    ["-C", referenceRoot, "show", `${commit}:${sourcePath}`],
    { encoding: null, maxBuffer: 32 * 1024 * 1024 },
  );
  if (result.status !== 0) {
    const detail = result.stderr?.toString("utf8").trim() || `exit ${result.status}`;
    throw new Error(`cannot read pinned blob ${sourcePath}: ${detail}`);
  }
  return result.stdout;
}

function flattenSelection(selection) {
  const licenseIds = new Set(selection.licenses.map((license) => license.id));
  if (licenseIds.size !== selection.licenses.length) {
    throw new Error("license ids must be unique");
  }
  const entries = [];
  const sourcePaths = new Set();
  const outputPaths = new Set();
  for (const group of selection.groups) {
    if (!licenseIds.has(group.license_id)) {
      throw new Error(`unknown license id ${group.license_id}`);
    }
    if (typeof group.role !== "string" || group.role.length === 0) {
      throw new Error("asset group role must be non-empty");
    }
    for (const item of group.paths) {
      const sourcePath = normalizePath(
        typeof item === "string" ? item : item.source_path,
        "source_path",
      );
      const outputPath = outputFor(
        sourcePath,
        typeof item === "string" ? undefined : item.output_path,
      );
      if (!outputPath.startsWith("assets/m8/")) {
        throw new Error(`asset output escapes assets/m8: ${outputPath}`);
      }
      if (sourcePaths.has(sourcePath)) throw new Error(`duplicate source path ${sourcePath}`);
      if (outputPaths.has(outputPath)) throw new Error(`duplicate output path ${outputPath}`);
      sourcePaths.add(sourcePath);
      outputPaths.add(outputPath);
      entries.push({
        role: group.role,
        license_id: group.license_id,
        source_path: sourcePath,
        output_path: outputPath,
      });
    }
  }
  return entries;
}

function stableCounts(entries, field) {
  const counts = {};
  for (const entry of entries) counts[entry[field]] = (counts[entry[field]] ?? 0) + 1;
  return counts;
}

async function ensureBytes(path, expected, outputPath) {
  if (checkOnly) {
    let actual;
    try {
      actual = await readFile(path);
    } catch {
      throw new Error(`materialized asset missing: ${outputPath}`);
    }
    if (!actual.equals(expected)) throw new Error(`materialized asset drift: ${outputPath}`);
    return;
  }
  let current = null;
  try {
    current = await readFile(path);
  } catch {
    current = null;
  }
  if (current?.equals(expected)) return;
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, expected);
}

const selectionBytes = await readFile(selectionPath);
const selection = JSON.parse(selectionBytes.toString("utf8"));
const catalog = JSON.parse(await readFile(catalogPath, "utf8"));
if (selection.schema_version !== 1) throw new Error("unsupported M8 asset selection schema");
if (selection.source_commit !== catalog.source_commit) {
  throw new Error("M8 asset selection commit does not match the pinned asset catalog");
}
const selected = flattenSelection(selection);
const glbCatalog = new Map(catalog.entries.map((entry) => [entry.path, entry]));
const generatedEntries = [];
let totalBytes = 0;
let animations = 0;
let skins = 0;
let glbs = 0;

for (const entry of selected) {
  const bytes = readBlob(selection.source_commit, entry.source_path);
  const digest = sha256(bytes);
  const generated = {
    role: entry.role,
    license_id: entry.license_id,
    source_path: entry.source_path,
    asset_path: entry.output_path,
    byte_length: bytes.length,
    sha256: digest,
  };
  if (entry.source_path.endsWith(".glb")) {
    const pinned = glbCatalog.get(entry.source_path);
    if (!pinned) throw new Error(`GLB missing from pinned asset catalog: ${entry.source_path}`);
    if (pinned.sha256 !== digest || pinned.byte_length !== bytes.length) {
      throw new Error(`GLB identity differs from pinned asset catalog: ${entry.source_path}`);
    }
    generated.gltf = {
      version: pinned.gltf_version,
      extensions_used: pinned.extensions_used,
      extensions_required: pinned.extensions_required,
      animation_count: pinned.animation_count,
      skin_count: pinned.skin_count,
    };
    glbs += 1;
    animations += pinned.animation_count;
    skins += pinned.skin_count;
  }
  totalBytes += bytes.length;
  generatedEntries.push(generated);
  const absoluteOutput = resolve(wocRoot, ...entry.output_path.split("/"));
  const m8Root = `${resolve(wocRoot, "assets", "m8")}${sep}`;
  if (!absoluteOutput.startsWith(m8Root)) {
    throw new Error(`resolved output escapes assets/m8: ${entry.output_path}`);
  }
  await ensureBytes(absoluteOutput, bytes, entry.output_path);
}

const manifest = {
  schema_version: 1,
  source_commit: selection.source_commit,
  selection_sha256: sha256(selectionBytes),
  entries: generatedEntries,
  licenses: selection.licenses,
  totals: {
    files: generatedEntries.length,
    bytes: totalBytes,
    glbs,
    animations,
    skins,
    by_role: stableCounts(generatedEntries, "role"),
    by_license: stableCounts(generatedEntries, "license_id"),
  },
};
const manifestBytes = Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`, "utf8");

if (checkOnly) {
  let current;
  try {
    current = await readFile(manifestPath);
  } catch {
    throw new Error("generated M8 asset manifest is missing");
  }
  if (!current.equals(manifestBytes)) throw new Error("generated M8 asset manifest is stale");
} else {
  await writeFile(manifestPath, manifestBytes);
}

console.log(
  JSON.stringify({
    mode: checkOnly ? "check" : "materialize",
    files: manifest.totals.files,
    bytes: manifest.totals.bytes,
    glbs: manifest.totals.glbs,
    animations: manifest.totals.animations,
    skins: manifest.totals.skins,
    selection_sha256: manifest.selection_sha256,
  }),
);
