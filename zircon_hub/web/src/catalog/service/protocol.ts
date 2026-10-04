import { decimal, emptyResource, record, uuid, type Page } from "../../account/protocol";

export type CatalogAction =
  | { action: "catalog"; backendEpoch: string; generation: string; after?: string; query?: string }
  | { action: "catalog-entitlements"; backendEpoch: string; generation: string; organization: string; after?: string }
  | { action: "package-inventory"; schemaVersion: 2; targetMode: PackageRuntimeMode; backendEpoch: string; generation: string; organization: string }
  | { action: "catalog-license"; backendEpoch: string; generation: string; organization: string; operationId: string; expectedPolicyRevision: string; packageId: string; revision: string; licenseId: string }
  | { action: "package-install"; schemaVersion: 2; targetMode: PackageRuntimeMode; backendEpoch: string; generation: string; organization: string; operationId: string; packageId: string; revision: string; expectedInventoryRevision: string };

export type PackageRuntimeMode = "editor_host" | "client_runtime";

export interface CatalogRelease {
  iss: string; aud: string; sub: string; exp: number;
  package_id: string; revision: string; version: string; name: string; kind: "plugin" | "asset";
  description: string; license_id: string; license_text: string; artifact_digest: string; artifact_size: number;
}
export interface CatalogEntitlement { packageId: string; revision: string; licenseId: string }
export interface InstalledPackage { operationId: string; packageId: string; version: string; releaseRevision: string; artifactDigest: string }
export interface PackageInventory { schemaVersion: 2; targetMode: PackageRuntimeMode; revision: string; packages: InstalledPackage[] }
export interface InstallReceipt { schemaVersion: 2; targetMode: PackageRuntimeMode; operationId: string; inventoryRevision: string; package: InstalledPackage }
export interface InventoryResource { data: PackageInventory | null; targetMode: PackageRuntimeMode | null; loading: boolean; error: string | null }

export const emptyInventory = (): InventoryResource => ({ data: null, targetMode: null, loading: false, error: null });
export const catalogResources = () => ({ serviceCatalog: emptyResource<CatalogRelease>(), catalogQuery: "", catalogEntitlements: emptyResource<CatalogEntitlement>(), packageInventory: emptyInventory() });

export function parseCatalogPage(value: unknown): Page<CatalogRelease> {
  const source = record(value);
  const items = pageItems(source).map((value): CatalogRelease => {
    const row = record(value);
    uuid(row.package_id); positiveRevision(row.revision);
    const kind = row.kind;
    if (kind !== "plugin" && kind !== "asset") invalid();
    return {
      iss: text(row.iss), aud: text(row.aud), sub: text(row.sub), exp: integer(row.exp),
      package_id: row.package_id, revision: row.revision, version: text(row.version, 64), name: text(row.name, 256), kind,
      description: text(row.description, 8192, true), license_id: text(row.license_id, 128), license_text: text(row.license_text, 8192, true),
      artifact_digest: digest(row.artifact_digest), artifact_size: integer(row.artifact_size, 1, 1073741824),
    };
  });
  if (source.nextCursor !== null) uuid(source.nextCursor);
  unique(items.map(item => item.package_id));
  // Verification can filter every row while the database still has another page.
  return { items, nextCursor: source.nextCursor as string | null };
}

export function parseEntitlements(value: unknown): Page<CatalogEntitlement> {
  const source = record(value);
  const items = pageItems(source).map(value => {
    const row = record(value);
    uuid(row.packageId); positiveRevision(row.revision);
    return { packageId: row.packageId, revision: row.revision, licenseId: text(row.licenseId, 128) };
  });
  if (source.nextCursor !== null) { positiveRevision(source.nextCursor); if (items.length === 0) invalid(); }
  unique(items.map(entitlementKey));
  return { items, nextCursor: source.nextCursor as string | null };
}

export function parseInventory(value: unknown, expectedTargetMode?: PackageRuntimeMode): PackageInventory {
  const row = record(value);
  if (row.schemaVersion !== 2 || !isPackageRuntimeMode(row.targetMode) || (expectedTargetMode !== undefined && row.targetMode !== expectedTargetMode) || !Array.isArray(row.packages) || row.packages.length > 128) invalid();
  decimal(row.revision);
  const packages = row.packages.map(parseInstalledPackage);
  unique(packages.map(item => item.packageId));
  return { schemaVersion: 2, targetMode: row.targetMode, revision: row.revision, packages };
}

export function parseInstallReceipt(value: unknown, expectedTargetMode: PackageRuntimeMode): InstallReceipt {
  const row = record(value);
  if (row.schemaVersion !== 2 || !isPackageRuntimeMode(row.targetMode) || row.targetMode !== expectedTargetMode) invalid();
  uuid(row.operationId); positiveRevision(row.inventoryRevision);
  const installed = parseInstalledPackage(row.package);
  if (installed.operationId !== row.operationId) invalid();
  return { schemaVersion: 2, targetMode: row.targetMode, operationId: row.operationId, inventoryRevision: row.inventoryRevision, package: installed };
}

export function isInstalled(release: CatalogRelease, inventory: PackageInventory | null, targetMode: PackageRuntimeMode) {
  if (!inventory || inventory.targetMode !== targetMode) return false;
  return inventory.packages.some(item => item.packageId === release.package_id && item.releaseRevision === release.revision && item.artifactDigest === release.artifact_digest);
}

export const MAX_INSTALL_PACKAGE_BYTES = 16 * 1024 * 1024;
export const supportsPackageInstall = (release: CatalogRelease) => release.kind === "plugin" && release.artifact_size <= MAX_INSTALL_PACKAGE_BYTES;

export const entitlementKey = (item: CatalogEntitlement) => `${item.packageId}:${item.revision}`;
function parseInstalledPackage(value: unknown): InstalledPackage {
  const row = record(value);
  uuid(row.operationId); uuid(row.packageId); positiveRevision(row.releaseRevision);
  return { operationId: row.operationId, packageId: row.packageId, version: text(row.version, 64), releaseRevision: row.releaseRevision, artifactDigest: digest(row.artifactDigest) };
}
function pageItems(row: Record<string, unknown>): unknown[] { if (!Array.isArray(row.items) || row.items.length > 100) invalid(); return row.items; }
function unique(values: string[]) { if (new Set(values).size !== values.length) invalid(); }
function positiveRevision(value: unknown): asserts value is string { decimal(value); if (value === "0") invalid(); }
function digest(value: unknown): string { if (typeof value !== "string" || !/^[0-9a-f]{64}$/.test(value)) invalid(); return value; }
function isPackageRuntimeMode(value: unknown): value is PackageRuntimeMode { return value === "editor_host" || value === "client_runtime"; }
function integer(value: unknown, min = 0, max = Number.MAX_SAFE_INTEGER): number { if (typeof value !== "number" || !Number.isSafeInteger(value) || value < min || value > max) invalid(); return value; }
function text(value: unknown, max = 8192, empty = false): string { if (typeof value !== "string" || (!empty && !value) || new TextEncoder().encode(value).length > max) invalid(); return value; }
function invalid(): never { throw new Error("account_protocol_invalid"); }
