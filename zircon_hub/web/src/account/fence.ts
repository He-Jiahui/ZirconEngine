import type { AccountSnapshot } from "./protocol";

export interface AccountResponseToken {
  sequence: number;
  scope: string;
  session: number;
  backendEpoch: string;
  generation?: string;
}

export class AccountResponseFence {
  private session = 0;
  private scopes = new Map<string, number>();
  private backendEpoch: string;

  constructor(backendEpoch: string) { this.backendEpoch = backendEpoch; }

  reset(backendEpoch: string) {
    this.backendEpoch = backendEpoch;
    this.session += 1;
    this.scopes.clear();
  }

  invalidate(scope: string) { this.scopes.set(scope, (this.scopes.get(scope) ?? 0) + 1); }

  begin(generation?: string, scope = "account"): AccountResponseToken {
    this.invalidate(scope);
    return { sequence: this.scopes.get(scope)!, scope, session: this.session, backendEpoch: this.backendEpoch, generation };
  }

  current(token: AccountResponseToken, currentGeneration: string): boolean {
    return token.session === this.session && token.sequence === this.scopes.get(token.scope)
      && token.backendEpoch === this.backendEpoch
      && (token.generation === undefined || token.generation === currentGeneration);
  }

  accepts(token: AccountResponseToken, response: AccountSnapshot, currentGeneration: string): boolean {
    return this.current(token, currentGeneration) && response.backendEpoch === this.backendEpoch
      && (token.generation === undefined || response.account.generation === token.generation)
      && BigInt(response.account.generation) >= BigInt(currentGeneration);
  }
}

const errorCodes = new Set([
  "account_configuration_invalid", "identity_provider_unavailable", "account_credential_store_unavailable",
  "account_callback_invalid", "account_login_cancelled", "account_login_timed_out", "account_browser_unavailable",
  "account_identity_invalid", "account_session_expired", "account_operation_busy", "account_service_operation_failed",
  "account_service_outcome_unknown", "account_not_configured", "hub_runtime_state_unavailable", "hub_state_epoch_stale",
  "account_protocol_invalid", "account_revision_changed",
  "account_operation_store_unavailable", "account_operation_id_conflict", "account_revocation_pending",
  "account_package_service_unavailable", "account_package_trust_denied", "account_package_capacity_exceeded", "account_package_inventory_conflict",
  "account_package_target_migration_required", "account_package_policy_unconfigured", "account_package_target_unconfigured",
]);

export function accountErrorCode(error: unknown): string {
  const value = error instanceof Error ? error.message : typeof error === "string" ? error : "";
  return errorCodes.has(value) ? value : "account_service_operation_failed";
}
