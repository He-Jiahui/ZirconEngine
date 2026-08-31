# Runtime82 focus-loss composition cancellation

Date: 2026-08-28

Status: `current_source_and_unreal_review_complete / architecture_selected /
implementation_complete_unvalidated / focused_current_source_1_of_1_passed /
focus_loss_owner_queue_2_of_2_passed / managed_runtime_and_wgpu_acceptance_pending`

## Problem

`UiSurface::disable_input_method_for_focus_loss` currently applies `CommitComposition` directly to
Surface properties. The retained text document and its revision authority live in
`UiInputManager::text_documents`, so this path mutates committed source and advances the Surface
source epoch without a document transaction or public receipt. The next managed edit consequently
has to rebind a new document instead of preserving one UUID and adjacent revisions.

This is an ownership error, not a missing local synchronization call. Threading the manager-owned
document session into `UiSurface` focus mutation would reverse the existing dependency boundary and
still make programmatic focus changes dependent on a caller supplying a transaction authority.

## Unreal reference review

`FSlateEditableTextLayout::HandleFocusLost` in
`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/SlateEditableTextLayout.cpp`
establishes the required order and ownership:

- lines 989-994 cancel an active text-input-method composition before deactivating its context;
- lines 1003-1029 publish the committed-text notification for a non-read-only widget;
- lines 1024-1025 clear the widget-local undo chain on focus-loss commit;
- the notification uses the committed editable value after composition cancellation rather than
  converting preedit into a new source edit.

Zircon must preserve that semantic split: preedit is transient Surface state, retained source edits
are manager transactions, and focus loss is a lifecycle boundary rather than a hidden document edit.

## Selected architecture

1. `finish_editable_text_for_focus_loss` applies `UiTextEditAction::CancelComposition` when preedit
   exists. It commits that restored state without a `CommittedTextEditIntent`, so composition cleanup
   does not advance the text document source epoch.
2. Every non-read-only editable focus loss publishes `UiComponentEvent::Commit` for a bound Submit
   route, whether or not preedit existed. The payload is the restored committed source; secure text
   uses the existing opaque `SecureCommit`. Read-only still permits transient cleanup but publishes
   no Commit.
3. Surface records actual editable focus-loss owners independently of input-method requests.
   `UiInputManager` drains this set before orchestration and after a managed dispatch, then removes
   each owner-local history.
4. Do not infer loss only from the manager's last observed `focused` node or from IME Disable. A
   programmatic blur-and-refocus can occur between manager calls, while secure or explicitly
   deactivated inputs may have no IME owner at all. The focus transition is the authoritative signal.
   The set is deduplicated and capped at 1,024 owners; overflow fail-closed clears all histories.
5. Do not create a document receipt or revision for cancellation. The next real edit must reuse the
   same document UUID and advance from the last committed revision. Undo after focus loss must be
   handled locally as unavailable because the previous local history was cleared.

## Complexity and performance

- composition cancellation does not flatten or reopen the retained document, but current Surface
  state/property projection still owns complete Strings and therefore remains `O(N)`; the separate
  Surface edit projection profiler now measures that open P1-17 cost;
- history invalidation is one `BTreeMap::remove`, `O(log D)` for `D` active document bindings;
- lifecycle collection inserts one owner into a bounded `BTreeSet`, not a UI-tree scan. Overflow
  performs a fail-closed history clear rather than retaining an unbounded queue;
- no new timer, coordinator polling, or per-frame tree work is introduced.

This is a correctness/ownership repair. It does not claim a measured optimization and therefore
does not change history budgets or performance thresholds before managed profiling.

## Acceptance

- focus loss restores the committed source and clears preedit/composition properties before Disable;
- non-read-only focus loss publishes Commit with the restored source even without preedit; secure
  text publishes only an opaque Commit reference; read-only focus loss does not publish;
- hidden, disabled, detached, direct clear-focus, and dispatch-driven lifecycle paths share the same
  cancellation behavior;
- deferred programmatic blur clears only that owner's history, including blur/refocus before the
  next manager call;
- the next real edit preserves document UUID and advances from the previous committed revision;
- focus-loss cancellation emits no text edit receipt and creates no undo entry;
- scoped direct tests and static checks precede managed Runtime and WGPU acceptance.

## 2026-08-28 implementation status

Completed in source:

- focus loss now applies `CancelComposition`, restores committed source through the shared fixed-ten-
  property transaction, clears composition state, and supplies no committed edit intent. Therefore
  this state-only repair does not advance the Surface document epoch or manufacture a receipt;
- read-only transition no longer traps an already-active composition. Cleanup still runs, while the
  focus-loss Commit notification is suppressed for read-only text;
- `UiSurfaceInputState` retains actual focus-loss owners in a deduplicated 1,024-owner set.
  `UiInputManager` drains it before orchestration and after managed dispatch; overflow clears all
  histories fail-closed;
- history invalidation is owner-local and works when programmatic blur and refocus both occur before
  the next manager call. The product regression then requires the next real edit to reuse the first
  receipt's document UUID and advance revision `1 -> 2`;
- direct focus lifecycle regressions now expect restored committed source for clear, hidden,
  disabled, and focused-detach paths. An unfocused invalid IME owner only cancels preedit and does not
  synthesize a focus Commit. Additional regressions require a normal Commit without preedit, an
  opaque secure Commit without an IME owner, no text-edit receipt on cancel, and no read-only Commit.
  These product tests remain managed-Runtime unvalidated;
- scoped Rustfmt and whitespace checks pass. An E-drive executable that includes the current
  `edit_state.rs` passes `1/1` for cancellation after a read-only transition. Its simplified
  character-boundary dependency is used only by this ASCII-focused check and is not Unicode layout
  acceptance. A second current-source executable passes `2/2` for focus-owner deduplication and
  overflow fail-closed signaling.

Still open:

- managed Runtime compilation and product regression execution;
- platform IME lifecycle verification and genuine WGPU text-rendering screenshot acceptance;
- no screenshot was produced by this non-rendering ownership slice.
