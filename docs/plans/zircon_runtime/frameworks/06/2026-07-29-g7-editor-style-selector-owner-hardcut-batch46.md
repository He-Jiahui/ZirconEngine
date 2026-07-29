---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector
implementation_files:
  - docs/plans/performance/01/2026-07-17-editor-style-selector-static-review.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - python tools/check_conventions.py --only docs --json
  - current style_selector Rust file and line inventory
  - git diff --check -- exact2 Batch46 paths
---

# Frameworks06 G7 Editor Style Selector Owner Hard Cut Batch 46

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-29
Session: `frameworks06-g7-editor-style-selector-owner-hardcut-batch46-r1-20260729`

## Completed Items

- Replaced the deleted `style_selector.rs` owner and glob-shaped subtree owner
  with the single existing `style_selector` directory authority.
- Reconciled the scope prose to distinguish the recorded 157-file/7,825-line
  2026-07-17 review baseline from the current 157-file/8,629-physical-line
  inventory, and set the status to `partial_static_complete_dynamic_pending`
  until the intervening content receives a fresh file-by-file review.
- Added no old-path alias, wildcard compatibility interpretation, shim,
  generated owner, or duplicate architecture record.

## Validation State

- Fresh G7 reports zero violations for both exact2 documents. The current
  `style_selector` directory contains exactly 157 Rust files and 8,629 physical
  lines; retired machine owners have zero hits, and the prose does not claim
  that the current delta has already received a file-by-file review.
- Exact-scope `git diff --check` passes. The shared current-source G7 baseline
  remains red at 627 violations across 141 documents and 68,842 checked paths.
- Corrected snapshot1300 independent review passed with
  Critical/Important/Moderate/Minor = 0/0/0/0; rejected snapshot1299 is not
  acceptance evidence. This batch does not claim Frameworks06 M1, M2, the
  global G7 gate, or plan completion.
