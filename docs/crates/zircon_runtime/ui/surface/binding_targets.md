---
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/surface/binding_transaction.rs
  - zircon_runtime/src/ui/surface/binding_targets.rs
  - zircon_runtime/src/ui/surface/control_index.rs
  - zircon_runtime/src/ui/surface/mutation_snapshot.rs
  - zircon_runtime/src/ui/surface/input/effect/transaction.rs
  - zircon_runtime/src/ui/surface/surface/event_routing.rs
  - zircon_runtime/src/ui/surface/surface/compiled_binding_event_index.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/tests/binding_transaction.rs
  - zircon_runtime/src/ui/tests/event_routing/component_events.rs
  - zircon_runtime/src/ui/tests/event_routing/component_events/performance.rs
  - zircon_runtime/src/ui/tests/asset_binding/apply_report_performance.rs
  - zircon_runtime/src/ui/tests/asset_binding/telemetry_performance.rs
  - zircon_runtime_interface/src/ui/binding/model/execution_receipt.rs
  - zircon_runtime_interface/src/ui/binding/model/mutation_receipt.rs
  - zircon_runtime_interface/src/ui/binding/model/update.rs
implementation_files:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime/src/ui/surface/binding_transaction.rs
  - zircon_runtime/src/ui/surface/binding_targets.rs
  - zircon_runtime/src/ui/surface/control_index.rs
  - zircon_runtime/src/ui/surface/mutation_snapshot.rs
  - zircon_runtime/src/ui/surface/input/effect/transaction.rs
  - zircon_runtime/src/ui/surface/surface/event_routing.rs
  - zircon_runtime/src/ui/surface/surface/compiled_binding_event_index.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/tests/binding_transaction.rs
  - zircon_runtime/src/ui/tests/event_routing/component_events.rs
  - zircon_runtime/src/ui/tests/event_routing/component_events/performance.rs
  - zircon_runtime/src/ui/tests/asset_binding/apply_report_performance.rs
  - zircon_runtime/src/ui/tests/asset_binding/telemetry_performance.rs
  - zircon_runtime_interface/src/ui/binding/model/execution_receipt.rs
  - zircon_runtime_interface/src/ui/binding/model/mutation_receipt.rs
  - zircon_runtime_interface/src/ui/binding/model/update.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
tests:
  - cargo test -p zircon_runtime --lib ui::tests::binding_transaction --locked -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib pointer_binding_target_ --locked -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib compiled_binding_ --locked -- --test-threads=1 --nocapture
doc_type: module-detail
---

# Atomic Binding Targets

`UiSurface` executes template target assignments after pointer default interactions and focus-event
collection, but before the dispatch result is published. The executor currently owns
pointer-originated component events. `UiDocumentCompiler` emits the generation-qualified
`UiCompiledBindingProgram` consumed by this path; other event sources remain outside this module.

## Transaction

For one matching binding, execution has three phases:

1. Resolve every precompiled typed target expression against one retained-surface snapshot.
   Validate boolean-only class/visibility/enabled values and existing action-payload fields.
2. Build the current action invocation, including prepared payload overrides, then capture the five
   writable domain groups: tree/style/invalidation/dirty ownership, focus, input, component state,
   and navigation. Arranged frames, hit-test and render output, text caches, node pools, compiled
   bindings, and other non-writable surface state are not cloned.
3. Apply every prepared mutation directly to the authoritative surface. Success discards the
   rollback snapshot; rejection or `UiTreeError` restores all captured domains before the event can
   publish. The report carries a `UiBindingMutationReceipt` with the base generation, prepared and
   applied and unchanged target counts, scheduled surface revision, complete impact domains, and
   `Committed` or `RolledBack` outcome.

Property, visibility, and enabled targets reuse `UiSurface::mutate_property`; class targets update
template metadata and run the runtime style owner; action-payload targets change only the action
invocation produced by the binding. Reports preserve the binding id as the source path and include
the old/new value plus dirty domains returned by the existing mutation paths.

The transaction receipt is the executor apply authority. Its `base_generation` is the surface
invalidation generation observed before mutation. `revision` advances by one only when a committed
Property, Class, Visibility, or Enabled target actually changes retained surface state; an
unchanged, action-payload-only, rejected, or rolled-back transaction remains at the base revision.
`applied_target_count` and `unchanged_target_count` count target assignments rather than the number
of secondary binding-update rows emitted by component-state synchronization. `impact` unions the
actual dirty domains returned by each mutation, adds Accessibility/Interaction for a focus change,
and records ActionPayload publication as Interaction without pretending that it changed surface
state. Rollback publishes no impact and no new revision.

This receipt is generated incrementally while the executor owns the actual mutation results. The
release gate compares that path with scanning 32 update rows after every report to reconstruct the
same target counts, revision, impact, and outcome. It runs 21 alternating sample pairs over 2,048
receipts per sample; external validation recomputes nearest-rank P95 and requires at least 50%
lower P95 with update scans reduced from 32 per receipt to zero.

## Ordering

Target expressions observe state after ordinary pointer/default-interaction mutations for the
dispatch. All expressions are prepared before any target is applied. Prepared targets commit in
asset order, and the action invocation is constructed before an enabled target can disable the
source node, so that target changes cannot retroactively cancel the event that caused them.

## Compiled identity and cost boundary

Authoring strings remain in source TOML. Compilation assigns domain-specific dense IDs for
bindings, properties, controls, routes, actions, and action-payload fields. A binding handle carries
the compiled artifact generation and binding ID; each target endpoint additionally carries the
compiled node and target slots. Pointer event creation resolves the node/source slot directly, and
the executor validates generation, node, event kind, and interned binding identity before mutation.
A missing, mismatched, or stale endpoint emits a rejected report and suppresses the event.

Every target-bearing binding report also carries at most one `UiBindingExecutionReceipt`. The
receipt identifies the compiled asset, public binding, and generation, then records execution,
miss, error, and elapsed-nanosecond counts for that attempt. Asset and binding identifiers are
bounded to 256 and 128 bytes; over-budget values keep a stable hash suffix. A stale, missing, or
mismatched endpoint is a miss. A valid endpoint that enters target evaluation is an execution, and
a rejected evaluation or transaction is also an error. This receipt is the per-binding authority;
the profiling layer only derives four fixed-name `ui.binding.*` counters from it and never formats
asset or binding IDs into metric names.

The old per-dispatch `BTreeMap<node,event,String,...>` construction, binding clones, whole-surface
transaction clone, target-string lookup, and `UiBindingExpression::parse` calls are removed from
the valid compiled path. Action payload fields carry interned field IDs plus literal or compiled
expression values; pointer and default-interaction dispatch evaluate them directly. Bindings
without targets retain the fast path. Target-bearing bindings borrow the compiled program and
capture only mutation domains required for rollback.

The bounded-telemetry release gate runs 21 alternating sample pairs with 4,096 receipts per sample.
It compares the receipt plus four fixed counter identities with the rejected high-cardinality design
that formats and updates 128 asset/binding/generation metric keys. External validation recomputes
nearest-rank P95 from both raw arrays and requires the bounded receipt P95 to be at least 50% lower.

The transaction performance gate uses 21 paired samples with alternating first-run order. Each
sample captures 64 transactions over a surface with 4,096 retained arranged-frame nodes and
compares that cost with 64 legacy whole-surface clones. The external validator independently
recomputes nearest-rank P95 from both raw sample arrays and requires the transaction P95 to improve
by at least 25%, with `staged_surface_clones=0` and five snapshot domain groups.

## Compiled event lookup

Installing a compiled binding program builds a generation-qualified index from `(node,event)` to
the matching source slots, dense handles, and typed component-event identities. Pointer and typed
default-interaction delivery use that index instead of scanning every authored binding on the node.
Entries preserve compiler order. Once the index generation matches the installed program, dispatch
uses the dense handle directly instead of resolving the source slot, comparing the authored binding
name, and validating the handle a second time.

The index is derived runtime state, not another artifact authority. It is skipped by `UiSurface`
serialization and is usable only while its generation matches the installed program. A surface
loaded without the derived index falls back to the previous authored-binding scan, preserving old
snapshot behavior until the canonical builder installs a program again.

The compiled binding stores `component_event` as an optional serde-defaulted field. Older artifacts
remain readable and mean no typed selector. Default interaction delivery compares this compiled
enum from the derived index, so post-install authored metadata cannot change the event identity.

The event-index performance gate runs 21 paired samples with alternating first-run order. Each
sample performs 4,096 Click lookups on one node with 256 bindings spread evenly across 16 event
kinds. Both paths visit the same 16 matching bindings; indexed dispatch reduces binding probes from
256 to 16 per lookup (93% rounded down). External validation independently recomputes nearest-rank
P95 from raw samples and requires at least 50% lower P95 than the authored scan.

A second 21-pair gate isolates indexed delivery itself: 4,096 lookups with 128 matches compare the
previous source-slot/handle/name revalidation against direct dense-handle delivery. The external
validator requires at least 50% lower nearest-rank P95 and verifies that 128 handle revalidations
and binding-name comparisons per lookup both fall to zero.

## Compiled control lookup

Installing a compiled binding program also maps each `UiCompiledControlId` to a dense optional node
slot. The existing incremental control index remains the authority for uniqueness: one matching
node produces a slot value, duplicates produce `None`, and tracked tree mutations refresh only the
old and new control names. The dense table is generation-qualified and is rebuilt lazily when a
deserialized surface has no matching derived state.

Compiled `ControlProperty` expressions use the dense slot directly instead of decoding the control
name and performing a `BTreeMap<String, ...>` lookup for every evaluation. A final node/control-name
check keeps stale positive entries fail closed. The release gate runs 21 alternating pairs with
8,192 lookups over 2,048 controls per sample. External validation recomputes nearest-rank P95 from
the raw samples, requires at least 25% lower P95, and locks string-index lookups from 8,192 to zero
per sample.

## Event buffer ownership

The pointer binding filter performs stable in-place compaction with `Vec::retain_mut`. It does not
allocate a second retained-event vector or discard capacity reserved by the dispatch owner.
Target-bearing events that publish and target-free events remain in their original order; rejected,
stale, or mismatched endpoints are removed. A propagated tree error clears the entire batch, which
preserves the previous failure contract.

The event-buffer performance gate compares this path with the prior allocating filter using 21
paired samples and alternating first-run order. Each sample filters 32 batches of 512 real component
events with one compiled target mutation per batch. The external validator recomputes nearest-rank
P95 from raw samples, requires at least 15% lower P95, and requires event-buffer allocations to fall
from 32 per sample to zero.

## Component event payload ownership

Pointer component events retain ownership of the typed payload passed into dispatch. Matching
bindings are processed in authored order. When one binding matches, its event envelope takes the
original payload without cloning it. When several bindings match, only the non-final envelopes
clone the payload and the final envelope takes the retained original, reducing the required payload
clones from `N` to `N - 1` without changing the public envelope or binding order.

The payload-move performance gate runs 21 paired samples with alternating first-run order. Each
sample emits 2,048 single-binding keyboard-text events with a 4,096-byte payload. External
validation recomputes nearest-rank P95 from both raw sample arrays, requires at least 20% lower P95
than the former clone-on-every-match path, and locks payload clones from 2,048 to zero per sample.

## Action payload override ownership

Prepared action-payload overrides use the compiled field's `UiPropertyId`, not another owned field
name. Invocation construction consumes this dense-keyed map and moves each resolved value into the
final public string-keyed payload. This retains the existing dispatch contract while avoiding one
field-name allocation and one additional `UiValue` clone per overridden target. The target update
report retains its separate value because commit evidence still needs the previous/new pair.

The dense-override performance gate runs 21 paired samples with alternating first-run order. Each
sample constructs 256 invocations with 16 overridden 1,024-byte string values. The external
validator recomputes nearest-rank P95 from raw samples, requires at least 20% lower P95 than the old
string-keyed cloning handoff, and requires handoff value clones to fall from 16 per invocation to
zero.

Compiled target expressions share fixed source-byte, token, node, and depth budgets with the source
parser and artifact reader. Runtime evaluation uses an explicit frame/value stack, preserves boolean
short-circuit behavior, and returns a rejected binding report when the node or depth budget is
exceeded. Both stacks keep the first eight entries inline and use a bounded spill only for deeper
valid expressions; ordinary shallow bindings do not allocate stack storage. Evaluation does not
recurse through artifact-controlled expression depth.

## Remaining Boundary

This implementation does not yet provide model/provider subscriptions, OneTime/OneWay/TwoWay mode,
frame safe-point batching, a shared Editor/gameplay command gateway, or target execution originating
outside pointer component dispatch. Authoring-only compatibility surfaces without a compiled
program retain source evaluation, but compiled Runtime action payloads no longer parse text.
These requirements remain open in Runtime74.
