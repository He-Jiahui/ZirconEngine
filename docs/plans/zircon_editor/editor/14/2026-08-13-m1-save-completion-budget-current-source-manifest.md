Plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
Milestone: M1
Status: source_bound_managed_acceptance_pending
Files: ["docs/plans/zircon_editor/editor/14/2026-08-13-m1-save-completion-budget-current-source-manifest.md", "docs/plans/zircon_editor/editor/14/failure-2026-07-23-autosave-job-admission-and-save-mutex-adapter.md", "docs/plans/zircon_editor/editor/14/failure-2026-08-01-interactive-save-batch-admission-lane.md", "docs/zircon_editor/core/jobs.md", "docs/zircon_editor/core/recovery.md", "tools/tests/test_editor14_interactive_save_job_adapter_contract.py", "zircon_editor/src/core/asset/dirty/save_job_adapter.rs", "zircon_editor/src/core/asset/dirty/save_job_adapter/tests.rs", "zircon_editor/src/core/recovery/autosave_adapter.rs", "zircon_editor/src/core/recovery/mod.rs", "zircon_editor/src/core/recovery/tests/autosave_adapter.rs"]
Depends-On-Failures: ["docs/plans/zircon_editor/editor/14/failure-2026-07-23-autosave-job-admission-and-save-mutex-adapter.md", "docs/plans/zircon_editor/editor/14/failure-2026-08-01-interactive-save-batch-admission-lane.md"]

# Editor14 M1 Save Completion Budget Current-Source Manifest

## Scope Delivered

This exact manifest freezes two M1 completion paths that share the one
`EditorJobSystem` admission owner without merging their domain state.

- Interactive Save preallocates fixed document completion slots after atomic
  batch admission. Each ticket retains only a slot index, each terminal outcome
  performs an O(1) accumulator write, and the terminal batch transfers ownership
  without rebuilding a keyed container.
- Autosave retains pending tickets in one `VecDeque`, inspects at most the
  explicit ticket budget per poll, accumulates success/failure counts across
  ticks, and advances its scheduler exactly once when the admitted batch is
  terminal.

The old Interactive `BTreeMap` completion API and the Autosave whole-vector
completion scan are removed. No compatibility facade, private worker, second
queue authority, or synchronous save fallback is retained.

## Fresh Testing Evidence

- `python -B -m unittest tools.tests.test_editor14_interactive_save_job_adapter_contract -v` passed `3/3`.
- Scoped `rustfmt --edition 2024 --check --config skip_children=true` passed for
  the five Rust sources in this manifest.
- Static Autosave completion checks reject a whole-vector `take`, require the
  64-ticket default and explicit bounded inspection, and cover zero budget,
  blocked-head rotation, mixed terminal accumulation, reset, and next-interval
  scheduling.
- Scoped `git diff --check` passed for all declared source, test, plan, and
  module-document paths.
- No direct Cargo command was run. Current-source Rust behavior remains a
  coordinator-managed gate.

## Review

Independent read-only final review reported `Critical/Important/Minor = 0/0/0`
for both slices. Interactive cancellation evidence is reachable after releasing
the shared save mutex blocker. Autosave's budget-one loops observe ticket state
rather than snapshot-capture timing, do not assume success/failure terminal
order, and retain the cumulative-not-delta API contract.

## Remaining Acceptance

The two failure records remain open until managed current-source Cargo and their
declared upward/product gates complete. Interactive Save still requires the
owner-complete Autosave/Settings source closure, scale/WPR evidence, and
Editor06/09 acceptance. Autosave still requires PERF-MVP-592 and Editor17
acceptance. These external gates delay accepted closeout only; they do not
restore the removed completion paths or authorize fallback behavior.
