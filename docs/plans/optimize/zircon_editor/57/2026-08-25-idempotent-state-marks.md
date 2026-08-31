---
title: Editor57 Idempotent State Marks
category: zircon_editor
report_id: Editor57-idempotent-state-marks-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Idempotent State Marks

## Scope

This slice removes redundant shared-string reconstruction from repeated Asset Browser toggle and
utility-tab state synchronization. It preserves selected and focused values, active and idle surface
variants, active and idle text tones, first matching control behavior, and panel selection helpers.
It does not change Editor57 interaction, selection, navigation, mutation, or retained UI authority.

## Implementation

The retired state markers assigned `surface_variant` and `text_tone` on every call, even when the
target node already held the requested values. The current retained-host `SharedString` is a `String`
alias, so a repeated active toggle rebuilt two owned strings. The optimized path compares the current
borrowed text first and assigns only when the value changes. Initial activation and real state
transitions retain the same output, while steady-state synchronization performs no shared-string
write or allocation.

The regression covers active and idle toggle values, utility-tab values, and pointer stability across
an unchanged repeated update. A source contract requires all four production field updates to route
through the conditional helper.

## Performance Contract

| Evidence per repeated unchanged toggle mark | Retired path | Optimized gate |
| --- | ---: | ---: |
| Shared-string writes and reconstructions | 2 | 0 |
| Selected/focused scalar writes | 2 | 2 |
| Alternating release benchmark | 11 samples x 16,384 marks | optimized P95 <= 45% of retired P95 |

The benchmark emits `EDITOR57_IDEMPOTENT_STATE_MARKS_BENCH_V1` with both P95 timings, reduction basis
points, sample/iteration counts, and repeated shared-string writes.

## Validation

The TDD source probe first observed no conditional helper and both unconditional field assignments,
then observed five helper references and no retired assignments. Rust 1.94.1 formatting and scoped
static checks passed before batching. One managed Editor batch covers this slice together with the
single-pass summary layout, including behavior regressions, source contracts, pointer stability, and
both ignored release benchmarks. Dynamic P95 evidence, integration SHA, automatic commit, and
automatic WeCom performance delivery remain coordinator-owned and pending.
