import { record, uuid } from "./protocol";

const hexDigest = /^[0-9a-f]{64}$/;
const maxBlobBytes = 16 * 1024 * 1024;
const maxProjectBytes = 512 * 1024 * 1024;
const invalidComponents = new Set([".git", ".hg", ".svn", ".ssh", ".aws", ".azure", ".gnupg", ".codex", ".env", "target", "build", "binaries", "generated", "node_modules", "cache", "deriveddatacache", "intermediate", "saved", "crashdumps", "credentials", "secrets"]);
const reservedStems = /^(con|prn|aux|nul|com[1-9]|lpt[1-9])$/;
const privateSuffix = /\.(pem|key|pfx|p12|keystore)$/;

export interface CloudFile { path: string; digest: string; bytes: number }
export interface CloudManifest {
  schemaVersion: 1;
  engine: string;
  packageLockDigest: string;
  ignorePolicy: "zircon-project-v1";
  sourceRevision: string | null;
  files: CloudFile[];
}
export interface CloudSnapshot {
  organizationId: string;
  projectId: string;
  baseRevision: string;
  revision: string;
  manifestDigest: string;
  createdAt: number;
  createdBy: string;
  manifest: CloudManifest;
}

function fail(): never { throw new Error("account_protocol_invalid"); }
function shape(value: Record<string, unknown>, names: string[]) {
  if (Object.keys(value).length !== names.length || names.some(name => !Object.prototype.hasOwnProperty.call(value, name))) fail();
}
function digest(value: unknown): asserts value is string {
  if (typeof value !== "string" || !hexDigest.test(value)) fail();
}
function revision(value: unknown): asserts value is string {
  if (typeof value !== "string" || !/^(0|[1-9][0-9]*)$/.test(value) || value.length > 19 || BigInt(value) > 9223372036854775807n) fail();
}
function cleanText(value: unknown, maxBytes: number): asserts value is string {
  if (typeof value !== "string" || value.length === 0 || new TextEncoder().encode(value).length > maxBytes || /[\x00-\x1f\x7f-\x9f]/.test(value)) fail();
}
function safeInteger(value: unknown, max: number): asserts value is number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0 || value > max) fail();
}
function filePath(value: unknown): asserts value is string {
  cleanText(value, 512);
  if (/[\\:<>"|?*]/.test(value)) fail();
  for (const component of value.split("/")) {
    const lower = component.toLowerCase();
    if (!component || component === "." || component === ".." || /[. ]$/.test(component)
      || invalidComponents.has(lower) || lower.startsWith(".env.") || privateSuffix.test(lower)
      || reservedStems.test(lower.split(".", 1)[0])) fail();
  }
}

// Read-only projection only. The eventual apply owner must recheck the manifest
// and downloaded bytes before using any remote path as a local filesystem path.
export function parseCloudHead(value: unknown, organization: string, project: string): CloudSnapshot | null {
  uuid(organization); uuid(project);
  if (value === null) return null;
  const head = record(value);
  shape(head, ["organizationId", "projectId", "baseRevision", "revision", "manifestDigest", "createdAt", "createdBy", "manifest"]);
  uuid(head.organizationId); uuid(head.projectId);
  if (head.organizationId !== organization || head.projectId !== project) fail();
  revision(head.baseRevision); revision(head.revision);
  if (BigInt(head.revision) !== BigInt(head.baseRevision) + 1n) fail();
  digest(head.manifestDigest); digest(head.createdBy);
  safeInteger(head.createdAt, Number.MAX_SAFE_INTEGER);
  if (head.createdAt === 0) fail();

  const manifest = record(head.manifest);
  shape(manifest, ["schemaVersion", "engine", "packageLockDigest", "ignorePolicy", "sourceRevision", "files"]);
  if (manifest.schemaVersion !== 1 || manifest.ignorePolicy !== "zircon-project-v1") fail();
  cleanText(manifest.engine, 256); digest(manifest.packageLockDigest);
  if (manifest.sourceRevision !== null) cleanText(manifest.sourceRevision, 256);
  if (!Array.isArray(manifest.files) || manifest.files.length > 10000) fail();
  const paths = new Set<string>();
  let total = 0;
  const files: CloudFile[] = manifest.files.map(value => {
    const row = record(value);
    shape(row, ["path", "digest", "bytes"]);
    filePath(row.path); digest(row.digest);
    safeInteger(row.bytes, maxBlobBytes);
    const key = row.path.toLowerCase();
    if (paths.has(key)) fail();
    paths.add(key);
    total += row.bytes;
    if (total > maxProjectBytes) fail();
    return { path: row.path, digest: row.digest, bytes: row.bytes };
  });
  for (const path of paths) {
    for (let cursor = path.indexOf("/"); cursor !== -1; cursor = path.indexOf("/", cursor + 1)) {
      if (paths.has(path.slice(0, cursor))) fail();
    }
  }
  return { organizationId: head.organizationId, projectId: head.projectId, baseRevision: head.baseRevision,
    revision: head.revision, manifestDigest: head.manifestDigest, createdAt: head.createdAt, createdBy: head.createdBy,
    manifest: { schemaVersion: 1, engine: manifest.engine, packageLockDigest: manifest.packageLockDigest,
      ignorePolicy: "zircon-project-v1", sourceRevision: manifest.sourceRevision, files } };
}
