import { accountErrorCode, AccountResponseFence, type AccountResponseToken } from "./fence";
import { parseCloudHead, type CloudSnapshot } from "./cloud_head";
import { parseCloudApplyResult, parseCloudDiscardResult, parseCloudStageResult, type CloudApplyResult, type CloudDiscardResult, type CloudStageResult } from "./cloud_sync";
import {
  catalogResources, emptyInventory, entitlementKey, isInstalled, parseCatalogPage, parseEntitlements, parseInstallReceipt, parseInventory, supportsPackageInstall,
  type CatalogRelease, type PackageRuntimeMode,
} from "../catalog/service/protocol";
import {
  assertSnapshot, emptyResource, parseCloudBinding, parseCloudCommitReconciliation, parsePage, record, resourceKey, unavailableSnapshot, uuid,
  type AccountAction, type AccountSnapshot, type AuthAction, type Invitation, type IssuedInvitation, type Member,
  type CloudBinding, type Organization, type OrganizationMutation, type Project, type QueryAction, type Resource, type ResourceItem,
} from "./protocol";

interface AccountTransport {
  load: (epoch: string) => Promise<AccountSnapshot>;
  dispatch: (action: AccountAction) => Promise<AccountSnapshot>;
  operationId: () => string;
}

type MutationRequest = Extract<AccountAction, { action: "create-organization" | "mutate" | "catalog-license" | "package-install" | "cloud-push" | "cloud-resume-push" }>;
type RecoveryAction = "reconcile" | "retry" | "acknowledge";
export interface MutationState {
  operationId: string;
  status: "running" | "unknown" | "failed";
  error: string | null;
  sourceAction: AccountSnapshot["operations"][number]["action"] | null;
  canRetry: boolean;
}

export interface AccountState extends ReturnType<typeof catalogResources> {
  snapshot: AccountSnapshot;
  ready: boolean;
  pending: AuthAction | null;
  error: string | null;
  organizations: Resource<Organization>;
  invitations: Resource<Invitation>;
  issuedInvitations: Resource<IssuedInvitation>;
  members: Resource<Member>;
  projects: Resource<Project>;
  cloudHead: { organization: string; project: string; snapshot: CloudSnapshot | null; loading: boolean; loaded: boolean; error: string | null } | null;
  cloudBinding: CloudBinding | null;
  cloudBindingLoading: boolean;
  cloudBindingError: string | null;
  cloudSync: CloudSyncState | null;
  cloudSyncScope: CloudBinding | null;
  selectedOrganization: string | null;
  organizationsNeedingRefresh: string[];
  mutation: MutationState | null;
}

export type CloudSyncState =
  | { status: "uploading"; operationId: string; baseRevision: string }
  | { status: "committed"; operationId: string; baseRevision: string | null; revision: string | null }
  | { status: "unknown"; operationId: string; baseRevision: string | null; error: string | null }
  | { status: "failed"; operationId: string; baseRevision: string | null; error: string }
  | { status: "downloading"; revision: string }
  | { status: "discarding"; revision: string }
  | { status: "discarded"; result: CloudDiscardResult }
  | { status: "staged"; stage: CloudStageResult }
  | { status: "applying"; stageId: string; revision: string }
  | { status: "applied"; result: Extract<CloudApplyResult, { status: "applied" }> }
  | { status: "conflict"; result: Extract<CloudApplyResult, { status: "conflict" }> }
  | { status: "download-failed"; revision: string; error: string }
  | { status: "discard-failed"; revision: string; error: string }
  | { status: "apply-failed"; stageId: string; revision: string; error: string };

function resources() {
  return { organizations: emptyResource<Organization>(), invitations: emptyResource<Invitation>(), issuedInvitations: emptyResource<IssuedInvitation>(), members: emptyResource<Member>(), projects: emptyResource<Project>(), cloudHead: null, cloudBinding: null, cloudBindingLoading: false, cloudBindingError: null, cloudSync: null, cloudSyncScope: null, selectedOrganization: null, organizationsNeedingRefresh: [] as string[], ...catalogResources() };
}

export class AccountController {
  private state: AccountState;
  private readonly epoch: string;
  private readonly transport: AccountTransport;
  private readonly fence: AccountResponseFence;
  private readonly listeners = new Set<() => void>();
  private active = false;
  private pendingRequest: MutationRequest | null = null;
  private cloudAttachPending = false;
  private cloudBindingRefreshPending = false;

  constructor(epoch: string, transport: AccountTransport) {
    this.epoch = epoch;
    this.transport = transport;
    this.fence = new AccountResponseFence(epoch);
    this.state = { snapshot: unavailableSnapshot(epoch), ready: false, pending: null, error: null, mutation: null, ...resources() };
  }

  getSnapshot = (): AccountState => this.state;
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  stop = () => { this.active = false; this.fence.reset(this.epoch); };
  clearError = () => this.update({ error: null });

  start = async () => {
    this.active = true;
    this.fence.reset(this.epoch);
    const token = this.fence.begin();
    // A bootstrap is an authority boundary. Do not render the previous
    // account's tenant data while the new native snapshot is still pending.
    this.pendingRequest = null;
    this.update({ snapshot: unavailableSnapshot(this.epoch), ...resources(), mutation: null, ready: false, pending: null, error: null });
    try {
      const next = await this.transport.load(this.epoch);
      if (!this.accept(token, next)) return;
      this.update({ ready: true, error: this.snapshotError(next) });
      if (next.account.status === "signed-in") await this.loadOverview();
    } catch (error) {
      if (this.current(token)) {
        // A failed reload cannot establish that the cached identity and its
        // authorization are still valid. Drop them before exposing retry.
        this.fence.reset(this.epoch);
        this.pendingRequest = null;
        this.update({ snapshot: unavailableSnapshot(this.epoch), ...resources(), ready: true, pending: null, error: accountErrorCode(error) });
      }
    }
  };

  authenticate = async (action: AuthAction) => {
    if (!this.active || !this.state.ready || !this.state.snapshot.account.configured) return;
    if (this.state.pending && action !== "logout" && action !== "cancel") return;
    this.fence.reset(this.epoch);
    this.pendingRequest = null;
    const token = this.fence.begin();
    const account = this.state.snapshot.account;
    this.update({
      ...resources(), mutation: null, pending: action, error: null,
      snapshot: { ...this.state.snapshot, data: null, error: null, operations: [], operationsRevision: "0", operationsError: null, account: { ...account, status: "signed-out", issuer: null, subject: null, displayName: null, error: null } },
    });
    try {
      const next = await this.transport.dispatch({ action, backendEpoch: this.epoch });
      if (!this.accept(token, next)) return;
      this.update({ pending: null, error: this.snapshotError(next) });
      if (next.account.status === "signed-in") await this.loadOverview();
    } catch (error) {
      if (this.current(token)) this.update({ pending: null, error: accountErrorCode(error) });
    }
  };

  loadOverview = async () => { await Promise.all([this.load("organizations"), this.load("invitations")]); };

  selectOrganization = async (id: string) => {
    if (!this.canQuery() || !this.state.organizations.items.some(item => item.id === id)) return;
    const token = this.fence.begin(this.generation(), "organization-selection");
    this.fence.invalidate("members");
    this.fence.invalidate("projects");
    this.fence.invalidate("cloud-head");
    this.fence.invalidate("issued-invitations");
    this.invalidatePackageAccess();
    if (this.cloudAttachPending) this.cloudBindingRefreshPending = true;
    this.update({ selectedOrganization: id, organizations: { ...this.state.organizations, loading: false }, members: emptyResource(), projects: emptyResource(), issuedInvitations: emptyResource(), cloudHead: null });
    this.fence.invalidate("cloud-sync");
    this.update({ cloudSync: null, cloudSyncScope: null });
    await this.loadOrganizationResources(token);
  };

  loadCloudHead = async (project: string) => {
    const organization = this.state.selectedOrganization;
    if (!this.canQuery() || !organization || !this.state.projects.loaded || !this.state.projects.items.some(item => item.id === project)) return;
    const token = this.fence.begin(this.generation(), "cloud-head");
    this.update({ cloudHead: { organization, project, snapshot: null, loading: true, loaded: false, error: null } });
    try {
      const next = await this.transport.dispatch({ action: "cloud-head", backendEpoch: this.epoch, generation: this.generation(), organization, project });
      if (!this.accept(token, next) || this.state.selectedOrganization !== organization || !this.state.projects.items.some(item => item.id === project)) return;
      if (next.error) throw new Error(next.error);
      const snapshot = parseCloudHead(next.data, organization, project);
      this.update({ cloudHead: { organization, project, snapshot, loading: false, loaded: true, error: null } });
    } catch (error) {
      if (this.current(token) && this.state.selectedOrganization === organization) {
        this.update({ cloudHead: { organization, project, snapshot: null, loading: false, loaded: false, error: accountErrorCode(error) } });
      }
    }
  };

  loadCloudBinding = async () => {
    if (!this.canQuery()) return;
    if (this.cloudAttachPending) {
      this.cloudBindingRefreshPending = true;
      this.update({ cloudBinding: null });
      return;
    }
    const token = this.fence.begin(this.generation(), "cloud-binding");
    this.update({ cloudBinding: null, cloudBindingLoading: true, cloudBindingError: null });
    try {
      const next = await this.transport.dispatch({ action: "cloud-binding", backendEpoch: this.epoch, generation: this.generation() });
      if (!this.accept(token, next)) return;
      if (next.error) throw new Error(next.error);
      this.update({ cloudBinding: parseCloudBinding(next.data), cloudBindingLoading: false });
    } catch (error) {
      if (this.current(token)) this.update({ cloudBinding: null, cloudBindingLoading: false, cloudBindingError: accountErrorCode(error) });
    }
  };

  pushCloudSnapshot = async (): Promise<boolean> => {
    const { cloudBinding, cloudHead, selectedOrganization } = this.state;
    if (!this.canMutate() || !cloudBinding || !selectedOrganization || cloudBinding.organizationId !== selectedOrganization
      || !cloudHead?.loaded || cloudHead.loading || cloudHead.error || cloudHead.organization !== selectedOrganization
      || cloudHead.project !== cloudBinding.projectId || this.state.cloudBindingLoading || this.state.cloudBindingError
      || !this.state.projects.loaded || !this.state.projects.items.some(item => item.id === cloudBinding.projectId)) return false;
    const baseRevision = cloudHead.snapshot?.revision ?? "0";
    const operationId = this.transport.operationId();
    uuid(operationId);
    const token = this.fence.begin(this.generation(), "cloud-sync");
    this.update({ cloudSync: { status: "uploading", operationId, baseRevision }, cloudSyncScope: cloudBinding });
    const succeeded = await this.runMutation({ action: "cloud-push", backendEpoch: this.epoch, generation: this.generation(), operationId, baseRevision });
    if (!this.current(token) || this.state.selectedOrganization !== selectedOrganization) return succeeded;
    const operation = this.state.snapshot.operations.find(item => item.operationId === operationId);
    if (operation?.status === "committed") {
      this.update({ cloudSync: { status: "committed", operationId, baseRevision, revision: (BigInt(baseRevision) + 1n).toString() } });
    } else if (operation?.status === "unknown" || this.state.mutation?.operationId === operationId && this.state.mutation.status === "unknown") {
      this.update({ cloudSync: { status: "unknown", operationId, baseRevision, error: operation?.error ?? this.state.mutation?.error ?? null } });
    } else {
      this.update({ cloudSync: { status: "failed", operationId, baseRevision, error: operation?.error ?? this.state.mutation?.error ?? this.state.snapshot.error ?? "account_service_operation_failed" } });
    }
    return succeeded;
  };

  retryCloudSnapshot = async (operationId: string): Promise<boolean> => {
    const operation = this.state.snapshot.operations.find(item => item.operationId === operationId);
    const localUnknown = this.state.mutation?.operationId === operationId
      && this.state.mutation.sourceAction === "cloud-commit" && this.state.mutation.status === "unknown";
    if (!this.canQuery() || this.state.mutation?.status === "running" || this.state.snapshot.operationsError
      || (!localUnknown && (operation?.action !== "cloud-commit" || operation.status !== "unknown"))
      || (operation && (operation.action !== "cloud-commit" || operation.status !== "unknown"))) return false;
    const binding = this.state.cloudBinding;
    if (!binding || binding.organizationId !== this.state.selectedOrganization
      || !this.state.projects.loaded || !this.state.projects.items.some(item => item.id === binding.projectId)) return false;
    const syncScope = this.state.cloudSyncScope;
    if (syncScope && (syncScope.localProjectGuid !== binding.localProjectGuid
      || syncScope.organizationId !== binding.organizationId || syncScope.projectId !== binding.projectId)) return false;
    uuid(operationId);
    const currentCloudSync = this.state.cloudSync;
    const baseRevision = currentCloudSync && "operationId" in currentCloudSync && currentCloudSync.operationId === operationId
      && "baseRevision" in currentCloudSync ? currentCloudSync.baseRevision : null;
    const token = this.fence.begin(this.generation(), "cloud-sync");
    this.update({ cloudSync: { status: "unknown", operationId, baseRevision, error: operation?.error ?? null } });
    const request: Extract<AccountAction, { action: "cloud-resume-push" }> = {
      action: "cloud-resume-push", backendEpoch: this.epoch, generation: this.generation(), operationId,
    };
    this.pendingRequest = request;
    const succeeded = await this.runOperation(request, false);
    if (!this.current(token) || this.state.selectedOrganization !== binding.organizationId) return succeeded;
    const updated = this.state.snapshot.operations.find(item => item.operationId === operationId);
    if (updated?.status === "unknown") this.update({ cloudSync: { status: "unknown", operationId, baseRevision, error: updated.error } });
    return succeeded;
  };

  stageCloudDownload = async (expectedRevision: string): Promise<boolean> => {
    const { cloudBinding, cloudHead, selectedOrganization } = this.state;
    if (!this.canMutate() || !cloudBinding || !selectedOrganization || cloudBinding.organizationId !== selectedOrganization
      || !cloudHead?.snapshot || !cloudHead.loaded || cloudHead.loading || cloudHead.error
      || cloudHead.organization !== selectedOrganization || cloudHead.project !== cloudBinding.projectId
      || cloudHead.snapshot.revision !== expectedRevision || !this.state.projects.loaded
      || !this.state.projects.items.some(item => item.id === cloudBinding.projectId)) return false;
    const token = this.fence.begin(this.generation(), "cloud-sync");
    this.update({ cloudSync: { status: "downloading", revision: expectedRevision }, cloudSyncScope: cloudBinding });
    try {
      const next = await this.transport.dispatch({ action: "cloud-stage-download", backendEpoch: this.epoch, generation: this.generation(), expectedRevision });
      if (!this.accept(token, next) || this.state.selectedOrganization !== selectedOrganization) return false;
      if (next.error) throw new Error(next.error);
      const stage = parseCloudStageResult(next.data, expectedRevision);
      this.update({ cloudSync: { status: "staged", stage } });
      return true;
    } catch (error) {
      if (this.current(token)) this.update({ cloudSync: { status: "download-failed", revision: expectedRevision, error: error instanceof Error ? error.message : "account_service_operation_failed" } });
      return false;
    }
  };

  discardCloudDownload = async (expectedRevision: string): Promise<boolean> => {
    const { cloudBinding, cloudSync, selectedOrganization } = this.state;
    const stageRevision = cloudSync?.status === "staged" ? cloudSync.stage.revision
      : cloudSync?.status === "conflict" ? cloudSync.result.revision
        : cloudSync?.status === "apply-failed" || cloudSync?.status === "download-failed" || cloudSync?.status === "discard-failed"
          ? cloudSync.revision : null;
    if (!this.canMutate() || !cloudBinding || !selectedOrganization
      || cloudBinding.organizationId !== selectedOrganization || stageRevision !== expectedRevision) return false;
    const token = this.fence.begin(this.generation(), "cloud-sync");
    this.update({ cloudSync: { status: "discarding", revision: expectedRevision }, cloudSyncScope: cloudBinding });
    try {
      const next = await this.transport.dispatch({
        action: "cloud-discard-download",
        backendEpoch: this.epoch,
        generation: this.generation(),
        expectedRevision,
      });
      if (!this.accept(token, next) || this.state.selectedOrganization !== selectedOrganization) return false;
      if (next.error) throw new Error(next.error);
      const result = parseCloudDiscardResult(next.data, expectedRevision);
      this.update({ cloudSync: { status: "discarded", result } });
      return true;
    } catch (error) {
      if (this.current(token)) this.update({
        cloudSync: {
          status: "discard-failed",
          revision: expectedRevision,
          error: error instanceof Error ? error.message : "account_service_operation_failed",
        },
      });
      return false;
    }
  };

  applyCloudDownload = async (stageId: string, expectedRevision: string): Promise<boolean> => {
    const { cloudBinding, cloudHead, selectedOrganization } = this.state;
    if (!this.canMutate() || !cloudBinding || !selectedOrganization || cloudBinding.organizationId !== selectedOrganization
      || cloudHead?.snapshot?.revision !== expectedRevision || cloudHead.project !== cloudBinding.projectId
      || !((this.state.cloudSync?.status === "staged" && this.state.cloudSync.stage.stageId === stageId && this.state.cloudSync.stage.revision === expectedRevision)
        || (this.state.cloudSync?.status === "conflict" && this.state.cloudSync.result.stageId === stageId && this.state.cloudSync.result.revision === expectedRevision)
        || (this.state.cloudSync?.status === "apply-failed" && this.state.cloudSync.stageId === stageId && this.state.cloudSync.revision === expectedRevision))) return false;
    uuid(stageId);
    const token = this.fence.begin(this.generation(), "cloud-sync");
    this.update({ cloudSync: { status: "applying", stageId, revision: expectedRevision } });
    try {
      const next = await this.transport.dispatch({ action: "cloud-apply-download", backendEpoch: this.epoch, generation: this.generation(), stageId, expectedRevision });
      if (!this.accept(token, next) || this.state.selectedOrganization !== selectedOrganization) return false;
      if (next.error) throw new Error(next.error);
      const result = parseCloudApplyResult(next.data, stageId, expectedRevision);
      this.update({ cloudSync: result.status === "applied" ? { status: "applied", result } : { status: "conflict", result } });
      await this.loadCloudHead(cloudBinding.projectId);
      return result.status === "applied";
    } catch (error) {
      if (this.current(token)) this.update({ cloudSync: { status: "apply-failed", stageId, revision: expectedRevision, error: error instanceof Error ? error.message : "account_service_operation_failed" } });
      return false;
    }
  };

  attachCloudProject = async (project: string, selectedProjectId: string): Promise<boolean> => {
    const organization = this.state.selectedOrganization;
    if (!this.canQuery() || !organization || !selectedProjectId || this.cloudAttachPending || this.state.cloudBindingLoading || this.state.snapshot.operationsError
      || !this.state.projects.loaded || !this.state.projects.items.some(item => item.id === project)) return false;
    this.cloudAttachPending = true;
    const token = this.fence.begin(this.generation(), "cloud-binding");
    this.update({ cloudBindingLoading: true, cloudBindingError: null });
    try {
      const next = await this.transport.dispatch({ action: "attach-cloud-project", backendEpoch: this.epoch, generation: this.generation(), organization, project, selectedProjectId });
      if (!this.accept(token, next) || this.state.selectedOrganization !== organization) return false;
      if (next.error) throw new Error(next.error);
      const binding = parseCloudBinding(next.data);
      if (!binding || binding.organizationId !== organization || binding.projectId !== project) throw new Error("account_protocol_invalid");
      this.update({ cloudBinding: binding, cloudBindingLoading: false, cloudBindingError: null });
      return true;
    } catch (error) {
      if (this.current(token) && this.state.selectedOrganization === organization) this.update({ cloudBindingLoading: false, cloudBindingError: accountErrorCode(error) });
      return false;
    } finally {
      this.cloudAttachPending = false;
      if (this.state.selectedOrganization !== organization) this.cloudBindingRefreshPending = true;
      if (this.cloudBindingRefreshPending) {
        this.cloudBindingRefreshPending = false;
        if (this.canQuery()) {
          this.update({ cloudBindingLoading: false, cloudBindingError: null });
          await this.loadCloudBinding();
        }
      }
    }
  };

  refreshOrganization = async (target?: Pick<Member, "issuer" | "subject">, invitationId?: string): Promise<Organization | null> => {
    const organization = this.state.selectedOrganization;
    if (!this.canQuery() || !organization || this.state.mutation?.status === "running") return null;
    const token = this.fence.begin(this.generation(), "organization-selection");
    this.fence.invalidate("organizations");
    this.fence.invalidate("members");
    this.fence.invalidate("projects");
    this.fence.invalidate("cloud-head");
    this.fence.invalidate("issued-invitations");
    this.invalidatePackageAccess();
    this.update({ organizations: { ...this.state.organizations, loading: true, error: null }, members: emptyResource(), projects: emptyResource(), issuedInvitations: emptyResource(), cloudHead: null });
    const items: Organization[] = [];
    const cursors = new Set<string>();
    let after: string | undefined;
    try {
      // A selected organization may be beyond page one; keep its policy and membership bound to this selection.
      for (;;) {
        const next = await this.transport.dispatch({ action: "organizations", backendEpoch: this.epoch, generation: token.generation!, after });
        if (!this.accept(token, next)) return null;
        if (next.error) throw new Error(next.error);
        const page = parsePage("organizations", next.data);
        if (after) {
          const previousCursor = after;
          if ((page.nextCursor !== null && page.nextCursor <= previousCursor) || page.items.some(item => item.id <= previousCursor)) {
            throw new Error("account_protocol_invalid");
          }
        }
        if (page.nextCursor && cursors.has(page.nextCursor)) throw new Error("account_protocol_invalid");
        items.push(...page.items);
        if (new Set(items.map(item => item.id)).size !== items.length) throw new Error("account_protocol_invalid");
        const selected = items.find(item => item.id === organization);
        if (selected || page.nextCursor === null) {
          this.update({ organizations: { ...page, items, loading: false, loaded: true, error: null }, selectedOrganization: selected ? organization : null });
          if (!selected) return null;
          await this.loadOrganizationResources(token, target, invitationId);
          if (!this.current(token) || this.state.members.error || this.state.members.loading || !this.state.members.loaded) return null;
          if (invitationId && (this.state.issuedInvitations.error || !this.state.issuedInvitations.loaded)) return null;
          this.dismissFailedMutation();
          this.update({ organizationsNeedingRefresh: this.state.organizationsNeedingRefresh.filter(id => id !== organization) });
          return selected;
        }
        cursors.add(page.nextCursor);
        after = page.nextCursor;
      }
    } catch (error) {
      if (this.current(token)) this.update({ organizations: { ...emptyResource(), error: accountErrorCode(error) }, selectedOrganization: null, members: emptyResource(), projects: emptyResource(), issuedInvitations: emptyResource() });
      return null;
    }
  };

  private async loadOrganizationResources(token: AccountResponseToken, target?: Pick<Member, "issuer" | "subject">, invitationId?: string) {
    await Promise.all([this.load("members"), this.load("projects")]);
    const cursors = new Set<string>();
    while (this.current(token)) {
      const members = this.state.members;
      const account = this.state.snapshot.account;
      const contains = (identity: Pick<Member, "issuer" | "subject">) => members.items.some(item => item.issuer === identity.issuer && item.subject === identity.subject);
      const actorFound = account.issuer !== null && account.subject !== null && contains({ issuer: account.issuer, subject: account.subject });
      if ((actorFound && (!target || contains(target))) || members.loading || members.error || !members.nextCursor) break;
      if (cursors.has(members.nextCursor)) { this.update({ members: { ...members, error: "account_protocol_invalid" } }); return; }
      cursors.add(members.nextCursor);
      await this.load("members", true);
    }
    if (!this.current(token) || !this.canViewIssuedInvitations()) return;
    await this.load("issued-invitations");
    cursors.clear();
    while (this.current(token) && invitationId) {
      const invitations = this.state.issuedInvitations;
      if (invitations.items.some(item => item.id === invitationId) || invitations.error || invitations.loading || !invitations.nextCursor) return;
      if (cursors.has(invitations.nextCursor)) { this.update({ issuedInvitations: { ...invitations, error: "account_protocol_invalid" } }); return; }
      cursors.add(invitations.nextCursor);
      await this.load("issued-invitations", true);
    }
  }

  load = async <K extends QueryAction>(kind: K, more = false) => {
    if (!this.canQuery()) return;
    const organization = this.state.selectedOrganization;
    const scoped = kind === "members" || kind === "projects" || kind === "issued-invitations";
    if (scoped && !organization) return;
    if (kind === "issued-invitations" && !this.canViewIssuedInvitations()) return;
    const previous = (kind === "issued-invitations" ? this.state.issuedInvitations : this.state[kind as Exclude<QueryAction, "issued-invitations">]) as Resource<ResourceItem<K>>;
    const refreshRequired = this.state.organizationsNeedingRefresh;
    const operationRevision = this.state.snapshot.operationsRevision;
    const mutationRunning = this.state.mutation?.status === "running";
    if (more && (previous.loading || !previous.nextCursor)) return;
    const after = more ? previous.nextCursor! : undefined;
    const retained = more ? previous : emptyResource<ResourceItem<K>>();
    const token = this.fence.begin(this.generation(), kind);
    if (!more && (kind === "organizations" || kind === "projects")) {
      this.fence.invalidate("cloud-head");
      this.update({ cloudHead: null });
    }
    this.updateResource(kind, { ...retained, loading: true, error: null });
    if (kind === "organizations" && !more) {
      // Re-authorize the selected organization before exposing its cached resources.
      this.fence.invalidate("members"); this.fence.invalidate("projects");
      this.fence.invalidate("organization-selection");
      this.fence.invalidate("issued-invitations");
      this.invalidatePackageAccess();
      this.update({ selectedOrganization: null, members: emptyResource(), projects: emptyResource(), issuedInvitations: emptyResource(), cloudHead: null });
    }
    if (kind === "members" && !more) {
      this.fence.invalidate("issued-invitations");
      this.update({ issuedInvitations: emptyResource() });
    }
    const request: AccountAction = scoped
      ? { action: kind, backendEpoch: this.epoch, generation: this.generation(), organization: organization!, after }
      : { action: kind, backendEpoch: this.epoch, generation: this.generation(), after };
    try {
      const next = await this.transport.dispatch(request);
      if (!this.accept(token, next)) return;
      if (next.error) throw new Error(next.error);
      const page = parsePage(kind, next.data);
      if (kind === "issued-invitations" && (page.items as IssuedInvitation[]).some(item => item.organizationId !== organization)) throw new Error("account_protocol_invalid");
      if (more) {
        const cursor = after!;
        const cursorBacktracks = page.nextCursor !== null && (kind === "members"
          ? BigInt(page.nextCursor) <= BigInt(cursor)
          : page.nextCursor <= cursor);
        const rowBacktracks = kind !== "members" && page.items.some(item => resourceKey(kind, item) <= cursor);
        if (cursorBacktracks || rowBacktracks) throw new Error("account_protocol_invalid");
      }
      const items = more ? [...previous.items] : [];
      const positions = new Map(items.map((item, index) => [resourceKey(kind, item), index]));
      for (const item of page.items) {
        const key = resourceKey(kind, item);
        const index = positions.get(key);
        if (index === undefined) { positions.set(key, items.length); items.push(item); } else items[index] = item;
      }
      this.updateResource(kind, { ...page, items, loading: false, loaded: true, error: null });
      if (kind === "invitations" && !mutationRunning && operationRevision === this.state.snapshot.operationsRevision) {
        const refreshed = new Set((page.items as Invitation[]).map(item => item.organizationId).filter(id => refreshRequired.includes(id)));
        this.update({ organizationsNeedingRefresh: this.state.organizationsNeedingRefresh.filter(id => !refreshed.has(id)) });
      }
    } catch (error) {
      if (this.current(token)) this.updateResource(kind, { ...retained, loading: false, error: accountErrorCode(error) });
    }
  };

  loadCatalog = async (query = this.state.catalogQuery, more = false) => {
    if (!this.canQuery()) return;
    const previous = this.state.serviceCatalog;
    if (more && (previous.loading || !previous.nextCursor || query !== this.state.catalogQuery)) return;
    if (new TextEncoder().encode(query).length > 256) {
      this.update({ serviceCatalog: { ...previous, error: "account_protocol_invalid" } }); return;
    }
    const after = more ? previous.nextCursor! : undefined;
    const retained = more ? previous : emptyResource<CatalogRelease>();
    const token = this.fence.begin(this.generation(), "catalog");
    this.update({ catalogQuery: query, serviceCatalog: { ...retained, loading: true, error: null } });
    try {
      const next = await this.transport.dispatch({ action: "catalog", backendEpoch: this.epoch, generation: this.generation(), after, query: query || undefined });
      if (!this.accept(token, next)) return;
      if (next.error) throw new Error(next.error);
      const page = parseCatalogPage(next.data);
      if (after && (page.items.some(item => item.package_id <= after) || (page.nextCursor !== null && page.nextCursor <= after))) throw new Error("account_protocol_invalid");
      const items = [...retained.items, ...page.items];
      if (new Set(items.map(item => item.package_id)).size !== items.length) throw new Error("account_protocol_invalid");
      this.update({ serviceCatalog: { ...page, items, loading: false, loaded: true, error: null } });
    } catch (error) {
      if (this.current(token)) this.update({ serviceCatalog: { ...retained, loading: false, error: accountErrorCode(error) } });
    }
  };

  loadCatalogEntitlements = async (more = false) => {
    const organization = this.state.selectedOrganization;
    if (!this.canQuery() || !organization) return;
    const previous = this.state.catalogEntitlements;
    if (more && (previous.loading || !previous.nextCursor)) return;
    const after = more ? previous.nextCursor! : undefined;
    const retained = more ? previous : emptyResource<ReturnType<typeof parseEntitlements>["items"][number]>();
    const token = this.fence.begin(this.generation(), "catalog-entitlements");
    this.update({ catalogEntitlements: { ...retained, loading: true, error: null } });
    try {
      const next = await this.transport.dispatch({ action: "catalog-entitlements", backendEpoch: this.epoch, generation: this.generation(), organization, after });
      if (!this.accept(token, next)) return;
      if (next.error) throw new Error(next.error);
      const page = parseEntitlements(next.data);
      if (after && page.nextCursor !== null && BigInt(page.nextCursor) <= BigInt(after)) throw new Error("account_protocol_invalid");
      const items = [...retained.items, ...page.items];
      if (new Set(items.map(entitlementKey)).size !== items.length) throw new Error("account_protocol_invalid");
      this.update({ catalogEntitlements: { ...page, items, loading: false, loaded: true, error: null } });
    } catch (error) {
      if (this.current(token)) this.update({ catalogEntitlements: { ...retained, loading: false, error: accountErrorCode(error) } });
    }
  };

  loadPackageInventory = async (targetMode: PackageRuntimeMode) => {
    const organization = this.state.selectedOrganization;
    if (!this.canQuery() || !organization) return;
    const previous = this.state.packageInventory;
    const retained = previous.data?.targetMode === targetMode ? previous : emptyInventory();
    const token = this.fence.begin(this.generation(), "package-inventory");
    this.update({ packageInventory: { ...retained, targetMode, loading: true, error: null } });
    try {
      const next = await this.transport.dispatch({ action: "package-inventory", schemaVersion: 2, targetMode, backendEpoch: this.epoch, generation: this.generation(), organization });
      if (!this.accept(token, next)) return;
      if (next.error) throw new Error(next.error);
      const data = parseInventory(next.data, targetMode);
      if (retained.data && BigInt(data.revision) < BigInt(retained.data.revision)) throw new Error("account_protocol_invalid");
      this.update({ packageInventory: { data, targetMode, loading: false, error: null } });
    } catch (error) {
      if (this.current(token)) this.update({ packageInventory: { ...retained, targetMode, loading: false, error: accountErrorCode(error) } });
    }
  };

  acceptCatalogLicense = async (release: CatalogRelease): Promise<boolean> => {
    const organization = this.state.organizations.items.find(item => item.id === this.state.selectedOrganization);
    if (!organization || !this.canChangePackage(release) || !this.canViewIssuedInvitations()) return false;
    const operationId = this.transport.operationId();
    uuid(operationId);
    return this.runMutation({ action: "catalog-license", backendEpoch: this.epoch, generation: this.generation(), organization: organization.id, operationId,
      expectedPolicyRevision: organization.policyRevision, packageId: release.package_id, revision: release.revision, licenseId: release.license_id });
  };

  installCatalogPackage = async (release: CatalogRelease, targetMode: PackageRuntimeMode): Promise<boolean> => {
    const organization = this.state.selectedOrganization;
    const { packageInventory, catalogEntitlements } = this.state;
    const entitled = catalogEntitlements.loaded && !catalogEntitlements.loading && !catalogEntitlements.error && catalogEntitlements.items.some(item => item.packageId === release.package_id && item.revision === release.revision && item.licenseId === release.license_id);
    if (!organization || !supportsPackageInstall(release) || !this.canChangePackage(release) || !entitled || packageInventory.targetMode !== targetMode || packageInventory.data?.targetMode !== targetMode || packageInventory.loading || packageInventory.error || isInstalled(release, packageInventory.data, targetMode)) return false;
    const operationId = this.transport.operationId();
    uuid(operationId);
    return this.runMutation({ action: "package-install", schemaVersion: 2, targetMode, backendEpoch: this.epoch, generation: this.generation(), organization, operationId,
      packageId: release.package_id, revision: release.revision, expectedInventoryRevision: packageInventory.data.revision });
  };

  private canChangePackage(release: CatalogRelease) {
    const { members, snapshot, serviceCatalog, selectedOrganization, organizations } = this.state;
    return this.canMutate() && selectedOrganization !== null && !this.state.organizationsNeedingRefresh.includes(selectedOrganization)
      && !organizations.loading && members.loaded && !members.loading && !members.error && release.exp * 1000 > Date.now()
      && serviceCatalog.loaded && !serviceCatalog.loading && !serviceCatalog.error && serviceCatalog.items.includes(release)
      && members.items.some(item => item.active && item.issuer === snapshot.account.issuer && item.subject === snapshot.account.subject);
  }

  private invalidatePackageAccess() {
    this.fence.invalidate("catalog-entitlements"); this.fence.invalidate("package-inventory");
    this.update({ catalogEntitlements: emptyResource(), packageInventory: emptyInventory() });
  }

  createOrganization = async (name: string): Promise<boolean> => {
    if (!this.canMutate() || !name.trim()) return false;
    const operationId = this.transport.operationId();
    uuid(operationId);
    return this.runMutation({ action: "create-organization", backendEpoch: this.epoch, generation: this.generation(), operationId, name: name.trim() });
  };

  mutate = async (organization: string, expectedPolicyRevision: string, mutation: OrganizationMutation): Promise<boolean> => {
    if (!this.canMutate() || this.state.organizationsNeedingRefresh.includes(organization) || this.state.organizations.loading
      || (organization === this.state.selectedOrganization && this.state.members.loading)) return false;
    const operationId = this.transport.operationId();
    uuid(operationId);
    return this.runMutation({ action: "mutate", backendEpoch: this.epoch, generation: this.generation(), organization, expectedPolicyRevision, operationId, mutation });
  };

  reloadOperations = async () => {
    if (!this.canQuery()) return;
    const token = this.fence.begin(this.generation(), "operations");
    try {
      const next = await this.transport.load(this.epoch);
      if (!this.accept(token, next)) return;
      this.update({ error: this.snapshotError(this.state.snapshot) });
    } catch (error) {
      if (this.current(token)) {
        // Until the journal is read successfully, a new write could not be
        // reconciled after an IPC or storage failure. Preserve its last
        // summaries but make the write gate fail closed.
        const journalError = "account_operation_store_unavailable" as const;
        this.update({ snapshot: { ...this.state.snapshot, operationsError: journalError }, error: journalError });
      }
    }
  };

  retryMutation = async (operationId: string) => {
    if (!this.canQuery() || this.state.mutation?.status === "running" || this.state.snapshot.operationsError) return false;
    const operation = this.state.snapshot.operations.find(item => item.operationId === operationId);
    // A cloud retry requires the native staged manifest and blob set. Until
    // desktop staging exists, only receipt reconciliation is exposed here.
    if (operation?.action === "cloud-commit" || (this.state.mutation?.operationId === operationId && this.state.mutation.sourceAction === "cloud-commit")) return false;
    // Without a journal summary, only a retained original request has the
    // payload required for a safe local retry.
    if (!operation && (this.pendingRequest?.operationId !== operationId || !this.state.mutation?.canRetry)) return false;
    // Native rejection details are sanitized, so all definite organization failures need fresh user confirmation.
    if (operation && operation.action !== "create-organization" && operation.status === "failed") return false;
    if (this.pendingRequest && this.pendingRequest.action !== "create-organization" && this.state.mutation?.status === "failed") return false;
    if (this.pendingRequest?.operationId === operationId && !this.state.snapshot.operations.some(operation => operation.operationId === operationId)) {
      return this.runOperation(this.pendingRequest, false);
    }
    return this.runRecovery("retry", operationId);
  };
  checkMutation = async (operationId: string) => this.runRecovery("reconcile", operationId);
  acknowledgeOperation = async (operationId: string) => {
    const operation = this.state.snapshot.operations.find(item => item.operationId === operationId);
    if (!operation || operation.status === "unknown") return false;
    return this.runRecovery("acknowledge", operationId);
  };

  dismissFailedMutation = () => {
    if (this.state.mutation?.status !== "failed") return;
    this.pendingRequest = null;
    this.update({ mutation: null });
  };

  private async runRecovery(action: RecoveryAction, operationId: string): Promise<boolean> {
    if (!this.canQuery() || this.state.mutation?.status === "running" || this.state.snapshot.operationsError) return false;
    const operation = this.state.snapshot.operations.find(item => item.operationId === operationId);
    if (this.state.mutation?.operationId !== operationId && !operation) return false;
    if ((action === "retry" || action === "reconcile") && operation?.action === "package-install" && operation.status === "unknown" && !operation.targetMode) {
      this.update({ error: "account_package_target_migration_required" });
      return false;
    }
    uuid(operationId);
    return this.runOperation({ action, backendEpoch: this.epoch, generation: this.generation(), operationId }, false);
  }

  private async runMutation(request: MutationRequest): Promise<boolean> {
    this.pendingRequest = request;
    return this.runOperation(request, true);
  }

  private async runOperation(request: MutationRequest | Extract<AccountAction, { action: RecoveryAction }>, firstAttempt: boolean): Promise<boolean> {
    const { operationId } = request;
    const knownOperation = this.state.snapshot.operations.find(operation => operation.operationId === operationId);
    const sourceAction = request.action === "reconcile" || request.action === "retry" || request.action === "acknowledge"
      ? knownOperation?.action ?? (this.state.mutation?.operationId === operationId ? this.state.mutation.sourceAction : null)
      : request.action === "cloud-push" || request.action === "cloud-resume-push" ? "cloud-commit" : request.action;
    const canRetry = sourceAction !== "cloud-commit" && this.pendingRequest?.operationId === operationId;
    const reconcilingCloud = request.action === "reconcile" && sourceAction === "cloud-commit";
    const token = this.fence.begin(this.generation(), "mutation");
    this.update({ mutation: { operationId, status: "running", error: null, sourceAction, canRetry: false } });
    try {
      const next = await this.transport.dispatch(request);
      if (!this.accept(token, next)) return false;
      const snapshot = this.state.snapshot;
      const operation = snapshot.operations.find(item => item.operationId === operationId);
      if (reconcilingCloud && !next.error && !snapshot.operationsError) {
        try {
          if (!operation) throw new Error("account_protocol_invalid");
          parseCloudCommitReconciliation(next.data, operation, operationId);
        } catch {
          const error = "account_protocol_invalid";
          this.update({ snapshot: { ...snapshot, operationsError: error }, mutation: { operationId, status: "unknown", error, sourceAction, canRetry: false }, error });
          return false;
        }
      }
      if (request.action === "acknowledge" && !next.error && !snapshot.operationsError && !operation) {
        this.pendingRequest = null;
        const failedCloudSync = this.state.cloudSync?.status === "failed" && this.state.cloudSync.operationId === operationId;
        this.update({ mutation: null, ...(failedCloudSync ? { cloudSync: null, cloudSyncScope: null } : {}) });
        return true;
      }
      if (operation && !snapshot.operationsError) {
        this.pendingRequest = null;
        const conflict = next.error === "account_operation_id_conflict";
        const activeCloudSync = this.state.cloudSync;
        let cloudSync: CloudSyncState | undefined;
        if (operation.action === "cloud-commit" && activeCloudSync && "operationId" in activeCloudSync && activeCloudSync.operationId === operationId) {
          const baseRevision = "baseRevision" in activeCloudSync ? activeCloudSync.baseRevision : null;
          cloudSync = operation.status === "unknown"
            ? { status: "unknown", operationId, baseRevision, error: operation.error }
            : operation.status === "committed"
              ? { status: "committed", operationId, baseRevision, revision: baseRevision === null ? null : (BigInt(baseRevision) + 1n).toString() }
              : { status: "failed", operationId, baseRevision, error: operation.error ?? "account_service_operation_failed" };
        }
        this.update({ mutation: null, error: next.error && next.error !== "account_service_outcome_unknown" ? accountErrorCode(next.error) : null, ...(cloudSync ? { cloudSync } : {}) });
        if (operation.action === "cloud-commit" && operation.status !== "unknown") {
          this.fence.invalidate("cloud-head");
          this.update({ cloudHead: null });
        }
        if (operation.status === "committed" && !conflict && operation.action !== "cloud-commit") {
          if (operation.action === "package-install" || operation.action === "catalog-license") {
            let inventoryTargetMode = this.state.packageInventory.targetMode;
            if (operation.action === "package-install") {
              inventoryTargetMode = request.action === "package-install" ? request.targetMode : operation.targetMode ?? null;
              if (!inventoryTargetMode) {
                this.update({ error: accountErrorCode(new Error("account_package_target_migration_required")), mutation: null });
                return false;
              }
              if (request.action === "package-install" && operation.targetMode && operation.targetMode !== request.targetMode) {
                this.update({ error: "account_protocol_invalid", mutation: null });
                return false;
              }
            }
            if (operation.action === "package-install" && next.data !== null) {
              try {
                const outcome = request.action === "reconcile" ? record(next.data) : null;
                if (outcome && outcome.status !== "committed") throw new Error("account_protocol_invalid");
                const targetMode = request.action === "package-install" ? request.targetMode : operation.targetMode;
                if (!targetMode) throw new Error("account_package_target_migration_required");
                const receipt = parseInstallReceipt(outcome ? outcome.result : next.data, targetMode);
                if (receipt.operationId !== operationId || (request.action === "package-install" && (receipt.package.packageId !== request.packageId || receipt.package.releaseRevision !== request.revision))) throw new Error("account_protocol_invalid");
              } catch (error) {
                this.update({ error: accountErrorCode(error), mutation: null });
                return false;
              }
            }
            if (this.state.selectedOrganization === operation.organizationId) await Promise.all([
              this.loadCatalogEntitlements(),
              inventoryTargetMode ? this.loadPackageInventory(inventoryTargetMode) : Promise.resolve(),
            ]);
          } else if (operation.action === "mutate" && this.state.selectedOrganization === operation.organizationId) {
            await Promise.all([this.refreshOrganization(), this.load("invitations")]);
          } else await this.loadOverview();
        }
        return operation.status === "committed" && !conflict;
      }
      const error = this.snapshotError(snapshot) ?? "account_service_outcome_unknown";
      const failed = firstAttempt && next.error && next.error !== "account_service_outcome_unknown" && !snapshot.operationsError;
      this.update({ mutation: { operationId, status: failed ? "failed" : "unknown", error, sourceAction, canRetry: canRetry && (!failed || sourceAction === "create-organization") },
        ...(failed && (request.action === "mutate" || request.action === "catalog-license" || request.action === "package-install") ? { organizationsNeedingRefresh: [...new Set([...this.state.organizationsNeedingRefresh, request.organization])] } : {}) });
      return false;
    } catch (error) {
      // A lost IPC reply is not evidence that the database transaction did not commit.
      if (this.current(token)) this.update({ mutation: { operationId, status: "unknown", error: accountErrorCode(error), sourceAction, canRetry } });
      return false;
    }
  }

  private accept(token: AccountResponseToken, next: AccountSnapshot): boolean {
    assertSnapshot(next);
    if (!this.active || !this.fence.current(token, this.generation())) return false;
    if (next.backendEpoch !== this.epoch) throw new Error("hub_state_epoch_stale");
    if (BigInt(next.account.generation) < BigInt(this.generation())) throw new Error("account_protocol_invalid");
    if (token.generation !== undefined && next.account.generation !== token.generation) {
      this.fence.reset(this.epoch);
      this.pendingRequest = null;
      this.update({ snapshot: { ...next, data: null }, ...resources(), mutation: null, pending: null, error: this.snapshotError(next) });
      return false;
    }
    const previous = this.state.snapshot;
    const sameIdentity = previous.account.status === "signed-in" && next.account.status === "signed-in"
      && next.account.generation === previous.account.generation
      && next.account.issuer === previous.account.issuer
      && next.account.subject === previous.account.subject;
    let snapshot = { ...next, data: null };
    if (!sameIdentity) {
      // Resource replies are scoped to the prior generation. If they report a
      // different authority, discard the old tenant projection and abort the
      // caller so it cannot merge stale rows into the new account.
      this.pendingRequest = null;
      if (token.generation !== undefined) this.fence.reset(this.epoch);
      this.update({ snapshot, ...resources(), mutation: null, pending: null, error: this.snapshotError(next) });
      return token.generation === undefined;
    }
    if (sameIdentity && next.account.status === "signed-in") {
      if (next.operationsError) {
        snapshot = { ...snapshot, operations: previous.operations, operationsRevision: previous.operationsRevision };
      } else if (previous.operationsError && token.scope !== "account" && token.scope !== "operations") {
        // Resource responses carry journal fields too, but they do not prove
        // that the journal became readable. Keep the write gate closed until
        // the account bootstrap or journal-specific reload succeeds.
        snapshot = { ...snapshot, operations: previous.operations, operationsRevision: previous.operationsRevision, operationsError: previous.operationsError };
      } else if (BigInt(next.operationsRevision) < BigInt(previous.operationsRevision)) {
        snapshot = { ...snapshot, operations: previous.operations, operationsRevision: previous.operationsRevision, operationsError: previous.operationsError };
      }
    }
    const mutation = this.state.mutation;
    const newlyFailed = snapshot.operations.filter(operation => operation.action !== "create-organization" && operation.action !== "cloud-commit" && operation.status === "failed"
      && !previous.operations.some(item => item.operationId === operation.operationId && item.status === "failed"));
    this.update({ snapshot, organizationsNeedingRefresh: [...new Set([...this.state.organizationsNeedingRefresh, ...newlyFailed.map(item => item.organizationId!)])], ...(mutation && mutation.status !== "running" && !snapshot.operationsError
      && snapshot.operations.some(operation => operation.operationId === mutation.operationId) ? { mutation: null } : {}) });
    return true;
  }

  private current(token: AccountResponseToken) { return this.active && this.fence.current(token, this.generation()); }
  private generation() { return this.state.snapshot.account.generation; }
  private canQuery() { return this.active && this.state.ready && !this.state.pending && this.state.snapshot.account.status === "signed-in"; }
  private canViewIssuedInvitations() {
    const { members, snapshot } = this.state;
    return members.loaded && !members.loading && !members.error && members.items.some(member => member.active
      && member.issuer === snapshot.account.issuer && member.subject === snapshot.account.subject && (member.role === "owner" || member.role === "admin"));
  }
  private canMutate() { return this.canQuery() && !this.state.mutation && !this.state.snapshot.operationsError && !this.state.snapshot.operations.some(operation => operation.status === "unknown"); }
  private snapshotError(next: AccountSnapshot) { const error = next.operationsError ?? next.error ?? next.account.error; return error ? accountErrorCode(error) : null; }
  private updateResource<K extends QueryAction>(kind: K, value: Resource<ResourceItem<K>>) { this.update({ [kind === "issued-invitations" ? "issuedInvitations" : kind]: value }); }
  private update(change: Partial<AccountState>) { this.state = { ...this.state, ...change }; for (const listener of this.listeners) listener(); }
}
