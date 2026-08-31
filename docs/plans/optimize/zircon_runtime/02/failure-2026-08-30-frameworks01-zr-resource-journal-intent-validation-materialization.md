---
handoff_kind: failure
status: open
created_at: 2026-08-30
summary_slug: frameworks01-zr-resource-journal-intent-validation-materialization
origin_plan: docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/optimize/zircon_runtime/02
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/tests/migration/project_commandlet/crash_windows.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/journal/intent.rs
tests:
  - validation ticket 78ef39a572e1422e83b9c048832034e8
  - validation copy job 7b1be72ab9404e6aa1f16c4fe5450e4b
---

# Frameworks01: `zr_resource` journal intent is outside validation materialization

## Source executor

- Origin plan: `docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md`
- Origin slice: batched Runtime/Editor Release validation for optimization batches 501-514
- Fixing plan: `docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- Handoff reason: the missing source is the Frameworks01 physical owner path for the Runtime25
  resource hard cut, below Runtime02 optimization ownership.

## Failure evidence

After Runtime02 corrected `crash_windows.rs` from the removed monolithic journal path to the
canonical `zircon_runtime/crates/zr_resource/src/io/transaction/journal/intent.rs`, ticket
`78ef39a572e1422e83b9c048832034e8` reached closure planning with the new path. Job
`7b1be72ab9404e6aa1f16c4fe5450e4b` then failed
`validation_copy_compile_time_resource_missing` because the canonical intent file is still
untracked and owned by Session
`frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825`.

Current intent SHA-256 is
`7781ec21ea073e8290e7e885b6637a614b4bcbc2b30f6222c9f74301b9ade8a5`. Runtime02 did not edit,
attribute, transfer, or include that foreign source in a commit candidate.

## Lowest shared-layer root cause

The hard-cut consumer now names the correct crate-owned implementation, but the implementation has
not entered a validation-copy materializable source closure. Repeating Runtime02 tickets cannot pass
closure planning while the file exists only as an untracked foreign-owner change.

## Architecture acceptance

- Frameworks01 integrates the exact `zr_resource` journal intent source and its required module
  closure, or performs a legal exact-path ownership transfer with current hashes.
- The canonical path remains the only implementation; no monolithic Runtime compatibility file is
  restored.
- A managed Runtime/Editor validation copy advances beyond closure planning and compiles the
  batches against the same intent implementation.
- Runtime02 then reruns one aggregate validation rather than one ticket per optimization batch.

## Forbidden workarounds

- Do not recreate `zircon_runtime/src/core/resource/io/transaction/journal/intent.rs`.
- Do not copy the intent implementation into Runtime02-owned tests or add an alternate include
  fallback.
- Do not use maintenance ownership override or claim Frameworks01 source without owner rotation.

## Return contract

Return the integrated/transfer request ID, exact intent hash, and managed validation evidence that
the source is materializable. Runtime02 will resume the aggregate 501-514 validation without
polling the fixing Session.
