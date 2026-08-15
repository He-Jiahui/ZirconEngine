Plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
Milestone: M1
Status: completed
Acceptance: pending_independent_review
Files: ["docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md", "docs/plans/zircon_editor/editor_ui/12/2026-08-11-m1-reference-architecture-current-source-manifest.md"]
Depends-On-Milestones: []
Depends-On-Failures: []
---

# Editor UI12 M1 Reference Architecture Current-Source Manifest

## Scope delivered

This manifest binds the Unreal-primary, MagicaVoxel-secondary `.zui` design
convergence plan to M1. It delivers the reference evidence, editor/runtime
boundary, baseline inventory, milestone ordering, and the per-milestone
coordinator commit and WeCom policy. It contains no production implementation
claim. `Status: completed` means only that this source output record is
structurally complete for coordinator pre-bind. `Acceptance` remains pending
until the immutable source passes managed validation and an independent review
with zero Critical and zero Important findings.

## Fresh testing evidence

The source-bound coordinator validation run
`3a15aecc57d24f3f9ed1fbbf73645cfd` completed with exit code 0 on 2026-08-11.
The `coordinator-actions` template ran 36 tests covering the action catalog,
authorization, fingerprints, execution, and concurrency; all passed. The plan
path audit, repository-local reference-path resolution, plan-output audit, and
scoped `git diff --check` also passed before the immutable validation binding.
That historical run does not accept later source edits; the coordinator record
for the final immutable source is authoritative and must be freshly validated.

## Review

The independent reviewer must confirm that the plan keeps editor authoring
state in `zircon_editor::ui`, uses Unreal as the architecture authority, treats
MagicaVoxel as a visual and workflow reference rather than a source-copy
target, and defines testable style, trigger, popup/focus, and layout contracts.
The preceding review recorded 0 Critical and 6 Important findings. This source
revision addresses them by defining the binding/action survivor, runtime popup
ownership hard cut, logical-pixel layout oracle, composite primary-plus-overlay
state output, permanent UI Asset Editor scan-only exclusion, and the distinction
between source-record completion and milestone acceptance.
The coordinator review record is authoritative for the acceptance verdict.

## Milestone status

| Milestone | Testing scope | Status | Date | Evidence |
|---|---|---|---|---|
| M1 | M1-T reference architecture testing | 通过 | 2026-08-11 | Historical managed validation and scoped static audits passed; corrected immutable source requires fresh managed validation and a new independent coordinator review before commit. |
