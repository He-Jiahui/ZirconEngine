import type { HubShellState } from "../types/hub";

export type HubStateLoadErrorKind = "backend-unavailable" | "protocol-mismatch";

export class HubStateLoadError extends Error {
  readonly kind: HubStateLoadErrorKind;
  readonly cause: unknown;

  constructor(kind: HubStateLoadErrorKind, cause: unknown) {
    super(kind === "backend-unavailable" ? "Hub backend is unavailable" : "Hub state protocol mismatch");
    this.name = "HubStateLoadError";
    this.kind = kind;
    this.cause = cause;
  }
}

export type HubBootstrapOutcome =
  | { status: "booting" }
  | { status: "ready"; state: HubShellState }
  | { status: "backend-unavailable"; error: unknown }
  | { status: "protocol-mismatch"; error: unknown };

export async function resolveHubBootstrap(
  loadState: () => Promise<HubShellState>,
): Promise<HubBootstrapOutcome> {
  try {
    return { status: "ready", state: await loadState() };
  } catch (error) {
    if (error instanceof HubStateLoadError) {
      return { status: error.kind, error: error.cause };
    }
    return { status: "backend-unavailable", error };
  }
}

export function canDispatchHubAction(outcome: HubBootstrapOutcome): outcome is Extract<HubBootstrapOutcome, { status: "ready" }> {
  return outcome.status === "ready";
}
