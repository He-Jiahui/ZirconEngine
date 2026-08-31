---
title: Plugins21 Shared Discovery Input Path
category: zircon_plugins
report_id: Plugins21-shared-discovery-input-path-2026-08-26
date: 2026-08-26
session_id: root-runtime-interface03-activate-link-failure-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Plugins21 shared discovery input path

## Scope

- Parent scope: Plugins21 native-plugin discovery and immutable publication, specifically carrying a load-manifest export root through refresh admission, active/pending state, worker launch, completion, lookup, and publication.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`; `contract.rs` source blob `759d020e113114d5790423c351d5839dee1aca5e`.
- Owners: `NativePluginDiscoveryRefreshInput` storage and its direct Rust contract, the focused source contract, the shared projection/sort/discovery pressure model and contract, and this record.
- This slice preserves root-scan zero-payload identity, exact load-manifest path equality/order/hash behavior, automatic path borrowing at the collector boundary, generation isolation, and immutable publication. It does not change filesystem discovery, admission/trust policy, artifact verification, installation, dynamic-library loading, or the remaining Plugins21 acceptance gates.

## Change

- The internal `LoadManifest` refresh input now stores its export root as `Arc<PathBuf>` instead of an owned `PathBuf`.
- Construction establishes one shared path owner. Existing input clones used by keys, active/pending state, tasks, completions, and snapshots now clone only the `Arc` handle rather than each path buffer.
- `RootScan` remains a payload-free enum variant.
- A direct Rust contract clones a load-manifest input and proves both variants share the same export-root allocation with `Arc::ptr_eq`.

The refresh input is immutable after construction and is intentionally cloned across multiple asynchronous owners. Sharing only its path buffer removes repeated deep copies without changing the service surface or allowing a caller to mutate the canonical selection identity.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_plugins21_shared_discovery_input_path_performance_contract -v` initially reported 3 failures and 1 existing pass because shared storage, shared construction, and the direct Rust contract were absent while RootScan was already payload-free.
- GREEN: the same focused source contract passes 4/4 after implementation.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` and scoped `git diff --check` pass.
- The standalone model is compiled with `rustc 1.94.1 -O`; it does not use Cargo or the shared build lane.

The deterministic model measures 31 alternating owned-path/shared-path sample pairs. Each sample constructs 262,144 long export-root inputs and performs six clones per input, matching the repeated ownership transfers in the refresh service. Construction cost is included: the shared variant pays for both the original path buffer and its `Arc`. Every pair produces checksum `10711012688504291325` for both representations. Four sequential enlarged-workload runs passed the acceptance thresholds; the table records the final run.

| Metric | Owned `PathBuf` input | Shared `Arc<PathBuf>` input | Change |
|---|---:|---:|---:|
| P50 | 198.7112 ms | 77.9511 ms | -60.772% |
| P95 | 361.4602 ms | 144.8218 ms | -59.934% |
| allocations / 262,144 inputs | 1,835,008 | 524,288 | -71.429% |

The other three enlarged-workload runs produced P50 reductions of 58.972%, 61.729%, and 69.247%; P95 reductions of 53.327%, 7.482%, and 50.834%; and the same 71.429% allocation reduction. The one lower P95 reduction reflects host scheduling noise while median and exact allocation evidence remained stable. These timings isolate CPU input construction/cloning and do not claim complete discovery scan or plugin-load latency.

## Async validation

### Current-source convergence receipt

- Ownership transfer preview request: `d7b0d0d9f6674ac29b4a1c63800c853c`.
- Ownership transfer apply request: `c776f5230c1449a7ad53d90d3b5ece4a`.
- Applied fingerprint: `0caacce1f20a3d0c48dd892678b0040ce9a71872faa7cc03e9a89dee4d792f0f`.
- Current session: `root-runtime-interface03-activate-link-failure-20260831`.
- Shared static/model ticket: `4c6aa5481e1440819e427ac1568979ab` (queued, 20 Python tests).
- Plugins21 Cargo behavior ticket: `8de2bc7a35914821915f40d3c8f037e4` (queued; exact shared-owner test).
- Shared model: `tools/plugins_projection_sort_discovery_pressure.py`, source manifest `ECAB605B9EC8C342B696C4CEB59AE2FB270F75976140B7086BFDD497CC18CC05`.
- Current source hashes: `contract.rs` `380AFB0548365B2C70B2D79ECD25D6EFAB738A4235B17C498F786AB0D99385E3`; shared model `2F51A8F953D139FE68E7DB1AB87653DFCC769DA01571BC6CC429040974D34323`.

The current-source model is deterministic structural evidence, not wall-clock timing. For 262,144 inputs with six clones each, the owned path performs `1,835,008` path-owner allocations while the shared path performs `524,288`, a `71.428571%` reduction; deep path-clone allocations change `1,572,864 -> 0` and are replaced by the same count of shared-handle clones. Historical alternating release evidence remains the timing claim: P50 `198.7112 -> 77.9511 ms` (`-60.772%`), P95 `361.4602 -> 144.8218 ms` (`-59.934%`), checksum `10711012688504291325`.

The coordinator must run the four focused Python source contracts, the discovery-refresh contract Rust test module in one Cargo filter, Rust formatting checks, scoped diff checks, exact model parity, and the enlarged performance workload. This ticket joins the outstanding optimization validation queue and is reconciled with the batch; the business Session does not wait for it before the next slice.

Acceptance requires 4/4 source contracts and the exact filtered Rust test count to pass, identical checksum `10711012688504291325`, `1,835,008` legacy versus `524,288` shared allocations, a P50 reduction of at least 50%, no P95 regression, and an allocation reduction of at least 70%. Cargo validation remains required even while the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` prevents workspace compile-time input closure planning. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and allocation reductions and label them as CPU load-manifest discovery-input evidence for 262,144 inputs with six clones each.
