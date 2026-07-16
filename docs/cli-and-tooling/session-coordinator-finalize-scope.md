---
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_git_finalize_scope_names.py
implementation_files:
  - tools/session_coordinator/git_finalize.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
tests:
  - python -m unittest tools.session_coordinator.tests.test_git_finalize_scope_names
  - python -m unittest tools.session_coordinator.tests.test_git_finalize.GitFinalizeTests.test_maintenance_finalize_preserves_foreign_staged_index_on_degraded_baseline
  - python -m unittest tools.session_coordinator.tests.test_git_finalize.GitFinalizeTests.test_cleanup_shared_index_restores_head_index_without_changing_worktree
doc_type: module-detail
---

# Session Coordinator Finalize Scope Enumeration

## Purpose

The coordinator compares an immutable finalize manifest with the exact staged path set before creating a commit. Delete/add pairs must stay visible as two independent paths even when their content is similar enough for Git to classify them as a rename.

## Contract

`GitFinalizeService._staged_scope_paths()` is the only staged-name enumeration owner used by explicit finalize scope checks, shared-index cleanup, and foreign-index rejection. It invokes `git diff --cached --name-only --no-renames` so a manifest containing `failure-old.md` deletion plus `fixed-new.md` addition observes both source and target paths.

This changes only scope enumeration. Staging still uses path-bounded `git add -A`; attribution, live lease, worktree/index identity, secret scanning, Git mutex, immutable manifest, and foreign staged-path checks remain mandatory.

## Regression Evidence

`test_staged_scope_keeps_rename_source_and_target_paths` creates a temporary repository with an identical-content delete/add pair, stages it through the finalizer, and requires both paths. The maintenance-finalize foreign-index preservation and shared-index cleanup regressions remain green alongside it.
