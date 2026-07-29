---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract
  - zircon_runtime/src/ui
implementation_files:
  - docs/plans/performance/01/2026-07-17-editor-visual-assets-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-input-tests-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-text-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-template-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-taffy-tests-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-surface-input-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-surface-frame-tests-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-surface-default-interactions-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-input-ownership-tests-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-event-routing-tests-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-dirty-domain-tests-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-accessibility-widget-tests-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-window-event-tests-static-review.md
  - docs/plans/performance/01/2026-07-18-runtime-ui-window-pump-tests-static-review.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - python tools/check_conventions.py --only docs --json
  - exact15 retired wildcard and existing-directory guard
  - git diff --check -- exact15 Batch49 paths
---

# Frameworks06 G7 Runtime UI Owner Hard Cut Batch 49

Status: focused_g7_green_global_g7_red_review_findings_resolved_commit_pending
Date: 2026-07-29
Session: `frameworks06-g7-runtime-ui-owner-hardcut-batch49-r1-20260729`

## Completed Items

- Adopted 13 complete Runtime UI review records from archived Session
  `20260717-0515-performance-mvp-audit` after an independent full-content
  source review, and retained their current directory owner declarations.
- Replaced three glob-shaped owners in the tracked visual-assets review with
  the corresponding existing directory authorities. Its status is downgraded
  to `static_incremental_review_pending` because the independently measured
  `template_node_images` scope grew from 257 to 419 lines.
- This exact15 handoff contains 17 existing directory authorities across 14
  implementation records; it does not claim review completion for excluded
  documents or unreviewed source increments.
- Excluded workbench renderer, Runtime UI v2 tests, focus navigation, and
  accessibility reviews after independent review found current-source
  coverage drift. They remain outside this commit scope and with their
  original worktree owners.
- Excluded the runtime UI surface-render review because it still owns a
  separate non-wildcard missing path that requires an independent root-cause
  decision.
- Added no wildcard compatibility interpretation, old-path alias, shim,
  generated owner, or duplicate architecture record.

## Validation State

- The first exact19 review was rejected at `C0/I2/M0/Minor0` because it mixed
  complete untracked records with a metadata-only claim. The follow-up full
  content review was rejected at `C0/I2/M1/Minor1` and identified four stale
  coverage records plus the visual-assets incremental drift. This record
  applies those scope and status corrections: the four stale records are
  outside exact15, the 13 adopted records are precisely the records the
  reviewer found current, and visual-assets no longer claims the unreviewed
  162-line increment as complete.
- Exact15 retired-wildcard guard: `0` remaining `/**/*.rs` owners.
- Replacement-directory guard: `17/17` owner directories exist.
- Focused current-source G7: `0` violations across exact15.
- `git diff --check -- <exact15>`: exit `0`; Git reports only the existing
  LF-to-CRLF checkout warning for the tracked visual-assets review.
- Shared current-source G7 remains red: `546` missing-path violations across
  `105` documents while checking `68885` metadata paths. Frameworks06 M1/M2
  are not claimed complete by this bounded metadata migration.
- Coordinator snapshot `1308` captured exact15 at ordinal fingerprint
  `e36ef1ea3a936a1320647d9eff627de71731f63175e7d03362d4f396cd98e1be`;
  a post-reclaim preview remained `15/15` with `would_change=false`.
- Final immutable snapshot and coordinator maintenance commit remain pending.
