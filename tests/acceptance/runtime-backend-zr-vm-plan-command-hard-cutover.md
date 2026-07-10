---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_06.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_07.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
output_records:
  - docs/plans/zircon_runtime/runtime/06/2026-07-09-plugin-surface-and-lifecycle-output-records.md
  - docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md
---

# Runtime ZrVM Backend Plan-Command Hard Cutover

Date: 2026-07-10

Frameworks 03 renamed the runtime backend feature to `backend-zr-vm` without an alias. Runtime 01 dependency policy, Runtime 06 lifecycle validation, and Runtime 07 authoritative FPS validation now use that current feature in their executable instructions.

Historical output rows retain the feature spelling that was actually used when those commands ran; they are evidence, not current instructions.

Acceptance requires:

- the Runtime 06 plan-status Cargo-gate guard to find its full `backend-zr-vm` Vampire command;
- the Runtime 07 plan-status Cargo-gate guard to find its full `backend-zr-vm` FPS command;
- the parent plans to remain `in_progress`, because this documentation cutover does not execute the real-backend or performance gates.

Verification: the standalone plan-status harness was recompiled against the current plans and passed 48/48, including both Runtime 06 and Runtime 07 Cargo-gate guards.

The Runtime 06 lifecycle audit was also updated to its folder-backed guard children and the current Cargo-gate child owner. Python regression passes 3/3, and the direct lifecycle report has no missing source, document, or command anchors and no risks.

The Runtime 07 performance audit now reads concrete owner-budget/history evidence from the numbered Runtime 07 archive and recognizes the current FPS backend command. Its dedicated regression passes 1/1; direct evidence reports source files 46, test owners 91, no missing document or Cargo anchors, mirror guard present, and no risks.
