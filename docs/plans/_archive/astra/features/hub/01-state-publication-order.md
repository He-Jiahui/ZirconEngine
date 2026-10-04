---
status: in_progress
plan_family: docs/plans/astra/features/hub
parent_plan: docs/plans/astra/optimize/01-review-and-repair.md
owner_session: astra-hub-revision-20260905
---

# Hub State Publication Order

## Problem And Boundary

Hub commands and background/focus workers currently build `HubViewModel` snapshots while holding the session mutex, release the mutex, and emit later. A newer mutation can therefore reach the caller and React before an older delayed event, allowing the old event to overwrite current state. This plan owns the Hub Rust view model/session publication contract, its TypeScript wire parser and acceptance gate, and focused chronology fixtures. It does not own the Hub local service or Cargo manifests.

## Contract

- Every backend session has one opaque `backendEpoch` generated at load and one monotonic `stateRevision` assigned while the authoritative `HubRuntimeSession` mutex is held.
- `stateRevision` is serialized as a decimal string so JavaScript never rounds a future `u64` value; all invoke responses, bootstrap payloads, and `hub-state-changed` events carry the same pair.
- A client accepts a state only when it is the first bootstrap for a new epoch, or when its epoch matches the accepted epoch and its revision is strictly greater than the accepted revision. Stale and duplicate same-epoch states are ignored.
- A different epoch is never accepted from an event or ordinary invoke response. It becomes current only through a fresh bootstrap transaction, preventing delayed events from a retired backend from resetting the UI.
- Fallback fixtures use a stable synthetic epoch and revision `0`; they remain valid for non-Tauri rendering and are checked by the same parser.

## Milestones

### M1: Authoritative Rust publication

Add epoch/revision fields to `HubRuntimeSession` and `HubViewModel`, include them in the snapshot projection, and centralize mutation publication so command responses, focus refresh, and background worker emissions receive the revision assigned under the session mutex. Preserve existing A1-A4 command and focus behavior. Add Rust chronology tests covering strictly increasing revisions, delayed event snapshots, duplicate rejection inputs, and epoch identity.

### M2: TypeScript protocol and acceptance

Extend `HubShellState`, fallback data, and the runtime validator with lossless chronology fields. Add a small chronology acceptance helper shared by `loadHubState`, `dispatchHubAction`, bootstrap resolution, and event subscription. Update `App` to accept bootstrap as the only epoch rollover path and to ignore stale/duplicate events or invoke results. Add Node fixtures for same-epoch ordering, duplicate delivery, delayed old epoch, and large decimal revisions.

### M3: Verification and browser regression

Run the consolidated Hub typecheck and Node test suites once after M1-M2. Run the existing browser regression suite against its available Hub URL, including a delayed-event scenario that attempts to overwrite a newer action response. Record exact command output and any unavailable browser environment without converting skipped browser cases into passing claims.

## Testing Stage

The testing stage is one batched pass after implementation: focused Rust chronology tests through the existing Hub test target, `npm run typecheck`, `node --test` for all Hub web tests, and the existing Playwright browser regression command when `ZIRCON_HUB_TEST_URL` and its configured browser runtime are available. Any failure is debugged from the lowest shared parser/publication layer upward, then the complete batch is rerun. No Cargo manifest or dependency version changes are permitted.

## Acceptance Evidence

- Rust chronology tests prove a delayed older snapshot cannot be newer than the authoritative mutex revision and that epoch changes are explicit.
- TypeScript/Node tests prove decimal revisions remain exact, same-epoch stale/duplicate states are rejected, and delayed old-epoch events cannot reset accepted state.
- Browser regression proves the rendered state remains at the newer action result after an older event is delivered.
- `git diff --check` is clean for the owned files; existing dirty worktree content outside this scope is preserved.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M2/M3 web chronology candidate | Lossless epoch/revision parsing, stale-event rejection, browser state regression, and responsive dialog evidence | `browser_fixture_validated_rust_pending` | 2026-09-11 | A Chrome-backed Vite fixture exposed a `React.StrictMode` double-bootstrap race in the test harness, not in the chronology contract: the fixture now shares one bootstrap release gate and replays an already-emitted state to the latest listener. The shared-gate focused bootstrap case passed five consecutive runs; a subsequent focused run with an explicit bootstrap-start condition also passed. An earlier complete serial `hub_browser_regressions.test.mjs` run passed 10/10, covering delayed events, bootstrap faults, malformed action state, and 360/768/1280/1920 layouts. The full Web source suite passed 188 tests (115 pass, 73 environment-skipped), and `npm run typecheck` plus scoped diff checks passed. A later repeat reached 9/10 assertions but the 768px layout harness reported Vite HMR WebSocket `ERR_NO_BUFFER_SPACE`; that host-resource failure is inconclusive and must be rerun on a healthy browser host. All of this remains Web fixture evidence only: managed Rust chronology tests and real Tauri publication acceptance remain pending. |
| M2/M3 mixed-epoch follow-up | Bootstrap/event epoch reconciliation | `implemented_pending_validation` | 2026-09-26 | `node --test web/tests/hub_ui_regressions.test.mjs` passed 22/22 and `npm run typecheck` passed. Independent source review found no further safety issue in this bounded change. Ambiguous B+C epochs stay quarantined; liveness needs backend authoritative ordering/subscription or an explicit reset contract. The existing four-epoch staging cap remains a separate evidence-retention risk. Managed Rust, real Tauri publication, and browser regression gates remain pending; no acceptance promotion. |
| M2/M3 stale-error follow-up | Older malformed events and delayed window-action failures | `implemented_pending_validation` | 2026-09-27 | Focused regressions were RED before repair; `node --test web/tests/hub_ui_regressions.test.mjs web/tests/window_action_scheduler.test.mjs web/tests/hub_window_drag_regressions.test.mjs` passed 30/30 after repair. `npm.cmd run typecheck`, browser-test syntax, and scoped diff checks passed; independent source and corrected-test review found no actionable issue. Browser fixture execution was unavailable without a configured URL and Playwright runtime. Managed Rust chronology and real Tauri publication gates remain pending; no acceptance promotion. |
