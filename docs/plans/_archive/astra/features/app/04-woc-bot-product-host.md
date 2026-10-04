---
status: partially_implemented
review_date: 2026-09-09
plan_sources:
  - docs/plans/optimize/zircon_app/05-woc-native-server-bot-headless-service-tick-replication-persistence-operations-product-integration-review.md
---

# WOC Bot Product Host

`woc_bot` now enters an engine-owned local deterministic runner instead of
printing project identity and exiting. The runner loads the authored project
through `load_engine_project_vm` with the Bot role, activates the real
`ZrVmProjectVm`, installs a validated offline bootstrap, executes bounded
fixed ticks, and explicitly deactivates the VM on every terminal path.

The local policy is intentionally a no-op action policy: each step observes the
committed state digest and submits an empty command batch. This proves the
episode and transaction lifecycle without inventing gameplay decisions or
claiming a transport implementation. Optional checkpoint replay requires
byte-identical committed snapshots after rollback.

The report marks `mode=local_deterministic`, `realVm=true`, and
`networkReady=false`, `persistenceReady=false`, and `renderReady=false`.
Network principals, authenticated sessions, replication, durable journal/
checkpoint storage, and a visual client remain open service owners.

## Qualification Status

| Area | Status | Evidence / remaining gate |
|---|---|---|
| Binary reachability | `implemented_pending_validation` | `src/main.rs` -> `run.rs` -> `BotProductHost`; managed Windows build and packaged smoke still pending |
| Real VM and episode ticks | `implemented_pending_validation` | engine host adapter, offline bootstrap, bounded tick loop, and replay path; Cargo admission remains pending |
| Lifecycle cleanup | `implemented_pending_validation` | explicit activation/deactivation and failure cleanup; managed process evidence pending |
| Network/persistence/render providers | `partially_implemented` | explicitly reported unavailable until their service owners are composed |

Static contracts do not close the Bot product gate. Acceptance still requires a
managed Windows Release run, a versioned observation/action schema, principal
and rate policy, transport or in-process mode qualification, durable recovery,
and long-running performance evidence.
