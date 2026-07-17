<!-- related-code: tools/session_coordinator/baselines.py; tools/session_coordinator/tests/test_baselines.py -->
<!-- related-plans: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md -->

# Shared-main baseline integrity

The coordinator baseline is a content-hash snapshot of tracked and accepted
untracked workspace files. It is a diagnostic control-plane boundary: it never
cleans, stages, resets, or accepts live worktree bytes on behalf of a Session.

## Head advancement

`BaselineService.accept_commit` advances a baseline to the complete Git
manifest for the supplied commit, not only the managed Session's commit
manifest. A shared-main commit can contain paths outside the current Session
scope; leaving those paths at the prior epoch would misclassify already
committed bytes as unattributed workspace drift.

For an ordinary new HEAD, this update is incremental. The coordinator keeps
unchanged tracked hashes from the prior complete baseline, applies the exact
old-to-new Git tree delta, and hashes only added or modified commit blobs with
Git's worktree filters. Deleted paths are removed. This keeps the new baseline
complete without holding the shared Git mutex for a full `git archive` after
every small managed commit. Accepted untracked baseline entries are preserved.

The coordinator intentionally falls back to a full pinned-commit archive when
the prior baseline is incomplete, when it repairs a stale manifest for the same
HEAD, or when the commit changes any `.gitattributes` file. Attribute changes
can alter Git-filtered bytes for an otherwise unchanged blob, so those cases
have no trustworthy path-only delta from which every stored hash can be
restored.

Live dirty bytes remain visible because the advanced manifest is built from the
commit and Git worktree filters, rather than from the current checkout. The
next scan therefore still reports an unrelated uncommitted edit, while all
paths actually committed at the new HEAD are treated as baseline content.

If an older coordinator stored a partial manifest while its `head_commit`
already equals the current HEAD, normal scanning repairs that epoch from the
same commit-derived manifest. This repair only replaces stale committed hashes;
it preserves the epoch's baseline untracked entries and never reads live bytes
as acceptance input.

## Operational rule

When a baseline reports an unexpectedly large change count, inspect aggregate
attribution and the current HEAD before running a full diff. Full file hashing
is intentionally outside the normal mutation path and must not block session
heartbeats, lease claims, or Cargo scheduling in a large dirty checkout.

Normal `diff`, `scan`, and `reconcile` operations start from the reference
manifest and rehash only Git-reported tracked differences, current untracked
paths, and previously baselined untracked paths. This preserves deletion and
content-hash checks while avoiding a full hash of every unchanged tracked file.
Explicit baseline initialization and acceptance remain full-snapshot operations.

## Regression evidence

`BaselineTests.test_accept_commit_refreshes_all_paths_advanced_by_shared_head`
covers a shared commit whose supplied managed scope is only a subset of the
paths advanced by HEAD. The regression requires the foreign committed path to
match the new baseline and requires the post-advance workspace diff to be
empty.

`BaselineTests.test_accept_commit_updates_from_changed_git_paths_without_full_archive`
covers modified, added, and deleted tracked files. It rejects a full archive
for a new managed commit while requiring the resulting baseline to remain
complete and clean.

`BaselineTests.test_accept_commit_rebuilds_when_attributes_change_filtered_hashes`
requires a full pinned-commit rebuild when a commit changes `.gitattributes`,
protecting filtered-byte integrity over the incremental fast path.

`BaselineTests.test_diff_hashes_only_git_reported_workspace_candidates` ensures
that a normal diff does not call the full-workspace manifest builder.

`BaselineTests.test_scan_repairs_a_stale_manifest_when_head_is_unchanged`
covers historical same-HEAD manifest drift and requires a scan to repair the
stored commit hash without reporting a clean tracked file as workspace change.
