---
title: Editor51 Scene Picker Sort Projection Optimization
category: zircon_editor
report_id: Editor51-scene-picker-sort-projection-2026-08-24
date: 2026-08-24
session_id: root-editor51-scene-picker-sort-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor51 Scene Picker Sort Projection Optimization

## Scope

This slice removes one full URI clone per valid scene while building the retained Open Scene picker.
It advances Editor51's large-project open-path allocation work without changing catalog authority,
picker tickets, query paging, scene opening, document transitions, or startup ownership.

## Implementation

The catalog projection now sorts `(lowercase key, borrowed original URI)` pairs. The lowercase key
preserves the former case-insensitive primary ordering, the borrowed URI preserves the exact-string
secondary ordering and deduplication behavior, and the original URI is copied only once when the
persistent `ScenePickerEntry` row is created.

The former `sort_by_cached_key` path held the already-cloned URI plus a lowercase key and a second
original-URI key clone. The new projection retains only the lowercase key and a borrowed pointer
during sorting. Exact duplicate locators still collapse after ordering, case-distinct locators stay
distinct, and command IDs are still assigned after the stable display order is finalized.

Regression coverage checks case folding, original-string tie breaking, exact deduplication, and the
borrowed-key source contract.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Scene URI string bytes across sort plus persistent rows | 3 x locator bytes | 2 x locator bytes | 33.3333% byte reduction |
| Temporary sort-key URI bytes | 2 x locator bytes | 1 x locator bytes | 50.0000% temporary-key reduction |
| URI copies per unique valid scene | 3 string copies | 2 string copies | original URI borrowed during sort |
| 20,000 representative scene URIs release p95 | dynamic evidence pending | <= 100 ms and <= 90% of legacy p95 | coordinator release gate |

The ignored Windows-native release evidence alternates 11 legacy/optimized sample pairs and prints
`EDITOR51_SCENE_PICKER_SORT_BENCH_V1` with exact p95 nanoseconds, the target, URI and byte counts,
and deterministic string-byte reduction. Dynamic elapsed time is accepted only from coordinator
terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, and Editor51 borrowed sort-projection source
  contracts: passed.
- Ordering/dedup regression and ignored release performance evidence: pending one
  coordinator-managed Editor12+Editor51 batch using the `optimization_wave_20260824q_` filter.
- No local Cargo lane is launched, and no coordinator compilation is monitored in real time.
- Terminal marker values, commit integration, optimization-record finalization, and automatic WeCom
  delivery remain pending.

## Remaining Parent-plan Work

Editor51 still owns authoritative project startup/open/create sessions, Hub handshake, focus and
activation, recent-project reconciliation, dirty transition integration, recovery, and large-project
product qualification. Those milestones remain separate work and are not claimed complete by this
sort-projection optimization.
