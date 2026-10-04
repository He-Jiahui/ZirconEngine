---
title: Editor Animation Graph Label Single Buffer
category: zircon_editor
report_id: Editor890-animation-graph-label-single-buffer-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor890 Animation Graph Label Single Buffer

## Finding

Animation Editor Blend and Mask node labels first joined their IDs into a
temporary `String` and then formatted that temporary into the retained label.
Dense graph projections therefore allocated an intermediate join output for
every displayed node in addition to the required final label.

## Optimization

- Build every graph-node label through one capacity-bounded output `String`.
- Append Blend inputs and Mask target IDs with position-based delimiters instead
  of materializing a joined child string.
- Derive the capacity from the variant's borrowed fields and canonical locator
  upper bound, avoiding growth while retaining a single output allocation.
- Preserve Clip, empty/non-empty Blend, Additive, empty/non-empty Mask, Output,
  authored ID order, empty IDs, punctuation, and locator display exactly.

## TDD and deterministic evidence

The combined Editor890/Runtime871 source-model batch was observed RED at `2/10`
and GREEN at `10/10`. A tightened capacity contract was then observed RED at
`4/5` and GREEN at `5/5`. Lower regressions compare every node variant,
including empty ID lists and empty authored IDs, against the retired
join/format implementation.

Across 4,096 dense Blend labels with 64 input IDs, the deterministic model
changes temporary join outputs from `4096` to `0`; the required capacity-bounded
label remains. Ignored marker
`EDITOR890_ANIMATION_GRAPH_LABEL_SINGLE_BUFFER_BENCH_V1` emits 101 alternating
p50/p95/p99 sample pairs and requires single-buffer p95 to remain within 10%
of join/format.

## Local validation boundary

- Exact-file Rustfmt, Python bytecode compilation, and scoped
  `git diff --check` pass.
- Editor890 and Runtime871 contracts pass `10/10`; the combined batch with
  Animation Editor ZUI, curve projection, and timeline projection contracts
  passes `18/18`.
- Editor890 received no per-task Cargo run and was submitted with Runtime871 in
  asynchronous v17 (PID `10460`) at `2026-09-21T21:47:37.0745928+08:00`.
- No v13-v17 receipt was read or monitored after submission.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/animation_editor/session/graph.rs` | `40F7684BDAF22CF7A2D34D4A5008D2D8DE658FAEBF7CBF82D68485CD22B8668F` |
| `zircon_editor/src/ui/animation_editor/session/graph/single_buffer_tests.rs` | `A355BA5821E9592D2283DE87B1AE769F6D6BFB52D6F1B61363EAC55DA6C1C61A` |
| `tools/tests/test_editor890_animation_graph_label_single_buffer_performance_contract.py` | `E0B5CAE969CBE7C1490E1ED2D4876B6A2139A7DB0BBC11CE59E213F91D95034F` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus real
Animation Editor graph-projection p50/p95/p99 evidence. The deterministic
allocation model is not product acceptance.
