import type { CatalogAction, PackageRuntimeMode } from "../catalog/service/protocol";

export type AccountStatus = "unavailable" | "signed-out" | "signed-in";

export interface AccountView {
  configured: boolean;
  status: AccountStatus;
  issuer: string | null;
  subject: string | null;
  displayName: string | null;
  generation: string;
  error: string | null;
}

export interface AccountSnapshot {
  backendEpoch: string;
  account: AccountView;
  data: unknown | null;
  error: string | null;
  operations: OperationSummary[];
  operationsRevision: string;
  operationsError: string | null;
}

export interface OperationSummary {
  operationId: string;
  status: "unknown" | "committed" | "failed";
  action: "create-organization" | "mutate" | "catalog-license" | "package-install" | "cloud-commit";
  organizationId: string | null;
  error: string | null;
  targetMode?: PackageRuntimeMode;
}

export interface CloudBinding {
  localProjectGuid: string;
  organizationId: string;
  projectId: string;
}

export type AuthAction = "sign-in" | "refresh" | "logout" | "cancel";
export type QueryAction = "organizations" | "invitations" | "members" | "projects" | "issued-invitations";
export type Role = "owner" | "admin" | "member" | "viewer";
export type MemberRole = Exclude<Role, "owner">;
export type OrganizationMutation =
  | { action: "create-project"; name: string }
  | { action: "set-member"; issuer: string; subject: string; role: MemberRole; active: boolean }
  | { action: "transfer-ownership"; issuer: string; subject: string }
  | { action: "invite"; issuer: string; subject: string; role: MemberRole; expires_at: number }
  | { action: "accept-invite" | "revoke-invite"; invitation_id: string };

export type AccountAction =
  | CatalogAction
  | { action: AuthAction; backendEpoch: string }
  | { action: "organizations" | "invitations"; backendEpoch: string; generation: string; after?: string }
  | { action: "members" | "projects" | "issued-invitations"; backendEpoch: string; generation: string; organization: string; after?: string }
  | { action: "cloud-head"; backendEpoch: string; generation: string; organization: string; project: string }
  | { action: "cloud-push"; backendEpoch: string; generation: string; operationId: string; baseRevision: string }
  | { action: "cloud-resume-push"; backendEpoch: string; generation: string; operationId: string }
  | { action: "cloud-stage-download"; backendEpoch: string; generation: string; expectedRevision: string }
  | { action: "cloud-apply-download"; backendEpoch: string; generation: string; stageId: string; expectedRevision: string }
  | { action: "cloud-discard-download"; backendEpoch: string; generation: string; expectedRevision: string }
  | { action: "cloud-binding"; backendEpoch: string; generation: string }
  | { action: "attach-cloud-project"; backendEpoch: string; generation: string; organization: string; project: string; selectedProjectId: string }
  | { action: "create-organization"; backendEpoch: string; generation: string; operationId: string; name: string }
  | { action: "mutate"; backendEpoch: string; generation: string; organization: string; operationId: string; expectedPolicyRevision: string; mutation: OrganizationMutation }
  | { action: "reconcile" | "retry" | "acknowledge"; backendEpoch: string; generation: string; operationId: string };

export interface Organization { id: string; name: string; policyRevision: string }
export interface Member { issuer: string; subject: string; role: Role; active: boolean }
export interface Project { id: string; name: string }
export interface Invitation {
  id: string;
  organizationId: string;
  organizationName: string;
  policyRevision: string;
  role: Role;
  expiresAt: number;
  status: "pending" | "accepted" | "revoked";
}
export interface IssuedInvitation {
  id: string;
  organizationId: string;
  policyRevision: string;
  targetIssuer: string;
  targetSubject: string;
  role: Role;
  expiresAt: number;
  status: "pending" | "accepted" | "revoked" | "expired";
}
export interface Page<T> { items: T[]; nextCursor: string | null }
export interface Resource<T> extends Page<T> { loading: boolean; loaded: boolean; error: string | null }

export const unavailableSnapshot = (backendEpoch: string): AccountSnapshot => ({
  backendEpoch,
  account: { configured: false, status: "unavailable", issuer: null, subject: null, displayName: null, generation: "0", error: null },
  data: null,
  error: null,
  operations: [],
  operationsRevision: "0",
  operationsError: null,
});

export const emptyResource = <T>(): Resource<T> => ({ items: [], nextCursor: null, loading: false, loaded: false, error: null });

export function parseCloudBinding(value: unknown): CloudBinding | null {
  if (value === null) return null;
  const binding = record(value);
  if (Object.keys(binding).length !== 3 || !["localProjectGuid", "organizationId", "projectId"].every(key => Object.prototype.hasOwnProperty.call(binding, key))) fail();
  uuid(binding.localProjectGuid); uuid(binding.organizationId); uuid(binding.projectId);
  return binding as unknown as CloudBinding;
}

export function assertSnapshot(value: unknown): asserts value is AccountSnapshot {
  const source = record(value);
  text(source.backendEpoch);
  if (!("data" in source)) fail();
  nullableText(source.error);
  decimal(source.operationsRevision);
  nullableText(source.operationsError);
    if (!Array.isArray(source.operations) || source.operations.length > 128) fail();
    const ids = source.operations.map(item => {
      const operation = record(item);
      uuid(operation.operationId);
      nullableText(operation.error);
      if (!["unknown", "committed", "failed"].includes(String(operation.status))) fail();
      if (["mutate", "catalog-license", "package-install", "cloud-commit"].includes(String(operation.action))) uuid(operation.organizationId);
      else if (operation.action !== "create-organization" || operation.organizationId !== null) fail();
      if ("targetMode" in operation && (operation.action !== "package-install" || !["editor_host", "client_runtime"].includes(String(operation.targetMode)))) fail();
      if (operation.action === "cloud-commit" && (
      (operation.status === "unknown" && operation.error !== "account_service_outcome_unknown")
      || (operation.status === "committed" && operation.error !== null)
      || (operation.status === "failed" && !["account_cloud_conflict", "account_operation_id_conflict", "account_service_operation_failed"].includes(String(operation.error)))
    )) fail();
    return operation.operationId;
  });
  if (new Set(ids).size !== ids.length) fail();
  const account = record(source.account);
  if (typeof account.configured !== "boolean" || !["unavailable", "signed-out", "signed-in"].includes(String(account.status))) fail();
  decimal(account.generation);
  nullableText(account.issuer);
  nullableText(account.subject);
  nullableText(account.displayName);
  nullableText(account.error);
  if (account.status === "signed-in") {
    if (!account.configured) fail();
    text(account.issuer); text(account.subject);
  } else if (account.issuer !== null || account.subject !== null || account.displayName !== null
    || source.operations.length !== 0 || source.operationsError !== null || source.operationsRevision !== "0") fail();
  if (!account.configured && account.status !== "unavailable") fail();
}

export function parsePage<K extends QueryAction>(kind: K, value: unknown): Page<ResourceItem<K>> {
  const source = record(value);
  if (!Array.isArray(source.items) || source.items.length > 100) fail();
  nullableText(source.nextCursor);
  const items = source.items.map(item => {
    const row = record(item);
    if (kind === "members") {
      text(row.issuer); text(row.subject); role(row.role);
      if (typeof row.active !== "boolean") fail();
      return { issuer: row.issuer, subject: row.subject, role: row.role, active: row.active };
    }
    uuid(row.id);
    if (kind === "issued-invitations") {
      uuid(row.organizationId); decimal(row.policyRevision); text(row.targetIssuer); text(row.targetSubject); role(row.role);
      if (!Number.isSafeInteger(row.expiresAt) || Number(row.expiresAt) < 0 || !["pending", "accepted", "revoked", "expired"].includes(String(row.status))) fail();
      return { id: row.id, organizationId: row.organizationId, policyRevision: row.policyRevision, targetIssuer: row.targetIssuer, targetSubject: row.targetSubject, role: row.role, expiresAt: row.expiresAt, status: row.status };
    }
    if (kind === "invitations") {
      uuid(row.organizationId); text(row.organizationName); decimal(row.policyRevision); role(row.role);
      if (!Number.isSafeInteger(row.expiresAt) || Number(row.expiresAt) < 0 || !["pending", "accepted", "revoked"].includes(String(row.status))) fail();
      return { id: row.id, organizationId: row.organizationId, organizationName: row.organizationName, policyRevision: row.policyRevision, role: row.role, expiresAt: row.expiresAt, status: row.status };
    }
    text(row.name);
    if (kind === "organizations") { decimal(row.policyRevision); return { id: row.id, name: row.name, policyRevision: row.policyRevision }; }
    return { id: row.id, name: row.name };
  }) as ResourceItem<K>[];
  if (source.nextCursor !== null) {
    if (kind === "members") decimal(source.nextCursor); else uuid(source.nextCursor);
    if (items.length === 0) fail();
  }
  const keys = items.map(item => resourceKey(kind, item));
  if (new Set(keys).size !== keys.length) fail();
  return { items, nextCursor: source.nextCursor as string | null };
}

export type ResourceItem<K extends QueryAction> = K extends "organizations" ? Organization : K extends "invitations" ? Invitation : K extends "issued-invitations" ? IssuedInvitation : K extends "members" ? Member : Project;

export function resourceKey(kind: QueryAction, item: Organization | Invitation | IssuedInvitation | Member | Project): string {
  return kind === "members" ? JSON.stringify([(item as Member).issuer, (item as Member).subject]) : (item as Project).id;
}

export type CloudCommitReconciliation =
  | { status: "unknown" }
  | { status: "committed"; result: { status: "committed"; snapshot: {
    organizationId: string; projectId: string; baseRevision: string; revision: string; manifestDigest: string;
  } } }
  | { status: "conflict"; result: {
    status: "conflict"; organizationId: string; projectId: string; baseRevision: string; currentRevision: string; manifestDigest: string;
  } };

// The native broker binds the receipt to the journaled manifest digest and
// selected service identity. Web checks the projected identity and revision
// before displaying a terminal result; it never supplies a publish manifest.
export function parseCloudCommitReconciliation(value: unknown, operation: OperationSummary, operationId: string): CloudCommitReconciliation {
  uuid(operationId);
  if (operation.action !== "cloud-commit" || operation.operationId !== operationId) fail();
  uuid(operation.organizationId);
  const envelope = record(value);
  if (envelope.status === "unknown") {
    exactFields(envelope, ["status"]);
    if (operation.status !== "unknown" || operation.error !== "account_service_outcome_unknown") fail();
    return { status: "unknown" };
  }
  exactFields(envelope, ["status", "result"]);
  const result = record(envelope.result);
  if (envelope.status === "committed") {
    if (operation.status !== "committed" || operation.error !== null || result.status !== "committed") fail();
    exactFields(result, ["status", "snapshot"]);
    const snapshot = record(result.snapshot);
    exactFields(snapshot, ["organizationId", "projectId", "baseRevision", "revision", "manifestDigest"]);
    cloudReceiptScope(snapshot, operation.organizationId);
    cloudRevision(snapshot.baseRevision); cloudRevision(snapshot.revision);
    if (BigInt(snapshot.revision) !== BigInt(snapshot.baseRevision) + 1n) fail();
    return value as CloudCommitReconciliation;
  }
  if (envelope.status === "conflict") {
    if (operation.status !== "failed" || operation.error !== "account_cloud_conflict" || result.status !== "conflict") fail();
    exactFields(result, ["status", "organizationId", "projectId", "baseRevision", "currentRevision", "manifestDigest"]);
    cloudReceiptScope(result, operation.organizationId);
    cloudRevision(result.baseRevision); cloudRevision(result.currentRevision);
    if (result.currentRevision === result.baseRevision) fail();
    return value as CloudCommitReconciliation;
  }
  fail();
}

function exactFields(value: Record<string, unknown>, fields: string[]) {
  if (Object.keys(value).length !== fields.length || fields.some(field => !Object.prototype.hasOwnProperty.call(value, field))) fail();
}

function cloudReceiptScope(value: Record<string, unknown>, organizationId: string) {
  uuid(value.organizationId); uuid(value.projectId);
  if (value.organizationId !== organizationId || typeof value.manifestDigest !== "string" || !/^[0-9a-f]{64}$/.test(value.manifestDigest)) fail();
}

function cloudRevision(value: unknown): asserts value is string {
  if (typeof value !== "string" || !/^(0|[1-9][0-9]*)$/.test(value) || value.length > 19 || BigInt(value) > 9223372036854775807n) fail();
}

export function record(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) fail();
  return value as Record<string, unknown>;
}

export function decimal(value: unknown): asserts value is string {
  if (typeof value !== "string" || !/^(0|[1-9][0-9]*)$/.test(value) || value.length > 20 || BigInt(value) > 18446744073709551615n) fail();
}

export function uuid(value: unknown): asserts value is string {
  if (typeof value !== "string" || !/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(value)) fail();
}

function role(value: unknown) { if (!["owner", "admin", "member", "viewer"].includes(String(value))) fail(); }
function text(value: unknown) { if (typeof value !== "string" || !value || value.length > 8192) fail(); }
function nullableText(value: unknown) { if (value !== null && (typeof value !== "string" || value.length > 8192)) fail(); }
function fail(): never { throw new Error("account_protocol_invalid"); }
