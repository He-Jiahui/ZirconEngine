---
related_code:
  - zircon_editor/src/tests/ui/boundary/template_assets/invalidation.rs
  - zircon_editor/src/tests/host/retained_drawer_resize/surface_contract.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-27-retained-pointer-structural-gate-owner-paths.md
status: implemented_pending_validation
---

# Editor1023 / Editor01 retained pointer structural gates completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| Pointer callback source ownership | Ten groups now check the actual parent and child owners for a committed-layout call; each owner retains the no-recompute and no-chrome-snapshot constraints. The lifecycle entrypoint is checked in `host_lifecycle/tick.rs`. | Pre-repair static replay found eight parent-only positives absent. Current source-shape replay passed; managed Editor lib execution is pending. | implemented_pending_validation |
| Dirty-domain and resize ownership | Dirty writes are allowed only at the exact invalidation owner, while all other scanned files keep the original forbidden checks. Resize capture, wiring, drag geometry and input dispatch checks now use their implementation children. | Source replay passed 37/37 post-repair checks; 9/9 pre-repair checks identified expected failures. Existing real-host toolbar and resize behavior tests remain the dynamic regression gate. No production source changed. | implemented_pending_validation |
| Editor01 product performance | This repair restores meaningful structural guards only. | Pointer-time slow-path counts, CPU/allocator/RSS and latency product gates remain pending; no speedup or dynamic pass is claimed. | product_gate_pending |

The Batch S preimage, static replay, inverse diff, and source manifest are
stored under `.codex/state/session-coordinator/async-validation-batches/`.
