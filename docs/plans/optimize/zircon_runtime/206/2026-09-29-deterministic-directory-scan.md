---
title: Runtime206 deterministic metadata directory scan
category: zircon_runtime
report_id: Runtime206-deterministic-directory-scan-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
implementation_status: implemented_pending_validation
validation_status: static_checks_complete_managed_tests_pending
performance_status: measurement_pending
related_code:
  - zircon_runtime/src/asset/registry/rebuild.rs
  - zircon_runtime/src/asset/registry/rebuild/deterministic_scan_tests.rs
tests:
  - metadata_scan_collects_siblings_in_lexical_depth_first_order
plan_sources:
  - docs/plans/optimize/zircon_runtime/206-runtime-asset-registry-project-catalog-index-persistence-rebuild-incremental-query-watch-generation-current-working-tree-review.md
---

# Runtime206 deterministic metadata directory scan

## Change

`ASSETREG-P1-021` identifies filesystem enumeration order as an input to
metadata scanning. `collect_meta_paths` previously processed `fs::read_dir`
directly. The order could change which sidecar first owns a duplicate GUID
and the order of diagnostics emitted during the rebuild.

The collector now gathers each directory's immediate paths and sorts them
before validating entries or descending into child directories. Each root is
therefore traversed in lexical, depth-first path order. The regression checks
the exact `.zmeta` sequence from nested real directories and also passes a
deliberately unordered sibling list to the same traversal logic. That second
assertion fails without the sort even if the filesystem happens to enumerate
names in sorted order. Non-sidecar files are excluded.

The sort costs `O(k log k)` time and `O(k)` temporary paths for a directory
with `k` entries. Parent entry lists remain live during recursive descent,
so peak temporary storage is the sum of immediate entry counts along the
active directory path. The pre-existing full metadata path list is unchanged.
Each sibling list is collected before an unsafe-link error can be reported;
directory-width admission limits remain part of `ASSETREG-P1-028`.
Release-scale scan latency and allocated-byte evidence has not been measured.
No performance improvement is claimed.

## Evidence and limits

| Path | Preimage SHA-256 | Candidate SHA-256 |
|---|---|---|
| `zircon_runtime/src/asset/registry/rebuild.rs` | `2f68e5a840ce10f761742b1a44f8f8c6f001e6294ea13673ca581799d3381249` | `8ee80cd7f347f23dd0687939e7950efed2671fa3ed622942cce3eafa8264317f` |
| `zircon_runtime/src/asset/registry/rebuild/deterministic_scan_tests.rs` | absent | `c55c3c0848a70e740c9871eb800f88dcf9110b1c7cf38325fb98bf5c5d343c67` |

The source preimage already contained an unrelated capacity reservation and
its test module from an archived Session; both were preserved. A read-only
coordinator database check at 2026-09-29 07:55 UTC found that preimage's
attribution hash matched the current file and that these exact paths and
their ancestors had no live lease. An initial coordinator health preflight
timed out before command submission. A prior four-path lease claim succeeded
under request `8e608d15df1340ff913e80f228caebe9`, followed by content
attribution under `4384d97b21954b519d3e7875edf7972f`. The regression was
subsequently strengthened with a known unordered input. One claim attempt
failed before admission while the coordinator API was unavailable; a read-only
check found no active lease. The next exact four-path claim succeeded under
`870c6ed1756745ad8430c529d3ad34f9`, and current content attribution
completed under `42362e24d46c4c26a103ad8486e9f9d9`. The final record
bytes were reattributed after this status update. Grouped managed Cargo
validation remains pending.

Pinned Rustfmt 1.94.1 passed with edition 2021 and `skip_children=true` for
both Rust files, and the scoped tracked diff check passed after test
strengthening. Independent review of the strengthened traversal and adversarial
test found no correction needed. Four-path whitespace, hash, wiring, and
document-structure checks passed. The regression, Runtime package check,
and Release-scale scan profile have not run.

An iterator-level `read_dir` error still occurs during collection before
sorting, so the first reported iterator error can follow enumeration order.
Sorting one root does not define precedence between separately supplied
`asset_roots`, and it does not settle case-folding or Unicode-equivalent
collisions. Those remain under `ASSETREG-P1-022` and `ASSETREG-P1-023`.
`ASSETREG-P1-021` should only be marked accepted after the focused regression
and package gate pass on the attributed current-source snapshot.
