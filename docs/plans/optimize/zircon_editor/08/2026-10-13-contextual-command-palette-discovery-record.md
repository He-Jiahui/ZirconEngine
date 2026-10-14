---
title: Editor Contextual Command Palette Discovery Record
category: zircon_editor
report_id: Editor-contextual-command-palette-discovery-2026-10-13
date: 2026-10-13
implementation_status: source_reviewed
validation_status: combined_native_validation_pending
performance_status: no_cold_build_p95_claim
---

# Editor Contextual Command Palette Discovery Record

## Problem and scope

The global palette catalog previously discovered payload-dependent metadata-only operations. A generic palette commit could select one of these rows and reach `MissingFactory` because its contextual payload route is supplied by a specialized UI path. The repair adds an explicit descriptor participation flag for global palette discovery only. It does not filter by factory presence, add a fake factory/executor, guess a target, or add a new contextual execution route.

The five default contextual-only operation IDs are:

- `view.editor.ui_asset.open`
- `timeline_sequence.authoring.open`
- `animation_graph.authoring.open_graph`
- `animation_graph.authoring.open_state_machine`
- `inspector.field.apply_batch`

They remain in the real registry with their operation metadata; only global palette rows are omitted. The default registry still contains 68 metadata entries, while its palette catalog contains 63 visible entries. The catalog test also confirms a normal visible `file.project.open` entry remains discoverable and its existing registry event route is available. No command invocation implementation changed.

## Descriptor and catalog behavior

`EditorCommandDescriptor::new` defaults `palette_visible` to true. Serde defaults a missing field to true and omits the default-true field during serialization, so legacy descriptor JSON continues to round-trip canonically. An explicit true also serializes canonically without the field; false remains explicit and survives round-trip. The source scan found no foreign struct literal that requires updating.

Only the five contextual builtins opt out. `EditorCommandRegistry` passes a private `PaletteCommandDescriptors` iterator into the existing `EditorCommandPaletteCatalog::from_descriptors` constructor. The iterator counts visible descriptors once over a cloned `BTreeMap::Values` iterator, then yields the visible descriptors with an exact remaining `size_hint` and `len`. This preserves the real allocation reservation lower bound without a temporary collected `Vec` or a factory-presence filter. The count adds one boolean-count pass over registry metadata per cold catalog build/generation; it is not timed here, and no CPU or P95 improvement is claimed.

The unchanged palette consumer builds dense indices from the filtered visible catalog. Its unfiltered MRU path resolves IDs through that visible index and skips hidden IDs; its fuzzy path ranks visible projected entries by MRU order. The new registry tests exercise both consumers with hidden IDs present in the MRU. Existing `OnceLock` cache sharing within a registry, fresh cache on clone/reload, invalidation after hidden or visible registration, and immutability of an old `Arc` catalog are covered.

## Regression coverage

Five new ordinary tests are wired through `registry.rs` to `registry/tests/cases.rs`:

- `core::commands::registry::tests::default_palette_keeps_contextual_operations_registered_without_exposing_rows` checks the real 68/63 counts, all five retained descriptors and their missing palette rows, plus visible `file.project.open` discovery/routing.
- `core::commands::registry::tests::palette_visibility_defaults_for_legacy_descriptors_and_round_trips_opt_out` checks missing/true/false serde behavior and registry admission of hidden operation metadata.
- `core::commands::registry::tests::palette_visible_iterator_retains_exact_bounds_across_filtered_positions` checks exact remaining bounds for hidden head, middle, tail, all-hidden, and empty inputs, and compares real catalog output.
- `core::commands::registry::tests::palette_mru_and_paging_use_visible_catalog_indices` checks empty and fuzzy queries over two pages with hidden IDs in the MRU. Visible order is `[4, 2, 1]`, total matches/visits/enablement evaluations are 3, and candidate-handle/buffer counts remain asserted.
- `core::commands::registry::tests::palette_visibility_survives_registry_reload_clone_and_catalog_invalidation` checks registry JSON reload, independent clone cache, generation invalidation after both hidden and visible registration, and unchanged results through an old `Arc`.

The full normal module filter is `core::commands::registry::tests::` with a minimum of 16 cases. All 11 preexisting normal case bodies remain intact; the five above are the new Rust cases. Existing descriptors, menu projection, native execution admission, palette query, and query-burst regressions remain in the module.

## Preserved performance and native contracts

The existing `EDITOR811_DEFAULT_COMMAND_DIRECT_APPEND_BENCH_V1` producer remains unchanged: it feeds all 68 default descriptors directly to `default_workbench_commands()`, checks the 68-command capacity contract and the original geometric legacy-growth estimate, and retains its original modeled `optimized_growth_events=0` field. The visible-catalog count of 63 does not reduce this producer's 68-input workload. No new P95 gate or measured growth claim is attached to it.

The read-only `palette.rs`, the Editor811 producer file, and the retained UI test containing native Enter behavior were unchanged. The native OpenProject Enter test, locale/MRU producers, performance measurements, product UI acceptance, and MVP acceptance remain open. No native, product, Python unittest, or performance test was executed for this review.

For the next combined selector set, preserve the previous #3 selector and its 11 existing argv filters with its floor of 16. Add the entire `core::commands::registry::tests::` namespace (16 selected cases, zero overlap with the old filters), giving an aggregate selected floor of 32. This adds five source tests but selects 16 cases through the new namespace; it does not represent 16 newly authored tests. Batch11 has not been formally handed off.

## Frozen source and review artifacts

- Patch: [editor-contextual-palette.patch](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor-contextual-palette.patch), SHA-256 `bf6d8860b8d2147ec6c0273fbe6011a2763f1f9c790a0d5cf7d1c3376aede292`.
- Source and contract evidence: [editor-contextual-palette-evidence.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor-contextual-palette-evidence.json), SHA-256 `211a45ede6f9dc59189c6d99303da184c6dedfcd6b358dffda25c125f3cbae93`.
- Exact root guard: [editor-contextual-palette-guard.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor-contextual-palette-guard.json), SHA-256 `c021834568c399bd44fdc1ceda09ac10f2df1ca9d5e4604dcf7ab5ef532ab5eb`; passed with zero errors, warnings, or suppressed findings.
- Previous combined selection source: [batch10-validation-contract.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/batch10-validation-contract.json), SHA-256 `e2b75b4a129494d2007794fc02e76d160fa415a50c714727129ca5408b9b95a1`. It is the prior contract; the next #3 addition described above remains pending handoff.

| Path | Before raw SHA-256 | Reviewed raw SHA-256 |
| --- | --- | --- |
| `zircon_editor/src/core/commands/descriptor.rs` | `ce63ad887ba744e6b648a98b138e87266993036d2b2cd458f1461df50e4ce745` | `4340a3990c23cf234b87e2a39ff7d13be9694c38f123bbc062310251d6b7bc06` |
| `zircon_editor/src/core/commands/defaults.rs` | `05f8dec1fa0a9fcca8d56d74ded64edb8df20980e554f4930b866f2245dfcaa9` | `2699f5c541351d5a730b818f666ccf784d4de7537bcce3d5f82a321d2538e350` |
| `zircon_editor/src/core/commands/registry.rs` | `f246609154d82c67a5d233962d737cc285a47d02a1d309949a8cb64cfa639c64` | `5a6242f49a4bcba897112cd86eb6992f10c4e67e6e960b4a84952210b6440c6b` |
| `zircon_editor/src/core/commands/registry/tests/cases.rs` | `7fa2dc4d9039b0286e412b9e4b52358e070f43976298ddb0b8681ddfd1b9a288` | `422e738cb72772cd3d9ff9a964c07e83c9df3633138e70cc19fa88b72dfb5075` |

All four files are tracked and their reviewed raw hashes match the frozen evidence. Rustfmt and the named-base `git diff --check` passed. The source-structure check reports 16 registry cases with all 11 previous bodies preserved. These checks and the guard are static evidence only; managed native, product, UI, performance, and MVP acceptance remain pending.
