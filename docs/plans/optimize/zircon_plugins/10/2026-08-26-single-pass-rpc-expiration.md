---
title: Plugins10 Single-Pass RPC Expiration
category: zircon_plugins
report_id: Plugins10-single-pass-rpc-expiration-2026-08-26
date: 2026-08-26
session_id: root-runtime-interface03-activate-link-failure-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Plugins10 single-pass RPC expiration

## Scope

- Parent scope: Plugins10 Network RPC runtime, specifically pending-request deadline cleanup under `NNET-P1-037`.
- Baseline: `601472078e848164d2221967c55a77fea2452928`, epoch `447`.
- Owned paths: `manager/dispatch.rs`, its focused source contract, the shared RPC/routes pressure model and contract, and this record.
- This is a bounded CPU and allocation fix for an existing deadline-cleanup path. It does not claim to close preemptible handlers, canonical transport ownership, bounded executors, cancellation, deduplication, or terminal wire responses.

## Change

`expire_pending_requests` now uses `HashMap::retain` to identify an expired request, build its timeout report, and remove it during one table scan. The previous implementation first materialized every expired request ID, then rehashed each ID through `HashMap::remove`, and finally collected the reports into a second vector.

The timeout boundary is unchanged: a request expires when `timeout_ms == 0` or elapsed whole milliseconds are greater than `timeout_ms`. Reports retain `RpcDispatchStatus::TimedOut` and the `pending RPC request timed out` diagnostic. A direct Rust test seeds one expired and one live request, then verifies the sweep removes and reports only the expired request.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_plugins10_single_pass_rpc_expiration_performance_contract -v` produced 4/4 expected failures against the two-pass implementation.
- GREEN: the focused source contract now passes 4/4.
- `rustfmt +1.94.1 --edition 2021 --check --config skip_children=true` passes for `dispatch.rs`.
- Scoped `git diff --check` passes.
- The standalone model compiles with `rustc 1.94.1 -O`; it does not use Cargo or a shared build target.

The deterministic model measures 31 alternating legacy/single-pass sample pairs over 131,072 pending requests with 32,768 expired requests. Hash-map construction and cloning are outside the timed and allocation-counted region. The modeled report is wider than a request ID so the legacy temporary-ID buffer cannot be unrealistically reused as the output buffer. Both algorithms produced checksum `8727815200911380074` in all four runs.

| Metric | Two-pass scan/remove | Single-pass retain | Change |
|---|---:|---:|---:|
| P50 | 5.1467 ms | 2.0265 ms | -60.6253% |
| P95 | 11.0896 ms | 5.3114 ms | -52.1047% |
| allocations / sweep | 28 | 14 | -50.0000% |
| second-pass hash removals | 32,768 | 0 | -100% |

The other three runs produced P50 reductions of 64.0228%, 51.6015%, and 56.8673%, and P95 reductions of 62.1638%, 33.8948%, and 81.9139%, with identical allocation and hash-removal counts. These values cover CPU pending-request expiration only and do not claim transport latency or end-to-end RPC throughput improvement.

## Async validation

### Current-source convergence receipt

- Ownership transfer preview request: `8cbea769baba4c82ae6debf6c4338d46`.
- Ownership transfer apply request: `dfcd3de2cfd9454dbba46d0615255185`.
- Applied fingerprint: `04fce303e447edc840b141df5e769942aff944994377cd143f3b134d0a5fe2a0`.
- Current session: `root-runtime-interface03-activate-link-failure-20260831`.
- Shared static/model ticket: `dd4881f740a74ea1997f4e20faedb233` (queued, 17 Python tests).
- Plugins10 Cargo behavior ticket: `11945e53ecf442e7917fa876e97af501` (queued; exact expiration behavior test).
- Shared model: `tools/plugins_rpc_routes_pressure.py`, source manifest `C7976604E82B67CA2BC572A3AF78A8DF73B86DFA54D641C591F1D89B86EEF335`.
- Current source hashes: `dispatch.rs` `C2EFBBAE934A66BFD7DF2B200E75AE5BEAFD74D9E16084C3A3DE1715A512B852`; shared model `2C63D72FE8BAFCE0C6885A639A27B1E2E66C1BC2AFBCECAF55FBBC40AF0603D8`.

The current-source model is deterministic structural evidence, not wall-clock timing. For 131,072 pending requests with 32,768 expired, it preserves one table scan and 32,768 report writes while changing expired-ID materialization `32,768 -> 0`, second-pass hash removals `32,768 -> 0`, and temporary collections `2 -> 1`. Historical alternating release evidence remains the performance claim: P50 `5.1467 -> 2.0265 ms` (`-60.6253%`), P95 `11.0896 -> 5.3114 ms` (`-52.1047%`), allocations `28 -> 14` (`-50%`), checksum `8727815200911380074`.

The coordinator must run the four focused source contracts, the exact Rust test `manager::dispatch::expiration_sweep_tests::expiration_sweep_removes_only_timed_out_requests`, Rust formatting, scoped diff checks, checksum parity, and the standalone model. Acceptance requires 4/4 source contracts, exactly one filtered Rust test, checksum `8727815200911380074`, P50 reduction of at least 45%, P95 reduction of at least 20%, allocation reduction of at least 50%, and zero optimized second-pass hash removals.

The ticket joins the outstanding optimization validation batch and this business Session does not wait for it before subsequent work. Cargo remains coordinator-owned. At the time of this record, foreign unmanaged wave91-wave93 build directories and `E:\ZirconBuilds\mvp-resource-management-projects` can stop managed copies at `artifact_governance`; the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` can also stop Cargo input-closure planning. None of those paths belongs to this candidate.

Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include the managed P50/P95, allocation, and second-pass hash-removal reductions and label them as CPU RPC pending-request expiration evidence for the 131,072-request/25%-expired workload.
