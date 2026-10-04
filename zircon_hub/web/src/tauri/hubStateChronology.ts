import type { HubShellState } from "../types/hub";

export type HubStateSource = "bootstrap" | "invoke" | "event";

const MAX_STAGED_EVENT_EPOCHS = 4;
export const MAX_AUTOMATIC_BOOTSTRAP_RETRIES = 2;

export class HubBootstrapRetryBudget {
  private attempts = 0;

  reset() {
    this.attempts = 0;
  }

  tryAcquire(): boolean {
    if (this.attempts >= MAX_AUTOMATIC_BOOTSTRAP_RETRIES) {
      return false;
    }
    this.attempts += 1;
    return true;
  }
}

function revision(state: HubShellState): bigint {
  return BigInt(state.stateRevision);
}

export class HubStateChronology {
  private acceptedEpoch: string | undefined;
  private acceptedRevision = -1n;
  private bootstrapInFlight = false;
  private quarantined = false;
  private readonly stagedEvents = new Map<string, HubShellState>();

  constructor(initialState?: HubShellState) {
    if (initialState) {
      this.acceptedEpoch = initialState.backendEpoch;
      this.acceptedRevision = revision(initialState);
    }
  }

  beginBootstrap() {
    // A second reload cannot prove a staged epoch was retired. Keep that
    // evidence until a matching bootstrap establishes its authority.
    this.bootstrapInFlight = true;
  }

  cancelBootstrap() {
    this.bootstrapInFlight = false;
  }

  quarantine() {
    this.bootstrapInFlight = false;
    this.quarantined = true;
  }

  hasPendingBootstrap(): boolean {
    return this.stagedEvents.size > 0;
  }

  isProvablyStaleEvent(payload: unknown): boolean {
    // A bootstrap or unresolved epoch is an authority boundary. Only an event
    // from the settled, accepted epoch can be discarded before full validation.
    if (
      this.acceptedEpoch === undefined ||
      this.bootstrapInFlight ||
      this.quarantined ||
      this.stagedEvents.size > 0 ||
      payload === null ||
      typeof payload !== "object" ||
      Array.isArray(payload)
    ) {
      return false;
    }
    const envelope = payload as Record<string, unknown>;
    const eventRevision = envelope.stateRevision;
    if (
      !Object.hasOwn(envelope, "backendEpoch") ||
      !Object.hasOwn(envelope, "stateRevision") ||
      envelope.backendEpoch !== this.acceptedEpoch ||
      typeof eventRevision !== "string" ||
      eventRevision.length > this.acceptedRevision.toString().length ||
      !/^(0|[1-9]\d*)$/.test(eventRevision)
    ) {
      return false;
    }
    return BigInt(eventRevision) <= this.acceptedRevision;
  }

  acceptBootstrap(state: HubShellState): HubShellState | undefined {
    const wasQuarantined = this.quarantined;
    this.bootstrapInFlight = false;
    if (this.acceptedEpoch !== undefined && state.backendEpoch !== this.acceptedEpoch) {
      // An event from the previously accepted backend cannot veto a bootstrap
      // that establishes a different backend epoch.
      this.stagedEvents.delete(this.acceptedEpoch);
    }
    const stagedState = this.stagedEvents.get(state.backendEpoch);
    // A response cannot order other opaque epochs that published while the
    // bootstrap was in flight. Keep their evidence and reject ambiguity.
    const hasForeignEpoch = [...this.stagedEvents.keys()].some((epoch) => epoch !== state.backendEpoch);
    if (hasForeignEpoch) {
      // Epochs are opaque, so their order cannot establish which state wins.
      // Keep all staged epochs until the ambiguity has a safe resolution.
      return undefined;
    }
    this.stagedEvents.delete(state.backendEpoch);
    // A bootstrap for the staged epoch establishes the new authority; older
    // staged epochs can no longer participate in this transaction.
    this.stagedEvents.clear();
    const candidate = stagedState && revision(stagedState) > revision(state) ? stagedState : state;
    if (candidate.backendEpoch === this.acceptedEpoch && revision(candidate) <= this.acceptedRevision) {
      if (wasQuarantined && revision(candidate) === this.acceptedRevision) {
        // A valid bootstrap is the only recovery boundary after a protocol
        // fault. Re-publish its snapshot even when its revision is unchanged
        // so the UI can leave the error surface and reopen live lanes.
        this.quarantined = false;
        return candidate;
      }
      return undefined;
    }
    this.acceptedEpoch = candidate.backendEpoch;
    this.acceptedRevision = revision(candidate);
    this.quarantined = false;
    return candidate;
  }

  accept(state: HubShellState, source: Exclude<HubStateSource, "bootstrap">): HubShellState | undefined {
    if (this.quarantined) {
      return undefined;
    }
    if (this.bootstrapInFlight) {
      // A bootstrap is a transaction boundary. Events that arrive before it
      // resolves are staged by epoch and replayed only by a matching future
      // bootstrap; invoke responses must wait for the transaction as well.
      if (source === "event") {
        this.stageEvent(state);
      }
      return undefined;
    }
    if (this.stagedEvents.size > 0) {
      // A stale bootstrap response may have left evidence for a different
      // backend epoch. Keep every live lane quarantined until that epoch's
      // bootstrap establishes the next authority.
      return undefined;
    }
    if (this.acceptedEpoch === undefined) {
      return undefined;
    }
    if (state.backendEpoch !== this.acceptedEpoch || revision(state) <= this.acceptedRevision) {
      return undefined;
    }
    this.acceptedRevision = revision(state);
    return state;
  }

  private stageEvent(state: HubShellState) {
    const stagedState = this.stagedEvents.get(state.backendEpoch);
    if (stagedState && revision(stagedState) >= revision(state)) {
      return;
    }
    if (!stagedState && this.stagedEvents.size >= MAX_STAGED_EVENT_EPOCHS) {
      const oldestEpoch = this.stagedEvents.keys().next().value;
      if (oldestEpoch !== undefined) {
        this.stagedEvents.delete(oldestEpoch);
      }
    }
    this.stagedEvents.set(state.backendEpoch, state);
  }
}

export function acceptHubState(
  chronology: HubStateChronology,
  state: HubShellState,
  source: HubStateSource,
): HubShellState | undefined {
  return source === "bootstrap" ? chronology.acceptBootstrap(state) : chronology.accept(state, source);
}
