---
related_code:
  - zircon_runtime/src/animation/sequence.rs
  - zircon_runtime/src/animation/sequence/apply.rs
  - zircon_runtime/src/animation/sequence/target.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
implementation_files:
  - docs/editor-and-tooling/runtime-editor-boundary-cleanup.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/editor-and-tooling/runtime-editor-boundary-cleanup.md docs/plans/zircon_runtime/frameworks/06/2026-07-29-g7-animation-runtime-system-owner-doc-hardcut-batch42.md
---

# Frameworks06 G7 Animation Runtime-System Owner Hard Cut Batch 42

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-29
Session: `frameworks06-g7-animation-runtime-system-owner-doc-hardcut-batch42-r1-20260729`

## Completed Items

- Replaced the retired Animation Plugin `sequence.rs` machine owner with the
  current Runtime `animation/sequence.rs` owner.
- Removed the duplicate retired Plugin `sequence/apply.rs` and
  `sequence/target.rs` paths while retaining the existing Runtime apply and
  target owners.
- Replaced the deleted Runtime `animation/scene_hook.rs` path with the current
  Animation Plugin `runtime_system.rs`, which registers the PostUpdate
  animation evaluation system and dispatches the evaluation pipeline.
- Added no compatibility path, forwarding owner, alias, shim, or duplicate
  architecture record.

## Validation State

- Fresh G7 reports zero violations for the updated architecture document and
  this Batch42 record. The four retired owner paths have zero machine-owner
  hits, while the Runtime sequence facade/apply/target and Animation Plugin
  runtime-system paths all exist.
- Exact-scope `git diff --check` passes. The shared current-source G7 baseline
  remains red at 696 violations across 171 documents and 68,736 checked paths.
- Independent exact2 review found Critical/Important/Moderate/Minor
  `0/0/0/0` with zero input drift. This batch does not claim Frameworks06
  M1, M2, or plan completion.
