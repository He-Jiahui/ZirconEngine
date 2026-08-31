# Runtime74 Layered Model Data Context

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-23-layered-model-data-context.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_runtime/src/ui/binding/model_schema_registry.rs","zircon_runtime/src/ui/tests/mod.rs","zircon_runtime/src/ui/tests/model_context_registry.rs","zircon_runtime_interface/src/tests/mod.rs","zircon_runtime_interface/src/tests/model_context_contracts.rs","zircon_runtime_interface/src/ui/binding/mod.rs","zircon_runtime_interface/src/ui/binding/model/mod.rs","zircon_runtime_interface/src/ui/binding/model/model_context.rs"]

- Date: 2026-08-23
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-006`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- The interface defines the canonical surface, component, row, and item model-context layers.
- Sparse patches encode three distinct operations per layer: absent means inherit, `Bind` replaces
  that layer with one complete versioned provider key, and `Clear` removes the inherited provider.
- Resolved contexts retain unrelated layers while applying overrides in canonical order and expose
  deterministic layer/provider iteration.
- Runtime resolution validates the complete merged context against `UiModelSchemaRegistry`,
  including inherited state supplied by a caller. Unknown IDs and wrong provider versions fail
  closed with the exact layer and provider key.

## Reference Evidence and Divergence

- Slint's typed property and two-way binding cases support lexical context inheritance while
  preserving explicit model/property identities across component boundaries.
- Bevy's type registry supports exact complete-key validation and explicit rejection of missing or
  conflicting registrations rather than implicit latest-version lookup.
- Godot property metadata supports carrying type authority with a context owner rather than
  resolving fields from untyped strings at mutation time.

Zircon makes surface/component/row/item layers explicit and serializable because authored assets,
cooked artifacts, Editor previews, and gameplay runtime must share the same context vocabulary.
This slice intentionally does not install live provider objects, subscriptions, model field reads,
collection adapters, or writeback; those remain owned by later Runtime74 tasks.

## TDD and Validation Contract

Tests were authored before the context implementation. Interface coverage locks stable four-layer
order, TOML round-trip, inheritance, binding override, and explicit clear. Runtime coverage locks
exact provider-version validation, cross-layer preservation, and revalidation of inherited caller
state.

The grouped Runtime74 submission `caf7bfeb2eed4e3e9452e78fd45aed36` / request
`a97a2f548668430b997b32ec2891c14b` covered 88 tasks, 62 Cargo groups, 20 new behavior tests, and
18 existing performance rows under validator SHA-256
`E93B9E81B8EFA1225CDA3B5CF5632687E7CA29C1A02C20C4614342A91D6BAFB1`. It failed during
validation-copy `closure_planning` with `validation_copy_state_forbidden`, before Cargo started.
No behavior pass, performance result, or commit is claimed; grouped validation remains pending.

The forward grouped submission `a2c39ddcdd944d588daa96cd7c99d512` / request
`d92db795584a4c4e8a561e6d3df175e1` is queued asynchronously without waiting. It covers 89 tasks,
65 Cargo groups, 30 cumulative new behavior tests, and 18 performance rows under root validator
SHA-256 `D84C8CA2B28C1EE4137D0CCC580FB601ED34F7F4E4084081E1AA0BEC75701ACB`; its 245-path,
7-tombstone source manifest is `6d2edcabe8fb82f2971f30f13d908d13899a148aa747ce75ae863a87c2582063`.
This receipt is submission evidence only; acceptance remains pending.

## Performance

Context merge inspects exactly four fixed layers; provider validation performs at most four
`BTreeMap` lookups. This descriptor-only slice is not on frame or event execution paths and adds no
release benchmark row. Measured model read/subscription performance remains pending for the later
provider execution milestones.
