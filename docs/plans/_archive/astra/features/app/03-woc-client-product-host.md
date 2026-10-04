---
status: partially_implemented
review_date: 2026-09-09
plan_sources:
  - docs/plans/optimize/zircon_app/04-woc-native-client-window-input-shell-ui-presentation-frame-product-integration-review.md
  - docs/plans/optimize/zircon_app/05-woc-native-server-bot-headless-service-tick-replication-persistence-operations-product-integration-review.md
---

# WOC Client Product Host

The client binary no longer exits after an identity-only report. Its product
entry constructs `ClientProductHost`, loads the authored WOC project through
the engine host, activates a real `ZrVmProjectVm`, installs an offline session,
and drives bounded protocol-derived simulation frames (currently 20 Hz) before
explicit deactivation.

With no explicit `--project`, the binary resolves the checked-in authored
project root at `examples/woc` (which owns `zircon-project.toml`); the engine
host then resolves its `scripts/woc_game` package. Callers can override the
root for a fixture or packaged project.

## Current Boundary

The current project still returns an empty `presentation_payload` from
`fixedTick`. The host therefore uses `StateOnlyClientAuthority` and reports
`stateOnly=true`, `presentationReady=false`, `nativeWindowReady=false`, and
`renderedFrames=0`. It does not synthesize actor, HUD, window, GPU, audio, or
network output from world-state bytes.

The model-level auth boundary stores password, reset token, TOTP, and recovery
data in `AuthSecret`: diagnostics are redacted and clear/drop overwrite the
current byte buffer. Reset tokens are bounded and reject whitespace/control
values. The shell now correlates each auth and password-reset completion with
an opaque request identity and rejects stale or wrong-operation responses.
This reduces ordinary in-process leakage and UI race risk only; a credential
vault, TLS transport/session correlation, cancellation, expiry/origin
validation, and server authorization are still absent.

Realm status probes are also tied to a monotonically advancing directory
generation; late results from a replaced directory are rejected before they
can change status or recommendation state.

Analog input helpers reject non-finite/out-of-range calibration values, and the
frame driver rejects wall-clock deltas above its bounded suspend threshold before
touching the accumulator or queued commands.

The lifecycle report records `constructing`, `ready`, `running`, `stopping`,
and `stopped`. Session construction and offline bootstrap failures return the
authority to the host so an activated VM can be deactivated before the error
is returned. The shell also exposes an explicit preparation rejection path:
failures from `Welcome` or `Loading` clear the pending launch and return to the
picker with the user's draft intact, while an `InWorld` session cannot be
rolled back. Optional replay verification checkpoints the same real VM and
requires byte-identical committed snapshots after rollback.

## Qualification Status

| Area | Status | Evidence / remaining gate |
|---|---|---|
| Binary reachability | `implemented_pending_validation` | `src/main.rs` -> `run.rs` -> `ClientProductHost`; managed Windows build and packaged smoke still pending |
| Real project VM and fixed ticks | `implemented_pending_validation` | `load_engine_project_vm`, `StateOnlyClientAuthority`, bounded client frame loop; managed Cargo and runtime execution pending |
| Lifecycle cleanup | `implemented_pending_validation` | explicit deactivation on success and construction/bootstrap failure; shell preparation rejection is retryable and draft-preserving; injected VM cleanup tests and managed run pending |
| Presentation/window/render/audio/network | `partially_implemented` | intentionally unavailable until the project emits a validated presentation projection and engine-owned native services are composed |

Pure model tests, static source contracts, and the state-only report do not
close the native client P0. Product acceptance still requires a clean managed
Windows Release build, a real window/event loop, GPU present/screenshot,
transport/session identity, persistence, and shutdown evidence.
