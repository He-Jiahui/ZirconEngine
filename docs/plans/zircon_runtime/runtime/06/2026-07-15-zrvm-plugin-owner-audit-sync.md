# Runtime 06 ZrVM plugin owner audit sync

Plan: docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
Milestone: M1
Status: completed
Files: [".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py","docs/plans/zircon_runtime/runtime/06/2026-07-15-zrvm-plugin-owner-audit-sync.md"]
Date: 2026-07-15

## Scope delivered

- Updated the Runtime06 lifecycle inventory to the hard-cut ZrVM owner at `zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs`.
- Matched the four lifecycle anchors to the current guarded calls for `activate`, `deactivate`, `saveState`, and `restoreState`.
- Advanced the expected Runtime06 `last_refined` mirror to the already recorded 2026-07-14 plan state.
- Kept the old Runtime-side backend path retired; no re-export, compatibility module, or shim was added.
- The prerequisite Runtime06 M1-M3 workflow topology was submitted separately through the protected maintenance channel as `b58d799c`.

## Fresh testing evidence

- Direct `plugin_surface_lifecycle_boundary_audit`: `missing_source_files = []`, `missing_doc_files = []`, `missing_source_anchors = []`, `missing_doc_anchors = []`, `risks = []`.
- `python -m py_compile` passed for the changed audit owner.
- `git diff --check` passed for the exact milestone files.

## Review

- Critical: 0.
- Important: 0.
- The reviewer confirmed the new owner exists, the retired Runtime owner does not, and all five exact mirror conditions pass.

## Status and completed items

| Item | Status | Evidence |
|---|---|---|
| ZrVM lifecycle source inventory | completed | Plugin crate owner is the only audited real-backend path. |
| Guarded lifecycle anchors | completed | Four current calls match the audit exactly. |
| Runtime06 plan-state mirror | completed | Expected and actual `last_refined` are both 2026-07-14. |
| Audit and independent review | completed | `risks = []`; 0 Critical / 0 Important. |
