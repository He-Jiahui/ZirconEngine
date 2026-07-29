---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract
implementation_files:
  - docs/plans/performance/01/2026-07-17-editor-template-shell-panels-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-sliders-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-status-controls-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-style-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-surface-hit-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-surface-icon-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-table-rows-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-tooltips-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-tree-rows-static-review.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - python tools/check_conventions.py --only docs --json
  - exact10 retired wildcard and existing-directory guard
  - git diff --check -- exact10 Batch48 paths
---

# Frameworks06 G7 Editor Template Surfaces Owner Hard Cut Batch 48

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-29
Session: `frameworks06-g7-editor-template-surfaces-owner-hardcut-batch48-r1-20260729`

## Completed Items

- Replaced 31 glob-shaped Rust owners across nine retained-host template and
  surface review records with the corresponding 31 existing directory
  authorities.
- Preserved every review record's findings, status, concrete file owners, and
  prose; this metadata hard cut does not claim a new source review.
- Added no wildcard compatibility interpretation, old-path alias, shim,
  generated owner, or duplicate architecture record.

## Validation State

- Fresh G7 reports zero violations for all exact10 documents. Retired wildcard
  owners have zero hits, all 31 replacement directories exist, and exact-scope
  `git diff --check` passes.
- The shared current-source G7 baseline remains red at 566 violations across
  122 documents and 68,869 checked paths. Snapshot1305 independent review
  passed with Critical/Important/Moderate/Minor = 0/0/0/0 and exact10 ordinal
  fingerprint `426793a9ecb95fe8f5bba1a83622dc70e85687ac7f8f1be8c89f6ffa3d4af0fd`;
  this batch does not claim Frameworks06 M1, M2, the global G7 gate, or plan
  completion.
