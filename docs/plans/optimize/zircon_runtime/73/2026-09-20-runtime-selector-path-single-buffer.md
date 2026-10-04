---
title: Runtime854 runtime selector path single-buffer
category: zircon_runtime
report_id: Runtime854-runtime-selector-path-single-buffer-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime854 · runtime selector path single-buffer

## Scope

`runtime_selector_path` is called by both the retained-tree subtree walk and
single-node runtime style application. It previously collected the target-to-
root IDs in one `Vec`, reversed that vector, and then allocated a second path
`Vec` while looking up every node again. This slice keeps the same ancestor
order and `:host` semantics while constructing `SelectorPathNode` values
directly in one reversible buffer.

The shared `style.rs` owner also contains the adjacent Runtime785 capacity,
wildcard-selector, and Runtime853 retain changes. This record attributes only
the selector-path allocation cutover and preserves those existing changes.

## Optimization

- Build `SelectorPathNode` values while walking from the target toward the
  root, using the existing component-state lookup exactly once per node.
- Reverse the path in place and set only the first element's `is_host` bit,
  preserving the old root-first path and host matching behavior.
- Remove the intermediate ancestor-ID vector and the second node-map lookup
  loop; no selector parsing, matching, state collection, or tree ownership
  contract changes.

## TDD and deterministic evidence

The new Python source/model contract was intentionally run RED before the
implementation and lower module were wired (`3` structural failures), then
GREEN at `4/4`. The lower Rust regression builds a three-level tree, verifies
root-to-leaf component order, and checks that only the root is marked host.
The ignored Release probe emits
`RUNTIME854_RUNTIME_SELECTOR_PATH_SINGLE_BUFFER_BENCH_V1` with alternating
legacy/optimized samples and P50/P95/P99 fields. A depth-128 model changes the
intermediate ID-buffer count from `1` to `0` per path build.

## Local validation

- `tools/tests/test_runtime_selector_path_single_buffer_performance_contract.py`:
  `4/4`.
- Focused Runtime V2 style batch (selector-path, filter-retain, rule-capacity,
  pseudo-state-capacity contracts): `14/14`, zero failures/errors/skips.
- Exact-file `rustfmt --edition 2021 --check` passes for `style.rs` and the
  lower selector-path owner; scoped diff checks report no content errors.
- The latest widened one-process non-tooling loader covers `957` contract files
  and passes `4042/4042` tests in `125.129s`, with zero load errors, failures,
  errors, or skips. The focused V2 style batch is `18/18` after Runtime855.
- The current expanded source-contract loader covers `962` non-tooling files
  under the explicit performance-or-contract filename filter and passes
  `4072/4072` tests in `139.499s`, with zero load errors, failures, errors, or
  skips; the `957`/`4042` receipt remains pre-Editor856 historical context.
- No managed Windows Cargo/Release validation command was started locally.
  Allocator observations and product selector-style p50/p95/p99 evidence remain
  pending behind the shared external worktree gate.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/v2/style.rs` | `C7A9BDABD7E0DA0CC98595662DFD7858CEDB2B6827C323D1339B20174A145DCE` |
| `zircon_runtime/src/ui/v2/style/selector_path_tests.rs` | `082C6C96269932C99B33EAAAAC69510708606ED19A15D5255C7FDF5B4203A7E5` |
| `tools/tests/test_runtime_selector_path_single_buffer_performance_contract.py` | `8CEBCE81653879B4478D06E4DD6245219BDEFDEA84EDCE1A6BFD6D1253B7D7B3` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Runtime tree,
executes the lower regression and ignored marker, and supplies allocator plus
product selector-style p50/p95/p99 measurements. Tooling production remains
deferred for the later Rust migration.
