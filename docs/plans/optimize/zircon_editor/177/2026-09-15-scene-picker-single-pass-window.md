---
title: Editor177 Scene Picker Single-Pass Window
category: zircon_editor
report_id: Editor177-scene-picker-single-pass-window-2026-09-15
date: 2026-09-15
session_id: root-runtime-editor-async-optimization-20260915
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor177 · Scene Picker 单次扫描窗口

## Scope

`scene_open_query_window` originally counted every matching scene and then
filtered the same source a second time to materialize the requested 12-entry
window. That duplicated query matching for the common Scene Picker open path.
The broader Editor177 search-runtime, provider, cancellation, and result-receipt
work remains outside this focused local hot-path slice.

## Implementation

- Normalize the query once, then inspect each source entry at most once.
- Retain only the requested 12-entry page and the trailing 12-entry page while
  counting matches. The latter preserves the pre-existing final-page fallback
  when the requested offset is beyond the result range.
- Allocate the bounded page buffers lazily, so a no-match query retains the
  former no-result allocation behavior.
- Preserve query matching, order, 12-row page size, normalized offsets, and
  final-page fallback exactly; only the traversal/allocation shape changes.

## Regression and performance contract

The lower module
`zircon_editor/src/ui/retained_host/app/scene_picker_session/single_pass_tests.rs`
compares empty, filtered, missing-query, and out-of-range fallback windows with
the legacy two-pass implementation. It also guards the one-loop/two-bounded-page
source shape and provides the ignored paired marker
`EDITOR787_SCENE_PICKER_SINGLE_PASS_WINDOW_BENCH_V1`. The Python source contract
is `tools/tests/test_editor_scene_picker_single_pass_window_performance_contract.py`.

Current source fingerprints: `scene_picker_session.rs`
`36F9F1308F129F4E1EB91D1D5381A2029C92F373D204348FE7AF0960E61BFB78`, lower
regression `3774F21489B90B3B1A425E9E2B8F4F3FFBC7C6FED37D85C113717F7555F4248C`,
and Python contract `DBA6500D51951414B6E71C964834C5631C6832C097DF79F21B87AC6E46B912E6`.

For every query, the legacy shape performs two matching traversals; the revised
shape performs one traversal and retains at most two 12-entry page buffers. This
is deterministic source/allocation-shape evidence, not a CPU, allocator, RSS,
or product p50/p95/p99 measurement.

## Local receipt

The focused TDD source contract passes `3/3`, and the lower semantic regression
plus ignored Release benchmark are wired and Rustfmt-clean. A randomized
`60,000`-case parity model also passes after the final-page boundary repair.
The refreshed single-process Runtime/Editor source-contract batch loads `552`
modules and passes `1975/1975` tests in `4.790s`, with zero failures, errors,
or skips. This is local source/model evidence only.
The broader non-tooling Runtime/Editor Python regression discovery also passes
`3724/3724` across `915` modules in `326.952s`, with zero failures, errors, or
skips.
The current recent-record Rustfmt batch covers `169` Rust files referenced by
the Runtime/Editor optimize set (`74` Runtime, `84` Editor, and `11` shared
plugin/interface owners) and now passes with zero diffs after a mechanical
import-order repair in this lower-test owner; no production behavior changed.

## Validation boundary

Managed Windows Cargo/Release execution, allocator counts, and Scene Picker
product percentile evidence remain coordinator-owned and pending. No per-task
Cargo run, coordinator retry, or status query is made for this record. Tooling
production work remains deferred for the later Rust migration.
