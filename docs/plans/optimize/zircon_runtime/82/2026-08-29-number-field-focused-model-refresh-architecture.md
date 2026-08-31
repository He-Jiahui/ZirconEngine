# Runtime NumberField focused model-refresh architecture review (2026-08-29)

## Status

`architecture_review_complete / revision_infrastructure_implemented_unvalidated /
typed_float_gateway_implemented_unvalidated / owner_incarnation_invalidation_implemented_unvalidated /
unrelated_topology_key_stability_implemented_unvalidated / source_regressions_added /
managed_validation_pending`

This record is a correctness and ownership review, not a performance acceptance record. No cache,
parser, formatting, allocation, or power optimization starts in this slice.

## Current-source finding

The String-only `UiTextModelUpdateRequest` correctly rejects `NumberField`, because its CAS key owns
a retained String document while `NumberField.value` is a canonical Float. Widening that request to
`UiValue` would mix two revision domains and make a text document revision falsely authoritative for
numeric model changes.

The generic `UiPropertyMutationRequest` already accepts an external Float, but intentionally behaves
like an explicit replacement: it projects formatted text and closes the active edit state. It cannot
also represent a bound refresh that must preserve focused user input.

Canonical Float writes currently enter through four product paths:

1. optional valid per-key publication from the editable property transaction;
2. Enter/focus-loss/keyboard-step commits through the NumberField property transaction;
3. accessibility `SetValue` through the same typed property prepare/commit owner;
4. generic explicit property mutation through `commit_editable_text_properties_with_value`.

Adding a model revision outside that shared prepare/commit boundary would miss at least one writer and
produce false CAS success.

## Unreal reference

Primary reference is `SSpinBox.cpp`:

- `ValueAttribute` remains distinct from the editable String;
- `GetValueAsString` reads the external attribute while display text uses a bounded cache;
- `CommitValue` calls `ValueAttribute.UpdateNow`, detects external force through
  `CachedExternalValue`, and treats typed/spin/arrow sources explicitly;
- construction and display read a bound external value directly, including a value outside the
  declared min/max; range clamp and optional delta snap belong to user commit/spin/arrow paths, not
  bound-model ingestion;
- `EnterTextMode` and `ExitTextMode` make outer numeric focus and inner text focus separate states.

Zircon has one retained node and one focus target in the MVP. It therefore needs explicit revisions to
preserve the same authority separation without pretending that focus identity alone identifies the
editor layer.

## Required state machine

Each NumberField owns two monotonically increasing runtime integers:

- `number_value_revision`: revision of the canonical Float;
- `number_edit_base_revision`: canonical revision observed when the active buffer began or last
  published its own accepted Float.

Rules:

- a real canonical Float change advances `number_value_revision` exactly once in the same property
  transaction; unchanged writes do not advance it;
- beginning an edit captures the current value revision;
- accepted per-key publication advances both value revision and edit base as one self-authored write;
- bound Float refresh applies immediately to canonical value and advances only value revision while
  preserving active buffer/base;
- Enter on a stale edit base returns a typed conflict and preserves the buffer;
- focus loss on a stale edit base restores current canonical display and closes editing without
  publishing the stale user value;
- Escape and keyboard step intentionally discard the buffer and operate from current canonical value;
- explicit SetValue and accessibility actions replace canonical/display state and reset the edit base.

Revision exhaustion is a typed, pre-write rejection. TOML runtime state is limited to non-negative
`i64`; public DTOs must not imply a larger accepted range.

## Public gateway boundary

The numeric gateway must use a separate versioned request/receipt with Float value, tree/node identity,
request identity, expected numeric revision, origin, status, and typed failure. It must not carry a
text document UUID/revision or emit a text edit receipt.

`BoundRefresh` applies immediately and preserves the active edit buffer. `ExplicitSetValue` applies
immediately and closes the buffer. Receipts are content-free except for the requested/current numeric
revision and may report `Applied`, `Unchanged`, `Conflict`, or `Rejected`.

The implementation is synchronous and does not retain a second pending value. Numeric values do not
need the String gateway's pending-value queue, secure String store, or byte budgets. Adding a second
queue would create lifecycle complexity without preserving more user state.

## Implementation result

`NumberField` descriptor/runtime state now owns `number_value_revision` and
`number_edit_base_revision`. Every canonical writer that uses the shared numeric property transaction
projects those revisions in the same preflight/commit as Float, display text, edit-active, caret,
selection, and composition state. Non-negative `i64` validation and checked revision advance happen
before the metadata batch; exhaustion and malformed state are typed zero-write failures.

The Runtime Interface now exposes a separate `UiNumberModelUpdateRequest`/receipt family with a
manager-issued numeric model UUID, expected numeric revision, `BoundRefresh`/`ExplicitSetValue`
origin, and content-free status/failure fields. The `UiInputManager` gateway owns model identities per
Surface session, rejects stale keys before mutation, preserves focused `value_text` for bound refresh,
and routes explicit replacement through the same numeric transaction while closing edit mode.

Model-key issuance now validates the complete retained authority, including a finite canonical TOML
Float and the canonical/edit-base/edit-active revision invariant. `UiTreeNodes::insert` is now the
single allocator for the tree-lifetime monotonic insertion serial that also backs paint order;
`clear()` preserves the retired high-water mark, and `UiTree::node_incarnation` exposes that serial as
a read-only lifetime identity. The numeric manager stamps each cached model UUID with only its owner's
incarnation. Detach/reinsert with the same `UiNodeId` therefore reissues that UUID, while unrelated
sibling insertion, removal, layout, or property mutation keeps the key stable. Layout-order generation
remains only the gate for an `O(number model identities)` detached-owner prune after topology changes;
stable synchronization has no owner scan, no incarnation queue, and no second identity map.

Enter detects a stale edit base and reports `UiNumberInputCommitStatus::Conflict` without replacing
the buffer. Focus loss resolves the same conflict by displaying the latest canonical Float and
closing the edit. Escape and canonical keyboard step keep their intentional discard semantics.
Product-source regressions cover focused preservation, Enter/blur resolution, explicit replacement,
stale CAS, non-finite/revision-exhaustion zero-write rejection, generic internal-state bypass
rejection, generic/accessibility canonical-writer revision advance, and Surface-session identity
replacement. A retained node-pool detach/reinsert regression additionally proves that same-node-id
reuse changes model UUID and that the old key conflicts without a property write; malformed non-Float
canonical state cannot obtain a model key. An unrelated-sibling insert/detach regression additionally
requires the NumberField model key to remain unchanged across both topology generations.

## Profiling and acceptance plan

Five fixed counters now cover request count, focused-buffer preservation, CAS conflict, revision
advance, and rejection. Measure the unoptimized product route for p50/p95/p99 CPU
time and allocation count under focused/unfocused, unchanged, applied, and conflict cases. Compare the
same behavior with Unreal bound `SSpinBox` update/commit scenarios; do not infer power parity from wall
time.

Dynamic acceptance remains gated on the managed Windows validator, product input dispatch, actual
binding consumption, and a real WGPU frame showing that a focused buffer remains visible while the
canonical value changes underneath it. Any screenshot belongs in `docs/tests/runtime/text`, never in
`target`. No screenshot is produced for this architecture-only record.
