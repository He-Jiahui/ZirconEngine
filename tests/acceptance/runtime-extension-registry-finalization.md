# Runtime Extension Registry Finalization Acceptance

## Scope and Owning Layers

- Owner: `zircon_runtime::plugin::extension_registry`.
- Plan path: Runtime 06 plugin lifecycle with Runtime 15 public-surface and module-structure constraints.
- Behavior: registration epochs, stable logical extension slots, owner revocation, catalog finalization, world/application entry points, and the frozen table projection.

## Baseline Before This Slice

- `ExtensionSlot` was the current dense vector index.
- Removing one owner compacted the vectors and reassigned every surviving slot after the removed row.
- The first finalization draft cloned keys, owners, descriptor values, and key indices for every typed point while keeping the staging copies alive.
- The draft also exported the mutable `TypedExtensionPoint` storage type from the root plugin facade.

## Required Invariants

1. A surviving contribution keeps the same logical slot after another owner is revoked.
2. A revoked slot resolves to no key, owner, or value and is never reused by a later registration.
3. Sorting dense rows does not change logical slots.
4. `FrozenExtensionTable` preserves the same logical-slot mapping.
5. Finalizing a live registry does not clone descriptor payloads into a parallel cache.
6. Any registration, mutation, sort, or owner revocation clears the finalized state; a later finalize publishes the new registration epoch.
7. The root plugin facade does not expose the mutable typed-point storage implementation.
8. Registering or revoking a non-typed asset importer participates in the same finalized registry epoch.
9. A default world extension set is finalized before its first runtime read, and a failed install leaves the prior finalized registry unchanged.

## Test Inventory

- `zircon_runtime/src/plugin/extension_registry/typed_extension_point/tests.rs`
  - `frozen_table_dense_lookup_matches_registration`
  - `duplicate_extension_key_rejected`
  - `owner_revocation_preserves_survivor_slots_and_retires_removed_slots`
  - `sorting_dense_rows_preserves_logical_slots`
- `zircon_runtime/src/plugin/extension_registry/runtime_extension_registry/tests.rs`
  - importer revocation directly clears the private importer finalized-state bit, independent of typed-point thawing
- `zircon_runtime/src/tests/plugin_extensions/extension_registry_typed_points.rs`
  - catalog finalizes before report/apply
  - a write invalidates finalization and re-finalize restores it
  - owner unload preserves surviving rows
  - asset importer registration and revocation invalidate the registry epoch
- `zircon_runtime/src/core/runtime/state/world_runtime_extensions/tests.rs`
  - a default empty set can be applied immediately
  - a partially merged failed install is transactional

## Boundary and Failure Cases

- Duplicate keys remain rejected with the original slot.
- Removed slots remain tombstones after a plugin reload registers a replacement contribution.
- Dense ordering can change independently of stable logical identity.
- Catalog and world apply paths finalize idempotently.
- Package-level Cargo validation must compile the changed public method receivers and all registry descriptor types before acceptance.

## Tool Matrix and Results

| Check | Result | Evidence |
|---|---|---|
| Scoped rustfmt | passed | `rustfmt --edition 2021 --check` on the typed registry, registration iterators, facade and tests |
| Standalone typed-point tests | passed, 4/4 | `rustc --edition 2021 --test` wrapper over the real `typed_extension_point.rs`, then `--test-threads=1 --nocapture` |
| Finalization structure coverage | passed, 4/4 | `python -m unittest tools.tests.test_plugin_extension_registry_finalize_coverage -v` verifies all 20 typed fields, hash-free frozen ownership, no public dense-index compatibility API, catalog/apply finalization ordering, and transactional world install |
| Stale dense-index construction scan | passed | no tracked `ExtensionSlot::from_raw` callers remain in Rust sources |
| Mutable storage facade leak scan | passed | no `TypedExtensionPoint` export remains in `plugin/mod.rs` or `extension_registry/mod.rs` |
| Scoped diff health | passed | `git diff --check` returned only line-ending normalization warnings |
| `zircon_runtime` core-min library check | passed | `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plugin-typed-extension-freeze-0710 --message-format short --color never`; completed with existing warnings |
| Focused `zircon_runtime` Cargo tests | blocked before target test | latest `cargo test -p zircon_runtime --lib extension_registry_typed_points ...` reached the lib-test build, then failed in unrelated active `zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge/basics.rs:216` with `E0282`; an earlier attempt was blocked by the then-active font fixture `E0505`. No target test pass is claimed |

## Acceptance Decision

Not accepted yet. The lower typed-point behavior, structure coverage, formatting, and core-min library compilation pass. The focused registry/world lib tests still need execution after the unrelated active plugin-bridge test inference blocker is resolved. Runtime 06 and the full runtime architecture goal remain `in_progress`.
