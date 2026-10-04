---
title: Editor01 visual candidate capacity
category: zircon_editor
report_id: Editor829-visual-candidate-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor829 · visual candidate capacity

## Scope

Retained-host visual asset resolution constructs finite candidate lists for
packaged image variants, preview artifacts, and packaged icon aliases. The
previous collectors started at zero capacity, so common nonempty paths paid
geometric growth before the loader could probe the candidates. Empty inputs
and development-module expansion must keep their existing behavior.

## Implementation

- Reserve the four-entry packaged image bound only for nonempty image sources.
- Reserve the five-entry preview bound for relative artifact sources and keep
  the absolute-source fast path at one entry.
- Reserve the six-entry packaged icon bound only for nonempty icon names;
  development Material-UI module candidates continue to append and grow beyond
  that packaged bound when enabled.
- Preserve candidate order, adjacent-last-hit deduplication, asset-root
  containment, absolute-source identity, and empty-path zero capacity.
- Add a lower capacity/order regression and the ignored
  `EDITOR829_VISUAL_CANDIDATE_CAPACITY_BENCH_V1` Release marker.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the three
zero-capacity collectors and GREEN after the finite nonempty reservations were
added (`4/4`). For the representative 4/5/6 candidate variant counts, the
deterministic model removes five geometric growth events across the three
zero-capacity collectors (`5→0`);
empty image input remains zero-capacity. This is allocation-shape evidence
only, not allocator, CPU, RSS, or product percentile evidence.

## Local evidence

- `tools/tests/test_editor_visual_candidate_capacity_performance_contract.py`:
  `4/4`.
- Lower Rust module
  `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/candidates/query/capacity_tests.rs`
  covers packaged image cardinality, preview/icon bounds, empty input, and the
  ignored Release marker.
- Exact-file `rustfmt --edition 2021 --check` passes for the production and
  lower Rust files; Python compilation passes for the source contract.
- The current one-process non-tooling Runtime/Editor contract batch loads
  `627` modules and passes `2239/2239` tests in `5.505s` with zero failures,
  errors, or skips; the nine-slice focused loader passes `33/33` in `0.017s`.
- Managed Windows Cargo/Release compilation, lower Rust execution, allocator
  evidence, and visual-resource product p50/p95/p99 remain pending behind the
  external dirty-worktree admission gate. Tooling production remains deferred
  for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/candidates/query.rs` | `2A01FE36B0194445465F3B2E7AB80EF71C23C89F7C48DA357AE273D317036797` |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/candidates/query/capacity_tests.rs` | `E632458BE341AF2DAB02F7668751BA45752CFC1AE53DD51B82EEDEB9BD24429E` |
| `tools/tests/test_editor_visual_candidate_capacity_performance_contract.py` | `FCBAAACFB0FD7CFB88642657B1D0C68AA93B825E4A9F3EEE3FCD8835E88A7B0D` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release batch compiles the current Runtime/Editor
tree, executes the lower regression and ignored marker, and supplies allocator
plus visual-resource product p50/p95/p99 evidence. Local source/model evidence
does not claim final performance acceptance.
