---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-template-binding-hash-index.md
  - docs/plans/optimize/zircon_editor/01/2026-08-26-invalidation-scope-hash-index.md
related_records:
  - docs/plans/astra/features/editor/976-editor01-cache-and-menu-index-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/template/adapter/hash_index_tests.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/transaction/hash_index_tests.rs
tests:
  - zircon_editor/src/ui/template/adapter/hash_index_tests.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/transaction/hash_index_tests.rs
---

# Editor977 Editor01 lower contract repairs

The shared Editor debug selector exposed two test-contract defects rather than
production regressions:

- The template-binding source guard rejected the public `contains_binding`
  membership API even though registration already used the required single
  `HashMap::entry` probe. The guard now scopes its assertion to the
  `register_binding` implementation.
- The invalidation snapshot test expected a non-lexicographic
  `first, middle, last` order. `ViewInstanceId` derives lexical `Ord`, and the
  production path explicitly sorts; the fixture now expects the actual stable
  `first, last, middle` order.

Rustfmt and scoped diff checks pass for both repairs. The repairs were made
after the earlier debug binary was built, so no new Rust test result is inferred
until the fresh grouped managed Runtime/Editor source batch compiles them. That
batch was submitted together as Runtime PTY `98618` and Editor PTY `86711`; the
wrappers remain asynchronous and unpolled.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/template/adapter/hash_index_tests.rs` | `8C8C5636884CEEAC6BA04B9454FDE63E25C7A2719A2B6F8106B43BC2B7810A33` |
| `zircon_editor/src/ui/retained_host/app/invalidation/root/transaction/hash_index_tests.rs` | `AADD0EE4524C5C279195F9560301F0BE2C47348BB9E948A388C6C838453314E1` |
