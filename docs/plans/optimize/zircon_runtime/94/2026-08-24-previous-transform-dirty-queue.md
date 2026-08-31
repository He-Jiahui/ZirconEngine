---
title: Runtime94 Previous Transform Dirty Queue Optimization
category: zircon_runtime
report_id: Runtime94-previous-transform-dirty-queue-2026-08-24
date: 2026-08-24
session_id: root-runtime94-prev-transform-dirty-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime94 Previous Transform Dirty Queue Optimization

## Scope

This slice removes the successful-frame full live-entry scan used to roll GPU Scene previous
transforms. It advances Runtime94's dirty-proportional history work without changing instance span
allocation, GPU upload submission, visibility, HZB, material payloads, or the wider persistent
RenderScene architecture.

## Implementation

`GpuScene` now records stable instance keys that need a previous-transform roll. New registrations
enter the set so their first successful frame publishes valid history. Later writes enter it only
when `world_from_local` changes, while unregister removes the key before its span can be reused.

After a successful submission, the roll path takes that generation's pending set and visits only
the corresponding current entries. The existing instance shadow remains authoritative, unchanged
spans still avoid uploads, dirty spans continue through `GpuSceneUpdateQueue`, and the report's live
instance total comes from the maintained O(1) GPU Scene stats instead of another full scan.

Regression coverage checks dirty-only visitation, first-frame history validity, unchanged
steady-state zero visits, and the registration/write/unregister source contract.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 100,000 live entries with 32 transform-dirty entries | 100,000 entry visits | 32 dirty-key probes | 99.9680% lookup-work reduction |
| Successful-frame history roll | O(live entries + live instances) | O(transform-dirty entries + dirty instances) average | generation-local pending key set |
| Unchanged steady frame | scans every live entry | zero entry visits | no pending-key materialization |
| 100,000 live / 32 dirty release p95 | dynamic evidence pending | <= 5 ms and <= 50% of legacy p95 | coordinator release gate |

The ignored Windows-native release evidence alternates 11 legacy/optimized sample pairs and prints
`RUNTIME94_PREV_TRANSFORM_BENCH_V1` with exact p95 nanoseconds, the target, live/dirty counts, and
deterministic visit counts. Dynamic elapsed time is accepted only from coordinator terminal
evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, and Runtime94 dirty-roll source contracts:
  passed.
- Previous-transform regressions remain pending a copy-complete GPU Scene validation union. The
  current `gpu_scene.rs` owner also carries upload-transaction and skinned-palette migration work,
  so this slice is intentionally excluded from the self-contained Runtime94 index batch rather
  than validating `prev_transform.rs` against an incompatible HEAD owner.
- No local Cargo lane is launched, and no coordinator compilation is monitored in real time.
- Terminal marker values, commit integration, optimization-record finalization, and automatic WeCom
  delivery remain pending.

## Remaining Parent-plan Work

Runtime94 still owns authoritative bounds, persistent primitive and instance lifecycle, dense
handles, visibility-first preparation, multi-view culling, GPU compaction, history publication after
GPU completion, and product-scale qualification. Those milestones remain separate work and are not
claimed complete by this dirty-roll optimization.
