---
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_git_finalize_scope_names.py
  - tools/session_coordinator/tests/test_git_finalize_tracked_ignored.py
implementation_files:
  - tools/session_coordinator/git_finalize.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-15-tracked-ignored-codex-milestone-add.md
tests:
  - python -m unittest tools.session_coordinator.tests.test_git_finalize_scope_names
  - python -m unittest tools.session_coordinator.tests.test_git_finalize_tracked_ignored
  - python -m unittest tools.session_coordinator.tests.test_git_finalize.GitFinalizeTests.test_maintenance_finalize_preserves_foreign_staged_index_on_degraded_baseline
  - python -m unittest tools.session_coordinator.tests.test_git_finalize.GitFinalizeTests.test_cleanup_shared_index_restores_head_index_without_changing_worktree
doc_type: module-detail
---

# Session Coordinator Finalize Scope and Managed Ignored Paths

## Purpose

The coordinator compares an immutable finalize manifest with the exact staged path set before creating a commit. Delete/add pairs must stay visible as two independent paths even when their content is similar enough for Git to classify them as a rename. Repository-owned Codex skills and hooks must also remain committable when a parent ignore rule matches an already tracked file.

## Contract

`GitFinalizeService._staged_scope_paths()` is the only staged-name enumeration owner used by explicit finalize scope checks, shared-index cleanup, and foreign-index rejection. It invokes `git diff --cached --name-only --no-renames` so a manifest containing `failure-old.md` deletion plus `fixed-new.md` addition observes both source and target paths.

This changes only scope enumeration. Staging still uses path-bounded `git add -A`; attribution, live lease, worktree/index identity, secret scanning, Git mutex, immutable manifest, and foreign staged-path checks remain mandatory.

## Managed Ignored-Path Contract

Both explicit finalize and milestone commit classify the complete approved manifest with `git check-ignore --no-index`; classification therefore covers tracked and untracked paths equally. Ordinary paths use bounded `git add -A`. Ignored paths use `git add -A -f` only when they are `.codex/skills/**`, `.codex/hooks/**`, or `.codex/hooks.json`. Any other ignored path is rejected with a typed finalize or milestone error before staging.

The shared index preservation contract is unchanged: maintenance finalize builds its commit tree from the accepted baseline, commits only the approved paths, restores the prior index bytes, and leaves foreign staged work intact.

If a bounded Git command fails, `finalize_git_command_failed` retains the exit code, redacted and length-bounded stderr, and the exact path chunk that followed `--`. Operators can therefore identify the failing manifest slice without exposing file contents or widening the next retry.

## Regression Evidence

`test_staged_scope_keeps_rename_source_and_target_paths` creates a temporary repository with an identical-content delete/add pair, stages it through the finalizer, and requires both paths. The maintenance-finalize foreign-index preservation and shared-index cleanup regressions remain green alongside it.

`test_git_finalize_tracked_ignored` creates a tracked skill beneath an ignored `.codex` parent and proves four layers: the ignore scan reports the tracked path, Git add failures retain the exact path chunk, maintenance finalize force-adds only that approved skill while retaining a foreign staged file, and milestone commit combines the owned tracked skill with an ordinary untracked document while still retaining the foreign staged file.
