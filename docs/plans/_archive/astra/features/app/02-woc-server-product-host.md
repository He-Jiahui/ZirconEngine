---
status: implemented_pending_validation
source_recheck_required: true
plan_sources:
  - docs/plans/astra/optimize/01-review-and-repair.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_app/05-woc-native-server-bot-headless-service-tick-replication-persistence-operations-product-integration-review.md
---

# WOC Server Product Host

`woc_server` now has a real local authoritative composition root. Its binary
enables the `engine-host` feature, loads the project through
`woc_runtime::load_engine_project_vm` with the server role, activates the
ZrVM-backed package, and links the library-owned `FixedServerTickDriver`.
Each requested step advances exactly one `SERVER_TICK_NS` boundary, which
aliases the protocol-owned `woc_protocol::SIMULATION_STEP_NS`. The
optional replay check checkpoints the runtime, executes a tick, restores the
checkpoint, and requires the next tick's committed snapshot to be identical.
The VM is transferred back to the host and deactivated on every terminal path.
The report explicitly marks `presentationReady=false` and `renderedFrames=0`;
this role is a local simulation runner, not a visual client or network service.

This closes the former identity-only server entrance and supplies a bounded,
deterministic local simulation runner. It does not claim network ingress,
replication, durable persistence, authenticated principals, or a desktop
client; those remain the owners of the service/client plans. The product
subprocess test and source contract are evidence pending the managed Windows
Cargo lane, real external ZrVM dependency admission, and release p50/p95/p99
measurements.

## Evidence

- `tools/tests/test_woc_server_product_host_contract.py` checks the binary,
  feature gate, library link, activation/deactivation, tick boundary and
  replay path.
- `examples/woc/native/apps/woc_server/tests/product.rs` runs the packaged
  binary for two ticks plus a checkpoint replay when `engine-host` is enabled.
- Current source checks pass; managed Cargo remains inadmissible while the
  external `E:/Git/zr_vm` worktree is dirty.
