---
title: Tooling29 single-pass runtime module inventory
category: zircon_tooling
report_id: Tooling29
date: 2026-08-31
baseline_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
baseline_epoch: 575
status: release_validation_submitted
session: root-tooling29-single-pass-inventory-release-r2-20260831
validation_request_id: b698b0810e5a406b9c69dd0fd7d7a199
implementation_files:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py
tests:
  - tools/tests/test_tooling29_single_pass_module_inventory_performance_contract.py
---

# Tooling29 single-pass runtime module inventory

## Problem

`runtime_inventory` read every production Rust file four times: once each for module descriptor locations, stub descriptor locations, `EngineModule` owners, and line-count hotspots. On the live repository, the legacy scan touched 11,634 production files through 46,537 `_read_text` calls and read about 200.44 MB before any downstream structure audit ran.

## Change

`_scan_production_file` now reads and splits each source once, then gathers descriptor locations, stub locations, owner implementations, and line count in one ordered loop. Crate ordering, file ordering, line ordering, snippets, production-file exclusions, hotspot thresholds, and report shapes remain unchanged.

The performance contract builds a temporary workspace and asserts that the workspace manifest and every included production Rust file are read exactly once. It also verifies descriptor, stub, owner, hotspot, and test-file exclusion semantics. Its acceptance case invokes the production `runtime_inventory` implementation and compares the complete `RuntimeInventory` value with a test-only reconstruction of the former four-pass implementation before measuring either path.

## Historical live-tree evidence

Acceptance thresholds were at least 70% fewer source reads and bytes plus at least 20% P50 latency reduction.

| Measurement | Legacy | Optimized | Reduction |
|---|---:|---:|---:|
| Source reads at the final 11,636-file corpus | 46,545 | 11,637 | 75.00% |
| Source bytes at the final corpus | 200,558,957 | 50,141,375 | 75.00% |
| Three-sample P50 | 42,256,949,300 ns | 27,796,195,600 ns | 34.22% |
| Three-sample P95 | 42,379,447,300 ns | 31,872,605,500 ns | 24.79% |
| Same-corpus legacy/current comparison | 64,801,769,700 ns | 31,872,605,500 ns | 50.82% |

Legacy samples were `38284358800, 42256949300, 42379447300` ns. Optimized samples were `31872605500, 27796195600, 23213287000` ns. The shared worktree added two production Rust files between the legacy three-sample run and optimized run; the same-corpus comparison used the later 11,636-file tree and compared the Git `HEAD` implementation against the current implementation.

The same-corpus structured comparison was exactly equal across crate inventory, production paths, module/support classification, hotspots, descriptor locations, stub locations, and owner implementations. The final result contained 12 hotspots, 14 descriptor locations, zero stub locations, and 18 owners.

These measurements were captured on 2026-08-27 and remain historical evidence because the shared repository corpus changes continuously. They are not the terminal release acceptance result.

## Repeatable acceptance preflight

The 2026-08-31 Windows preflight used one temporary workspace crate with 32 production files of 2,048 lines each. Every file contains one descriptor, one stub reference, and one `EngineModule` owner. After four warm-up pairs, it measured 21 alternating legacy/current pairs so either implementation led approximately half the samples. Both implementations returned the same complete inventory and checksum `160`.

| Measurement | Legacy | Optimized | Reduction | Gate |
|---|---:|---:|---:|---:|
| Reads | 129 | 33 | 74.4186% | >= 70% |
| Bytes | 8,129,155 | 2,032,321 | 74.9996% | >= 70% |
| P50 | 520,432,900 ns | 279,904,400 ns | 46.2170% | >= 20% |
| P95 | 1,048,049,000 ns | 702,095,100 ns | 33.0093% | >= 20% |

Legacy raw samples in nanoseconds were `[452750300, 521040200, 388452700, 340488700, 619152300, 474133300, 491258000, 458639900, 385615900, 391996400, 520432900, 637200000, 365075200, 597024300, 1048049000, 1020343000, 841821200, 1067903200, 917640800, 839732100, 375504000]`.

Optimized raw samples in nanoseconds were `[287451900, 257062900, 289244800, 331876500, 254628300, 262925700, 257551800, 357343500, 197597300, 167645700, 211944800, 221176500, 216889100, 279904400, 708809300, 651918800, 658416200, 702095100, 320744100, 458292600, 266091800]`.

This preflight is reproducible acceptance evidence, but it is not terminal until the coordinator executes the snapshotted request.

## Validation

- Red phase: the original performance contract failed because the legacy implementation read the fixture `lib.rs` four times.
- Green phase: `python -m unittest tools.tests.test_tooling29_single_pass_module_inventory_performance_contract` passed 2/2 locally, including the production-path latency and structural acceptance benchmark above.
- `python -m py_compile` passed for the implementation and contract.
- `git diff --check` passed.
- The complete `audit_runtime_structure.py --json` command exceeded its 180-second local timeout in other audit domains. The two exact Python processes started by that command were terminated and confirmed absent; no Cargo or coordinator process was touched. Full coordinator validation remains the authoritative broad check.
- Coordinator request `b698b0810e5a406b9c69dd0fd7d7a199` will run `python -m unittest tools.tests.test_tooling29_single_pass_module_inventory_performance_contract` from an exact three-path snapshot. Expected result: 2 tests passed with the benchmark marker and all four gates above satisfied.
