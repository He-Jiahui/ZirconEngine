# Runtime82 surface-session document store preflight

Date: 2026-08-27

Status: `document_prepare_commit_boundary_focused_harness_passed /
exclusive_store_prepare_commit_focused_harness_passed /
explicit_limit_session_store_focused_harness_passed /
snapshot_lease_admission_and_release_focused_harness_passed /
global_manager_registration_rejected / surface_input_session_integration_implemented_unvalidated /
incremental_residency_accounting_profiled_and_implemented /
product_thresholds_and_managed_runtime_pending`

## Lifecycle decision

Current product construction creates one `UiInputManager` beside each `UiSurface` in
`RuntimeUiSurface`. `UiSurface` itself is cloneable and serializable, and its transient secure-value
and clipboard stores deliberately reset on clone/serde. A non-cloneable mutable document authority
therefore must not be embedded in `UiSurface`, nor registered as a process-global UI manager that can
mix node identities from different surfaces.

The next product owner is a surface/input editing session store, matching Unreal Slate's ownership:
`FSlateEditableTextLayout` wraps each editable control's retained layout and uses scoped edit
transactions; it is not a process-global document singleton. Zircon may later share an explicit
model document across surfaces, but that requires a separate registry/principal contract rather than
accidental node-key aliasing.

## Implemented foundation

`TextDocument::replace` now delegates to a two-phase `prepare_replace` / `commit_replace` boundary.
Prepare validates expected key, UTF-8 range, no-op, checked revision/length, hard-line repair, and
next piece topology without mutating revision, chunks, pieces, indexes, or snapshot state. Commit
rechecks the expected key, so a prepared edit cannot overwrite a newer revision.

`TextDocumentStore::with_limits` is the only constructor. There is intentionally no `Default` and no
module registration. The caller must provide document-count, per/total visible bytes, replacement
bytes, per/total retained source bytes, addition chunk, piece, current snapshot, active lease count,
and active lease byte limits. Open/edit/snapshot failures use content-free typed admission reasons.

Changed edits are admitted after exact preparation but before commit; rejected edits publish no
revision and append no immutable source chunk. No-op remains legal at retained-source capacity.
Snapshot admission checks the requested revision and current/active snapshot budgets before flattening.
A managed, non-cloneable lease decrements active count/bytes on `Drop`. Store reports expose only
counts and byte totals.

## Deliberate open work

No numeric policy is guessed in production code. Product-calibrated thresholds, model refresh,
grapheme-bound edit handles, and managed Runtime acceptance remain open. The production input manager
now owns the surface/session bindings, teardown, secure policy, and changed public receipt path. Store
lookup remains `O(log D)` through `BTreeMap`; the 1/16/256/1024-owner matrix showed aggregate report
reconstruction, not lookup, was the dominant multi-document term. Store-owned incremental residency
accounting now removes that `O(D)` admission scan. An `O(1)` UUID index still requires separate
evidence that lookup is material.

The prepare path is bounded by admitted document and replacement bytes and only rebuilds the local
hard-line envelope, but it has no independent time/work token yet. Profile rejection-heavy and
separator-dense workloads before adding a work budget. Do not use chunk/piece limits as a substitute
for compaction evidence; those limits are containment until the storage residency matrix selects the
algorithm.

## Verification state

Rustfmt/static whitespace checks pass. The current Interface library check passes. An E-drive direct
current-source document harness passes `53/53`, covering no mutation during prepare, stale commit,
cross-document UUID rejection, explicit store admission, no-op at capacity, chunk/piece limits,
snapshot budget/Drop release, current-snapshot preflatten denial, and public receipt projection
before commit. It also covers dropping an exclusive prepared store edit without publication and the
infallible commit result matching its prospective public receipt. It now also covers incremental
residency through open/edit/snapshot/re-edit/close and complete report stability after dropping a
prepared edit. This harness is focused source evidence, not a full Runtime pass.

The default Runtime check was blocked before text by concurrent untracked `zr_rhi_wgpu` readback type
errors. The latest text-only Runtime module-graph check reached text but failed with 95 workspace
errors and 198 warnings across concurrent text shaping/layout work and unrelated core/scene/platform owners;
none named the document store files. No WGPU/profile/power/PNG claim is made here.
