Plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
Milestone: M3
Status: source_bound_managed_acceptance_pending
Files: ["docs/plans/zircon_editor/editor/08/2026-08-13-m3-command-palette-query-index-current-source-manifest.md", "docs/plans/zircon_editor/editor/08/failure-2026-07-17-command-palette-catalog-clone-and-full-row-paint.md", "docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md", "docs/zircon_editor/core/commands.md", "tools/tests/test_editor08_command_palette_query_contract.py", "tools/tests/test_editorui06_command_palette_paged_keyboard_contract.py", "zircon_editor/src/core/commands/eval_snapshot_handle.rs", "zircon_editor/src/core/commands/palette.rs", "zircon_editor/src/core/commands/registry.rs", "zircon_editor/src/tests/commands/descriptor_when.rs", "zircon_editor/src/ui/retained_host/app/command_palette_actions.rs"]
Depends-On-Failures: ["docs/plans/zircon_editor/editor/08/failure-2026-07-17-command-palette-catalog-clone-and-full-row-paint.md"]

# Editor08 M3 Command Palette Query Index Current-Source Manifest

## Scope Delivered

This exact manifest freezes the remaining Editor08-owned command palette query
architecture without absorbing the EditorUI painter or deep-page component
owners.

- Each immutable catalog generation owns normalized search documents, a
  256-byte postings index, descriptor-aligned enablement slots, and lightweight
  entry handles.
- Non-empty queries choose the least-populated byte posting without truncating
  the candidate set. One byte pass per candidate preserves exact-substring and
  greedy-subsequence scores, then the bounded heap retains only
  `offset + limit` handles while reporting the full match count.
- `EditorCommandRegistry` only publishes the shared catalog `Arc`. Retained host
  code releases the registry mutex before enablement, fuzzy matching, MRU
  ranking, deep-page window materialization, or UI-value projection.
- `CommandEvalSnapshotHandle` publishes one shared `Arc<CommandEvalCtx>` per
  semantic generation. Palette input clones the Arc rather than the capability
  strings while non-hot callers retain an explicit owned-snapshot API.

The registry query facade is deleted. No compatibility wrapper, painter-owned
catalog, second cache authority, or full result-id vector remains.

## Fresh Testing Evidence

- `python -B -m unittest tools.tests.test_editor08_command_palette_query_contract -v`
  passed `4/4`; the initial three contracts first failed against the old
  registry facade, lock scope, and two-pass matcher, and the added shared
  context checks first failed `2/2` before the Arc generation was implemented.
- Rust regression sources cover one selective match in a 1,001-entry catalog,
  full match count plus deep window handles, exact/subsequence score retention,
  repeated query bytes, a later exact match overriding an earlier subsequence,
  descriptor `when` equivalence, shared context generation, and 1,000-query
  metrics.
- Retired `command_palette_query_window` Rust symbol scan returned `0`.
- The existing EditorUI06 paged-keyboard consumer contract now requires the
  catalog Arc/query path and still guards query/generation stale rejection; it
  no longer names the removed registry facade.
- Scoped `rustfmt --edition 2021 --check` and `git diff --check` passed for the
  exact Rust, Python, plan, and module-document scope.
- No direct Cargo command was run. Current-source Rust behavior, p95, and
  product/pixel evidence remain coordinator-managed gates.

## Review

Independent read-only second review found one Important stale-consumer contract:
the EditorUI06 paged-keyboard Python guard still named the deleted registry
query facade. The contract is updated in this manifest to require the catalog
Arc/query path while retaining stale query/generation checks. No algorithm,
lock, shared-snapshot, or hard-cut finding remained in that review pass. Final
read-only re-review reported `Critical/Important/Minor = 0/0/0` on the 11-path
tree.

## Remaining Acceptance

The failure record remains open until managed current-source Cargo, 1,000 input
p95, disabled/commit interaction, and pixel-equivalence evidence complete.
Those gates delay accepted closeout only. They do not authorize restoration of
the registry query facade, full-catalog UI projection, or offscreen painter
work.
