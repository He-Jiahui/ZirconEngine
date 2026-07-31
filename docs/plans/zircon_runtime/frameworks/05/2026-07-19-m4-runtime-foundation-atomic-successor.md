---
related_code:
  - zircon_runtime/src/core/framework/foundation
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/core/framework/platform
  - zircon_runtime/src/core/framework/scene
  - zircon_runtime/src/core/framework/ui
  - zircon_runtime/src/core/runtime/events
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/foundation
  - zircon_runtime/src/input/runtime/action_evaluator.rs
  - zircon_runtime/src/input/runtime/action_evaluator/binding_index.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
doc_type: milestone-detail
---

# Frameworks05 M4 Runtime Foundation Atomic Successor

Planning plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
Planning milestone: M4 atomic successor
Planning status: waiting_owner_handoff_and_fresh_manifest

Date: 2026-07-19

This record is not yet a commit manifest. Canonical `Plan:`, `Milestone:`, `Status:`, and `Files:`
fields are intentionally absent until every predecessor owner has handed off its current bytes and
the coordinator has produced one deduplicated current-HEAD fingerprint. This prevents milestone
prepare from binding a false one-file M4 manifest while planning is incomplete.

## Scope Delivered

The approved split was re-audited against clean HEAD `9cbc07ca2316f752b05dbef95ade9d70e893afeb`.
The old Frameworks05 exact40, Scene exact27, Runtime02 exact29, Runtime07 exact29, and Runtime12
exact8 manifests are not independently compilable or guard-complete. Their current whole-file
changes form one atomic contract migration:

- Frameworks05 owns the folder-backed Foundation traits and neutral Input, Platform, UI, and Scene
  module identities.
- Runtime02 owns the Foundation config/persistence implementations used by those traits.
- Runtime07 owns the typed event subscription, delivery policy, diagnostics, and Foundation event
  manager implementations.
- Runtime12 owns the one-pass action evaluator and the guards that require both the neutral Input
  identity and `evaluate_binding_axes`.

The current successor boundary excludes Runtime02 asset/meta-IO paths and Render13
`asset/project/manager/scan_and_import.rs`; those changes are not required by the Foundation
config/event contract closure. It also excludes unrelated Performance01 core-runtime work.

## Fresh Testing Evidence

Existing managed jobs remain directional evidence only until the successor manifest is frozen:

- Runtime12 input behavior job `d064840b0a8f40dcb405bab74b493ba1` passed 39/39.
- Runtime12 plan-status job `586f1f84cf814180a1bc71c48a713a90` passed exactly 1/1.
- Runtime12 canonical check job `f6841642e70c4a43b8674c92f9f18461` released with exit 0.
- Runtime12 action guard job `c5d6303ce4334f3995b2b5073af7569b` passed exactly 1/1.

None of those jobs proves a clean-HEAD atomic commit because their source manifests included dirty
predecessor bytes. After owner handoff, the successor requires a new source-bound canonical check,
the focused Frameworks05/Runtime02/Runtime07/Runtime12 guards, and the applicable behavior tests.

## Review

Current clean-HEAD dependency review rejects the old direct commits:

- Frameworks05 exact40 review: Critical 2 / Important 1 / Minor 0.
- Runtime12 exact8 review: Critical 0 / Important 2 / Minor 1.
- Atomic-closure audit: the shared `core/runtime/handle/events.rs`, `foundation/tests.rs`, module
  descriptor consumers, and Runtime12 input descriptor cannot be assigned to only one predecessor
  manifest without unresolved imports, trait-signature errors, or a current guard failure.

The successor is therefore `waiting_owner_handoff_and_fresh_manifest`, not accepted. Final review
must run after the exact manifest, current hashes, and managed validation evidence are all frozen.

## Required Owner Handoffs

1. Performance01 must explicitly release or narrow attribution for the exact Runtime07 event paths
   under `zircon_runtime/src/core/runtime`; directory-level write scope is not consent.
2. Frameworks05 r8, the superseded Scene identity owner, Runtime02 r4, Runtime07 r3, and Runtime12
   r10 must hand their current source hashes to this successor.
3. The coordinator must deduplicate the union, reject foreign drift, and record the final `Files:`
   array and fingerprint here before any milestone prepare or Cargo reservation.
4. No compatibility shim, cfg gate, guard deletion, threshold weakening, raw Git staging, or
   absorption of Render13/Performance01 foreign paths is authorized.
