---
status: in_progress
---

# Runtime Plugin Validation Worklist

## Native plugin validation obligations

Native discovery/load commands belong to plugin::native::discovery and stable live-host handles to plugin::native::host. NativePlugin is not a plugin-root loader alias. NativePluginAbiV1 / DESCRIPTOR_SYMBOL_V1 and V2 descriptors, callbacks and compatibility shims must not return; the V3 byte transport and V4 behavior/registration domains remain distinct. Unknown ABI rejection, panic containment and fallback lifecycle failure reporting stay observable through ZIRCON_NATIVE_PLUGIN_STATUS.

RuntimePluginDescriptor uses 15 private fields/accessors and RuntimePluginDescriptorBuilder. RuntimePluginDescriptor::new( is retired; F8 public-field/constructor completion must not be rewritten as public-field pending. Broader Cargo/native validation remains open. Keep extension_registry per-World retirement/state/concurrency tests independently mounted; PERF-MVP-532/533 generation state and quiescence are not closed by a test-owner split.

Availability separates membership/construction from evaluation.rs and selection.rs. System access separates parse/compile, authority.rs and typed error chains; package/module constructors keep the same serialized contract. No duplicate registry, cache or facade is introduced.

The six NativePluginProjectActivation types below belong to the existing live-host runtime group (loader/live/loading.rs, mounted through loader/live.rs and loader/mod.rs then plugin/native.rs). Required/optional selection, staging, bridge registration, commit/rollback and cleanup receipts do not create another ABI or a seventh group.

### Retained inventory and validation state

The latest explicitly reviewed source inventory is the 92-export/five-App-callsite contract below; earlier 64/68/74/86 counts are dated source snapshots, not competing current targets. Native implementation remains behind zircon_runtime::plugin::native. last_refined: 2026-08-27 preserves the worklist's declared refinement, not this preparation's validation date.

plugin_surface_lifecycle_boundary and native_plugin_public_surface retain exact metric/negative gates. VM scanning reads zircon_runtime/src/script/vm/tests/cases.rs; native tests read zircon_runtime/src/tests/plugins. script::vm, vampire_project_session, native loading, App/plugins, activation/rollback and real-backend execution still require separate terminal evidence.

This worklist retains its declared status independently of module policy, the runtime index and archive evidence. No new test/product acceptance is recorded.

### Retained source status identities

| Contract | Responsibility / evidence boundary |
|---|---|
| `f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred` | Retained dated source identity; independent validation remains pending. |
| `runtime_06_15_native_system_access_owner_split_static_passed_cargo_profile_deferred` | Parse/compile, capability authorization and typed errors retain separate owners. |
| `runtime_06_15_plugin_availability_evaluation_selection_owner_split_static_passed_cargo_deferred` | Membership, evaluation/reporting and profile/manifest selection remain separate. |
| `runtime_06_15_plugin_manifest_constructor_owner_split_static_passed_cargo_deferred` | Package/module owners preserve original types, defaults and serialization. |
| `runtime_15_runtime_plugin_descriptor_status_mirror_cleanup_static_passed_cargo_deferred` | Independent status mirrors retain their source/document responsibilities. |
| `runtime_plugin_descriptor_public_constructor_retired_coremin_check_passed` | Retained dated source identity; independent validation remains pending. |
| `runtime_plugin_descriptor_public_field_convergence_coremin_check_passed` | Retained dated source identity; independent validation remains pending. |

### Source and guard ownership

The finite source, guard and provenance inventory for these responsibilities is:

- `src/tests/plugin_surface_lifecycle/native_loader_namespace.rs`, `zircon_app/src/entry/product_composition/composition.rs`, `zircon_app/src/entry/product_composition/ownership.rs`.
- `zircon_app/src/entry/product_composition/request.rs`, `zircon_app/src/entry/tests/profile_bootstrap.rs`, `zircon_app/src/entry/export_bootstrap.rs`.
- `plugin_surface_lifecycle/split_layout.rs`, `plugin_surface_lifecycle/support.rs`, `src/plugin/mod.rs`.

### Validation interfaces

Keep these existing validation and domain interfaces in the gate:

- `runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts`, `review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors`, `runtime_06_native_loader_tests_use_isolated_plugin_native_namespace`, `review_f8_runtime_plugin_descriptor_public_constructor_is_retired`.
- `runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed`, `zr_vm_language`.

### Recorded targets

These retained targets belong to the original source snapshots, not a new measurement. The source dates/identities below keep distinct inventories separate.

- `public native namespace re-export locations 2/2`; source 1e4040515c2d:L325.
- `native loader V1/V2 implementation files 0/0`; source 1e4040515c2d:L325.
- `RuntimePluginDescriptor private fields 15/15`; source 2026-06-22:L51.
- `root public native re-export locations 0/0`; source 2026-07-01:L49.
- `unclassified native namespace symbols 0/0`; source 2026-07-01:L49.
- `native test namespace import files 3/3`; source 2026-07-01:L49.
- `zircon_plugins` V1/V2 usage files 0/0; source 1e4040515c2d:L325.
- `unclassified native root symbols 0/0`; source 2026-07-01:L49.
- `native_namespace_reexport_count = 92`; source 1e4040515c2d:L325.
- `fallback lifecycle failure tests 4/4`; source 1e4040515c2d:L325.
- `native namespace symbol groups 6/6`; source 2026-08-24:L56.
- `native test root import leaks 0/0`; source 2026-07-01:L49.
- `export_build_plan V1/V2 usage 0/0`; source 1e4040515c2d:L325.
- `native namespace re-export 92/92`; source 1e4040515c2d:L325.
- `expected_source_file_count = 20`; source 2026-08-24:L56.
- M4 gate `classified-and-clear`; source 2026-06-21:L48.
- `native loader test files 4/4`; source 2026-07-01:L49.
- `expected_doc_file_count = 5`; source 2026-08-24:L56.
- `native root re-export 0/0`; source 2026-07-01:L49.
- `root_reexport_count = 0`; source 2026-07-01:L49.
- `debt groups 0/0`; source 2026-07-01:L49.

### Contract details

The following original contract labels remain in scope; they do not establish new validation:

- `review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending`, `Runtime 15 F8 RuntimePluginDescriptor status mirror cleanup`, `RuntimePluginDescriptor public-field convergence complete`, `F8 RuntimePluginDescriptor public constructor retired`.
- `F8 RuntimePluginDescriptor public-field convergence`, `NativePluginProjectActivationSelectionStatus`, `NativePluginProjectActivationSelectionResult`, `app NativePlugin current call-site files: 5`.
- `NativePluginProjectActivationCleanupReceipt`, `RuntimePluginDescriptor::new retired`, `NativePluginProjectActivationRequest`, `NativePluginProjectActivationResult`.
- `mirror_docs_guard_present = true`, `hot reload failure injection`, `unknown ABI rejection`, `review findings`.
- `Runtime 06 plan`.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `06-plugin-surface-and-lifecycle.md`, SHA-256 `1e4040515c2df88fa15ea10c56eb15ee9ec9b421f4a3e4691101ff8e9cbcc48c`, selected source lines 37-327.
