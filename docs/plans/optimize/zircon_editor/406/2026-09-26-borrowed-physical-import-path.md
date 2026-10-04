---
title: Editor406 Borrowed Physical Import Path
category: zircon_editor
date: 2026-09-26
implementation_status: implemented
validation_status: batched_validation_pending
---

# Editor406 Borrowed Physical Import Path

## Scope and performance target

The UI Asset Editor import traversal uses a physical-path set to expand shared widget/style sources once per resolution. A duplicate logical import previously allocated a new `PathBuf` before the set rejected it. The traversal now probes with `&Path` and owns the path only for the first expansion. For 65,536 attempts against one valid 192-byte Windows path, the deterministic allocation target is **one owned path instead of 65,536**. The fixture has multiple components, each at most 255 characters, and stays below the usual 260-character path limit.

The ignored Windows Release benchmark `EDITOR406_BORROWED_DUPLICATE_PHYSICAL_PATH_BENCH_V1` uses 17 alternating baseline/optimized sample pairs and retains the **pending** optimized p95 target at most 65% of baseline p95. The separate `EDITOR406_PHYSICAL_PATH_UNIQUE_MIXED_BENCH_V1` tests 128 all-unique paths and a 128-event, 50%-unique mixture, each over 128 rounds in 17 alternating pairs; both require optimized p95 at most 110% of baseline. This exposes any extra ordered-set lookup cost on ordinary unique paths without lowering the duplicate target. These helper workloads do not establish product UI-import percentiles. No dynamic pass is claimed while the managed batch is pending.

## Behavior and validation

- Physical-path expansion still returns `true` on first use and `false` for repeated references or fragment aliases; the logical widget/style records continue to materialize.
- `finish_resolution` still clears expansion state so the next resolution can expand the same physical source again, while the generation parse cache remains available.
- A focused behavior regression covers a repeated logical reference in one resolution and its reuse in the next. Existing fragment-alias, parse-cache, and unresolved-reference regressions remain in the same Editor package batch.

| Gate | Scope | State |
|---|---|---|
| Source | `zircon_editor/src/ui/host/asset_editor_sessions/imports/{traversal,tests}.rs` | Rustfmt, scoped diff check, and borrowed-path structural check passed |
| Cargo | one managed Windows `zircon_editor` package check plus all affected import-traversal regressions, coalesced with other Editor repairs | pending coordinator batch |
| Performance | ignored Release duplicate and unique/mixed path benchmarks; representative UI Asset import p50/p95/p99 | pending coordinator batch; all three p95 gates unmeasured |

This is an implemented optimization candidate until the shared regressions and performance gate reach a terminal pass.
