---
title: Runtime200 Route Terminal State Bit
category: zircon_runtime
report_id: Runtime200-route-terminal-state-bit-2026-09-14
date: 2026-09-14
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime200 Route Terminal State Bit

## Scope

`surface/input/route_steps.rs` appended an out-of-route default-action step
only after scanning the accumulated route-step vector for a stopped row. The
route builder already returns immediately for preview/target stops and breaks
only when an ancestor is the terminal handled/blocked step. The implementation
now carries that loop state as a boolean and passes it to the append helper,
removing the repeated `O(H)` membership scan without changing route ordering,
stop disposition, or capacity reservation.

This is a constant-factor input-diagnostics improvement. It does not claim the
larger Runtime200 diagnostics-capture policy migration, which remains open.

## Complexity and deterministic target

For a routed path of depth `H`, the terminal append guard changes from a scan of
up to `H` rows to one state-bit check. The route construction itself remains
`O(H)` and the public DTO is unchanged. The ignored release benchmark uses a
4,096-row path, alternating legacy and optimized probes, and emits
`RUNTIME200_ROUTE_TERMINAL_SCAN_BENCH_V1`; acceptance requires the optimized
P95 to be at least 50% below the full scan.

## TDD and local evidence

- The source contract was RED before implementation because the production
  helper contained `steps.iter().any(|step| step.stopped)` and no loop state
  parameter.
- The focused Runtime route contract passes `3/3`.
- The combined Runtime/Editor pointer, reference, history, and input contract
  batch now passes `76/76` in one invocation after Runtime751 joined it.
- The full current Editor performance-contract discovery passes `622/622` and
  the full current Runtime performance-contract discovery passes `1201/1201`;
  these were run as separate broad batches to stay within Windows command-line
  limits.
- A subsequent single-process cross-surface discovery loaded both patterns and
  passed `1823/1823` tests in `22.612s`; this is a local source/model receipt,
  not managed Cargo or Release evidence.
- The Rust regression covers appended, already-stopped, and unhandled route
  semantics. Scoped Rustfmt, Python compilation, and `git diff --check` pass.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/input/route_steps.rs` | `A76F6137B4AEF851A84CA94DF5D3247782EADC97D2632E5CFE95B83FFCB5B409` |
| `zircon_runtime/src/ui/surface/input/route_steps_tests.rs` | `3387D739327055D90D5213F539C246EB1FE66E52A9B800C049F4002EA3790389` |
| `tools/tests/test_runtime_ui_route_terminal_scan_performance_contract.py` | `AF72D6F5D4ADA614C84515894ED6449F65CF8E5C3B5773743308B2102AF1AA41` |

## Managed gate

This slice joins the existing owner-attributed Runtime/Editor Windows Release
batch. Managed compilation, allocation, and product input p50/p95/p99 evidence
remain pending under the shared external-worktree admission blocker; no
standalone Cargo run or coordinator status polling is required. Tooling
production work remains deferred.
