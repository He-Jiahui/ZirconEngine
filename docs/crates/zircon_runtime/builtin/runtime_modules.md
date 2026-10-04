---
related_code:
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/compiled_plan.rs
  - zircon_runtime/src/builtin/runtime_modules/composition.rs
  - zircon_runtime/src/builtin/runtime_modules/composition/compiler.rs
  - zircon_runtime/src/builtin/runtime_modules/composition/identity.rs
  - zircon_runtime/src/builtin/runtime_modules/composition/outcome.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/feature_reports.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/profile_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_reports.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/availability.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/ids.rs
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/core/framework/platform/runtime_target_mode.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/diagnostics.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/report.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/descriptor_backed.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/composition_receipt.rs
  - zircon_runtime/src/dynamic_api/session/linked_plugins.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project/selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project.rs
  - zircon_runtime/src/plugin/runtime_profile/availability.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/mod.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/availability.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/mod.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/behavior.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
  - zircon_runtime/tests/frameworks_03_server_profile.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/support.rs
implementation_files:
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/compiled_plan.rs
  - zircon_runtime/src/builtin/runtime_modules/composition.rs
  - zircon_runtime/src/builtin/runtime_modules/composition/compiler.rs
  - zircon_runtime/src/builtin/runtime_modules/composition/identity.rs
  - zircon_runtime/src/builtin/runtime_modules/composition/outcome.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/feature_reports.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/profile_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_reports.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/availability.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/ids.rs
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/core/framework/platform/runtime_target_mode.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/diagnostics.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/report.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/descriptor_backed.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/composition_receipt.rs
  - zircon_runtime/src/dynamic_api/session/linked_plugins.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project/selection.rs
  - zircon_runtime/src/plugin/runtime_profile/availability.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - tools/tests/test_frameworks_03_server_feature_boundary.py
  - zircon_runtime/tests/frameworks_03_server_profile.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/entry_tree.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/availability.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/behavior.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d6_runtime_plugin_id.rs::review_d6_runtime_plugin_id_accepts_external_string_keys
  - rustfmt --edition 2021 zircon_runtime/src/builtin/runtime_modules.rs zircon_runtime/src/builtin/runtime_modules/*.rs zircon_runtime/src/builtin/runtime_modules/tests/*.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/builtin/runtime_modules/assembly.rs zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs zircon_runtime/src/builtin/runtime_modules/assembly/feature_reports.rs zircon_runtime/src/builtin/runtime_modules/assembly/profile_modules.rs zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs zircon_runtime/src/builtin/runtime_modules/assembly/registration_reports.rs zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs zircon_runtime/src/builtin/runtime_modules/tests/registration/mod.rs zircon_runtime/src/builtin/runtime_modules/tests/registration/behavior.rs zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/builtin/runtime_modules/plugin_modules.rs zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs zircon_runtime/src/plugin/runtime_profile/availability.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/builtin/runtime_modules/ids.rs zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs zircon_runtime/src/core/framework/platform/runtime_target_mode.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/builtin/runtime_modules/load_report.rs zircon_runtime/src/builtin/runtime_modules/load_report/diagnostics.rs zircon_runtime/src/builtin/runtime_modules/load_report/report.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs: passed 2026-06-23 for Plugins 12 M3/T1 importer family id stopgap
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_data_runtime -p zircon_plugin_asset_importer_model_runtime -p zircon_plugin_asset_importer_shader_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-importer-m3-0622 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warnings
  - rustfmt --edition 2021 --check zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs: passed 2026-06-23 for Plugins 12 M3/T1 split importer Opus id stopgap
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_obj_importer_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_audio_importer_runtime -p zircon_plugin_opus_importer_runtime -p zircon_plugin_shader_wgsl_importer_runtime -p zircon_plugin_ui_document_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-split-importer-m3-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warnings
  - rustfmt --edition 2021 zircon_runtime\src\builtin\runtime_modules\ids\plugin_id.rs zircon_runtime\src\builtin\runtime_modules\plugin_modules\loader.rs zircon_runtime\src\builtin\runtime_modules\tests\registration\structure.rs zircon_runtime\src\tests\plugin_extensions\plugin_workspace_shape.rs: passed 2026-06-23 for Plugins 12/13 M5 RuntimePluginId string-newtype
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-check --message-format short --color never: passed 2026-06-23 with existing warning noise
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-sdk-check --message-format short --color never: passed 2026-06-23 with existing warning noise
  - cargo test -p zircon_runtime --lib --no-default-features --features core-min runtime_plugin_id_accepts_external_keys_without_core_variant --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-check --message-format short --color never -- --test-threads=1 --nocapture: blocked before running by unrelated runtime lib-test compile drift; not counted as passed
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs::runtime_module_assembly_keeps_specialized_flows_in_child_owners
  - folder-backed runtime module registration test guard over zircon_runtime/src/builtin/runtime_modules/tests/registration/{mod.rs,behavior.rs,structure.rs}
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-modules-split-0604 --message-format short --color never
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-linked-feature-provider-manifest-0607 --message-format short --color never
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
doc_type: module-detail
---

# Runtime Module Assembly

## Purpose

`zircon_runtime::builtin::runtime_modules` owns runtime module assembly for target modes and runtime profiles. It converts either a target/profile plus project plugin inputs or one frozen `CompiledProjectPluginPlan` into `Result<RuntimeModuleCompositionPlan, RuntimeModuleCompositionRejection>`. A ready plan privately owns the final activation-order module and descriptor graph, structured runtime plugin availability, non-fatal diagnostics, and a deterministic composition identity. A rejection owns availability and fatal diagnostics but has no module field or accessor. Warning/fatal strings are projections created only when a host or ABI boundary must display them.

This boundary is runtime-owned. `zircon_app` calls the public assembly functions through `zircon_runtime::builtin::{...}`, but optional plugin implementation fan-out now sits behind the plugin-workspace `zircon_first_party_runtime_catalog` rather than process entry code. The `zircon_runtime` crate root deliberately does not re-export the assembly functions.

## Module Layout

- `runtime_modules.rs` is the facade. It declares child modules and re-exports the stable public API.
- `ids.rs` is the runtime plugin-id boundary. It declares only the `plugin_id` child and re-exports `RuntimePluginId`.
- `ids/plugin_id.rs` owns `RuntimePluginId`, including string-newtype storage, built-in associated constants, key validation, label, parse, and serde behavior.
- `core/framework/platform/runtime_target_mode.rs` owns the neutral `RuntimeTargetMode` contract consumed by module assembly, platform capability policy, plugin manifests, hosts, and external plugins.
- `composition.rs` is the public result boundary. It re-exports the compiler, immutable identity, ready plan, rejection, and result alias.
- `composition/compiler.rs` owns the frozen-plan vertical compiler and admits host modules before final validation.
- `composition/identity.rs` hashes catalog generation, source-manifest fingerprint, target/profile identity, and the full logical module/service descriptor graph.
- `composition/outcome.rs` is the only candidate finalizer. It freezes the Core module/service graph, materializes activation order, and produces either a ready plan or a module-free rejection.
- `load_report.rs` is the private mutable-candidate boundary. It re-exports only `RuntimeModuleLoadDiagnostic` publicly.
- `load_report/report.rs` owns the crate-private `RuntimeModuleLoadReport`, construction, availability replacement, and typed diagnostic storage.
- `load_report/diagnostics.rs` owns `RuntimeModuleLoadDiagnostic`, diagnostic severity, display-message projection, required-provider summaries, and fatal-diagnostic detection.
- `core_modules.rs` owns built-in core module vector construction for target modes and minimal profiles.
- `manifest.rs` owns default target manifests, profile manifests, and manifest baseline overlay behavior.
- `availability.rs` owns structured runtime plugin availability reports for profiles, targets, manifests, registration reports, and frozen plan provider rows.
- `plugin_modules.rs` is the private plugin-module boundary; it declares the built-in loader and descriptor-backed module children and re-exports only the narrow constructors consumed by assembly.
- `plugin_modules/descriptor_backed.rs` owns the `EngineModule` adapter for a module descriptor already selected by a frozen plan.
- `plugin_modules/loader.rs` owns concrete built-in plugin module loading.
- `assembly.rs` owns the stable public facade functions and delegates specialized target/profile/report assembly to private child owners.
- `assembly/compiled_plan.rs` owns plan-only candidate materialization. It consumes the plan's completed manifest, exact provider packaging rows, frozen extension registry, target-filtered module proposals, and catalog diagnostics; the composition finalizer performs the one authoritative final graph freeze after host modules have been added.
- `assembly/extension_inputs.rs` owns plugin extension-registry traversal for asset importer registries, render feature descriptors, render pass executors, runtime prepare collectors, and runtime provider registrations.
- `assembly/feature_reports.rs` owns runtime plugin catalog construction for feature dependency reports, active feature registration filtering, and blocked-feature diagnostic projection into module load reports.
- `assembly/profile_modules.rs` owns runtime-profile assembly flow, minimal-profile module construction, profile manifest lookup, and profile availability replacement.
- `assembly/registration_inputs.rs` owns the `RuntimeModuleRegistrationInputs` data object, linked-plugin id collection from active registration reports, and projection of one frozen extension registry into the built-in extension inputs.
- `assembly/registration_reports.rs` owns target/profile registration-report assembly flow, active plugin report filtering, asset-importer error projection, and registration-report availability updates.
- `assembly/target_modules.rs` owns target/manifest module-list construction, structured provider-availability consumption, built-in plugin selection, unknown-plugin diagnostics, and module ordering.
- `tests/` mirrors the behavior split: manifest baseline behavior, availability reporting, folder-backed registration/bootstrap behavior, and shared fixtures.
- `tests/registration/mod.rs` is wiring-only for registration/bootstrap tests.
- `tests/registration/behavior.rs` owns registration/bootstrap behavior assertions.
- `tests/registration/structure.rs` owns runtime-module source-shape guards.

## Architecture Notes

The split follows the M2 runtime module assembly decision in the runtime architecture review plan. It keeps Bevy-style profile/plugin composition in one runtime-owned facade, follows Fyrox-style Rust subsystem modules for runtime implementation details, and preserves Unreal-style separation between runtime assembly, plugin implementation domains, and editor/process hosts.

The legacy target/profile/report functions remain typed input adapters, but every public adapter now returns the same ready/rejected result. App entry and dynamic session each compile one `Arc<CompiledProjectPluginPlan>` and reuse its extension report. App contributes Dev diagnostics and Editor host modules to `RuntimeModuleCompositionCompiler` before finalization, then constructs its compatibility `ResolvedPluginGroup` directly from the already-frozen module/descriptor order without another profile assembly or topology sort. Dynamic selects Navigation/Animation fallback modules before compilation, never appends modules in session construction, and registers the plan's frozen descriptors instead of rematerializing them from `EngineModule`. The dynamic session retains the plugin plan for its lifetime.

`CompiledProjectPluginPlan` retains the source-manifest fingerprint used by the catalog cache. `RuntimeModuleCompositionIdentity` combines it with catalog generation, target/profile identity, and a BLAKE3 digest over the final logical descriptor graph. Factory and lifecycle pointers are intentionally excluded; module/service identity, authored order, startup modes, dependencies, descriptions, and init levels are included. Legacy target/profile adapters expose `None` for catalog generation and source fingerprint rather than manufacturing provenance. A dynamic session retains this exact identity and exposes a typed `ZrRuntimeModuleCompositionReceiptV1` projection through the existing profile-control JSON slot; the App consumes that receipt after session creation and passes it into the matching Editor gateway generation instead of assembling a second catalog or module graph.

The follow-up M2 provider slice moved linked first-party registration into `zircon_first_party_runtime_catalog`. The runtime assembly facade still consumes registration reports and stays independent of concrete plugin crates; the app wrapper only projects config and render-profile selections before delegating provider lookup to the catalog.

The 2026-06-07 M2 registration-input split moved extension registry traversal out of `assembly.rs`. Profile and target assembly now call a private registration-input owner for active plugin reports, linked provider ids, asset importer registration, render feature descriptors, render pass executors, runtime prepare collectors, and runtime provider registrations. This keeps `assembly.rs` as the public facade boundary while preventing it from regrowing plugin extension aggregation behavior.

The same M2 assembly pass moved target/manifest module selection into `assembly/target_modules.rs`. That child owns `ProjectPluginManifest::enabled_for_target(...)` traversal, linked-provider availability, built-in versus externalized plugin module handoff, and required-missing detection. `assembly.rs` now composes profile, manifest, plugin registration, and feature dependency flow without owning the target module-selection loop.

The 2026-06-07 feature-report split moved `RuntimePluginCatalog::from_registration_reports(...)`, `feature_dependency_report(...)`, blocked-feature warning/error projection, and active feature registration filtering into `assembly/feature_reports.rs`. `assembly.rs` still decides which public API path is being served, but it no longer owns feature dependency report construction or blocked-feature diagnostic loops.

The same assembly pass moved registration-report target/profile flow into `assembly/registration_reports.rs`. That child now owns active plugin report filtering, plugin/feature registration input handoff, asset-importer error projection, target/profile availability replacement for registration reports, and delegation to the feature-report and target-module owners. `assembly.rs` keeps the stable public API surface but no longer owns report-backed availability or asset-importer diagnostic loops.

The 2026-06-07 profile split moved runtime-profile assembly into `assembly/profile_modules.rs`. Minimal profile module construction, `RuntimeProfileDescriptor::for_id(...)`, profile manifest lookup, empty registration-input target assembly, and profile availability replacement now live behind that child owner. The public facade keeps the stable profile API functions, but no longer owns profile-specific branching or minimal-profile assembly details.

The same registration-input pass split extension registry traversal into `assembly/extension_inputs.rs`. `registration_inputs.rs` now owns the data shape and linked-plugin id materialization, while `extension_inputs.rs` owns `RuntimeExtensionRegistry` traversal, asset importer aggregation, render feature descriptors, render pass executors, runtime prepare collectors, and runtime provider registration collection.

The same hard-cut pass removed the stale private `extensions.rs` owner and then repaired the remaining child-owner import that still reached back to it. Asset importer aggregation and the rest of extension registry traversal now have a single owner at `assembly/extension_inputs.rs`.

The 2026-06-07 plugin-module split converted `plugin_modules.rs` into a private boundary file. The 2026-07-13 provider-identity hard cut then deleted `plugin_modules/availability.rs`: target assembly now consumes the already-built `RuntimePluginAvailabilityReport` instead of maintaining a second linked-provider predicate. `plugin_modules/loader.rs` remains the concrete optional UI module loader, and the boundary no longer manufactures either a second provider truth or a warning-string channel.

The plugin-module helper remains internal to `zircon_runtime::builtin::runtime_modules` instead of crate-public. `plugin_modules.rs` re-exports only built-in plugin module loading to `assembly/target_modules.rs`; provider availability is obtained from the structured availability owner rather than routed through this facade.

The same M2 id split first converted `ids.rs` into a structural boundary. The 2026-07-13 Frameworks05 hard cut then removed the misplaced `ids/target_mode.rs` child and the `zircon_runtime::builtin::RuntimeTargetMode` facade projection. `RuntimePluginId` remains assembly-owned at `zircon_runtime::builtin::RuntimePluginId`; the target-mode declaration now has one neutral owner and public path at `zircon_runtime::core::framework::platform::RuntimeTargetMode`.

The 2026-06-23 Plugins 12 M3/T1 importer-family cutover first added temporary runtime-plugin ids for `AssetImporterData`, `AssetImporterModel`, `AssetImporterShader`, and `OpusImporter` so the importer packages could finish their trait-backed registration entry migration. The later Plugins 12/13 M5 string-newtype cutover keeps those built-in ids as associated constants but removes the need for future core enum branches: `RuntimePluginId::parse_key(...)` now accepts any syntactically valid external package key, `RuntimePluginId::new(...)` provides the infallible construction path for already-validated static ids, and serialization stays a plain string.

Runtime 15 M3 D6 RuntimePluginId open string-newtype review sync: status `d6_runtime_plugin_id_open_string_newtype_review_static_passed_cargo_deferred` is locked by `tests/runtime_absorption/code_review_findings/plugin_importer_dx/d6_runtime_plugin_id.rs::review_d6_runtime_plugin_id_accepts_external_string_keys`. The guard ties this module doc to `RuntimePluginId`, `runtime_plugin_id_accepts_external_keys_without_core_variant`, the D6 review row, Runtime 15 status output, and the rule that third-party legal keys do not need a core enum branch.

The same M5 cutover keeps `plugin_modules/loader.rs` as the concrete built-in handoff owner. The optional UI module remains the only plugin module loaded there. Known external providers are represented by catalog registrations and structured availability; unknown-but-valid third-party ids become typed `UnknownPlugin` diagnostics in target assembly rather than string fallbacks or new engine-core `match` arms.

The 2026-07-13 Frameworks 02 hard cut removed the parallel `warnings`, `errors`, private required-missing storage, `RuntimeRequiredPluginMissing`, and all `effective_*` merge helpers. The 2026-08-27 composition cut then made that mutable report crate-private. Known required-provider absence has exactly one owner at `RuntimePluginAvailabilityReport::missing_required`; optional unavailable-provider warnings are projected from the non-required availability categories, while core, unknown-plugin, feature, and asset-importer failures stay typed. Fatal candidates can only become `RuntimeModuleCompositionRejection`, which cannot expose a submit-ready module vector.

The same single-source cut makes compile-time built-in availability explicit. When the runtime is compiled without the `ui` feature, a selected UI plugin is classified in structured availability with the reason that the built-in UI runtime is disabled; required UI also enters `missing_required`. The loader does not recreate an independent `"ui feature is disabled"` warning or required-missing list.

Provider identity is exact package identity. Registration-report availability materializes only `RuntimePluginRegistrationReport::package_manifest.id`; `project_selection.id`, normalized `RuntimePluginId::key()`, case variants, and aliases such as `audio`/`network` are not provider substitutes. Both linked and native-dynamic matching compare that package id against `RuntimePluginDescriptor::package_id()`, and target module loading consumes the resulting `Linked`/`NativeDynamic` category. A package with the same runtime domain but a different package id therefore remains unavailable and, when required, remains in `missing_required`.

The same runtime-module test cutover converted the former flat `tests/registration.rs` owner into `tests/registration/`. `tests/registration/mod.rs` now only declares behavior and structure child owners, `behavior.rs` keeps runtime module registration/bootstrap behavior coverage, and `structure.rs` keeps the source-shape guard that prevents assembly, id, load-report, plugin-module, and registration-input logic from drifting back into root or facade files.

The 2026-06-07 root-facade cutover made `zircon_runtime::builtin` the direct public owner for assembly helpers. The 2026-08-27 hard cut deleted `builtin_runtime_modules()` because it discarded diagnostics by extracting a raw module vector; callers use typed target/profile/compiler results. The crate root exposes the `builtin` module but does not forward assembly functions or runtime module DTOs.

The 2026-06-17 stale root DTO consumer cleanup remains historically valid for the crate-root surface: neither DTO is flattened at `zircon_runtime`. The later Frameworks05 cut strengthens domain ownership without adding compatibility paths. Runtime plugin ids still import from `builtin`; target/profile/platform/project-manifest consumers now import `core::framework::platform::RuntimeTargetMode` directly. The deleted builtin target-mode file and re-export must not return.

The same M2 hard-cut pass tightened internal id/report ownership. After the Frameworks05 target-mode move, implementation files under `runtime_modules` import `RuntimePluginId` from `ids`, `RuntimeTargetMode` from `core/framework/platform`, and `RuntimeModuleLoadReport` from `load_report` instead of depending on parent facade re-export wiring. Public callers use each direct domain owner; crate-root or builtin target-mode forwarding is not kept as a compatibility path.

## Invariants

- Root `runtime_modules.rs` must stay structural: child module declarations, curated re-exports, and test module wiring only.
- `zircon_runtime::builtin` is the public namespace for runtime module assembly helpers, `RuntimePluginId`, and typed composition outcomes. `RuntimeTargetMode` belongs to `zircon_runtime::core::framework::platform`. The `zircon_runtime` crate root must not forward any of these functions or DTOs, and builtin must not re-export the platform contract.
- A rejected composition must not contain or expose modules. Only the finalizer may construct `RuntimeModuleCompositionPlan`, and it must freeze both module and service dependencies before construction.
- App and Dynamic host modules must enter `RuntimeModuleCompositionCompiler` before final graph freeze. Product callers must not append a module or run a second profile/group topology sort after receiving a ready plan.
- A dynamic product session must retain and expose the identity of that frozen graph. Its cross-ABI receipt must distinguish module-selection profile from dynamic-session policy, preserve compiled catalog provenance, and must not manufacture provenance for legacy composition paths.
- Internal implementation files below `runtime_modules` must import DTOs from their direct owners: `ids::RuntimePluginId`, `core::framework::platform::RuntimeTargetMode`, and `load_report::RuntimeModuleLoadReport`. They must not route sibling implementation dependencies through the parent facade exports in `runtime_modules.rs`.
- The assembly facade may expose target/profile/plugin registration entry points, but runtime plugin-id re-export wiring belongs in `ids.rs`, plugin identity parsing belongs in `ids/plugin_id.rs`, target-mode declaration belongs in `core/framework/platform/runtime_target_mode.rs`, load-report re-export wiring belongs in `load_report.rs`, load-report data storage belongs in `load_report/report.rs`, typed diagnostic classification and display projection belong in `load_report/diagnostics.rs`, structured availability and exact package-provider matching belong in `availability.rs`, concrete built-in plugin loading belongs in `plugin_modules/loader.rs`, extension-registry traversal belongs in `assembly/extension_inputs.rs`, feature dependency report handling belongs in `assembly/feature_reports.rs`, profile assembly flow belongs in `assembly/profile_modules.rs`, registration-report assembly flow belongs in `assembly/registration_reports.rs`, registration-input data assembly belongs in `assembly/registration_inputs.rs`, target/manifest module selection and availability-category consumption belong in `assembly/target_modules.rs`, manifest defaults belong in `manifest.rs`, and concrete built-in module vector construction belongs in `core_modules.rs`.
- The only built-in plugin module loaded from this boundary remains the optional UI module behind `ui`; other runtime plugin implementations remain externalized to `zircon_plugins/*`.
- `RuntimePluginId` is an open string-newtype. First-party built-ins may add associated constants for ergonomics, but external plugin ids must not require editing engine core unless they need genuinely new built-in loader behavior.
- Generated export code must consume this facade or runtime/plugin catalog APIs; it must not duplicate profile assembly, required-provider availability, diagnostic severity mapping, plugin-domain mapping, or linked-provider crate fan-out.
- Render extension inputs exist only when the `graphics` feature is compiled. A `target-server` build constructs the core module vector without render descriptors/providers and without `ScriptModule`; it must not retain placeholder graphics/UI slots or runtime target checks as compatibility behavior.

## Validation

Frameworks 03 M1 server hard cutover passed WSL nightly checks for both `--no-default-features --features target-server` and the default feature set. The server dependency-tree gate found no wgpu, winit, taffy, glyphon, naga, swash, fontsdf, or woff2-patched packages. A later support-first review found that a default/client-compiled binary still selected `ScriptModule` for `ServerRuntime`; both graphics and non-graphics core-module assembly paths now exclude Script by target as well as compile feature. `frameworks_03_server_profile` passes 1/1 under the full default feature set (test 0.01s, cold command 26m52s), the fresh target-server lib check passes in 8m36s, and the server static guard passes 5/5. The broader per-domain and full-test matrix remains tracked by the Frameworks 03 plan.

An earlier implementation slice ran focused `zircon_runtime` checking after formatting. For the 2026-08-27 composition-outcome cut, only direct owned-file `rustfmt` parsing and static boundary review are recorded; managed Cargo, behavioral tests, performance, and power validation remain pending. No green result from the earlier slice is carried forward to this cut.

The 2026-07-13 provider-identity follow-up passed a Windows `core-min` build-and-run probe with `--no-default-features`: linked and native provider inputs containing only `RuntimePluginId::key()` did not satisfy a descriptor with a different package id; linked and native registration reports whose `project_selection.id` matched the runtime domain but whose `package_manifest.id` differed also remained unavailable; Client2d and Client3d each reported UI as their sole required missing plugin when `ui` was not compiled. The same probe exited 0 after 16m11s in a coordinator-owned disposable lane. A default-feature `cargo check -p zircon_runtime --lib --tests --locked --jobs 1` first completed the production library with existing warnings, then stopped in the unrelated integration target `material_shader_redirect_dependency_contract` because that target still calls missing `MaterialAsset::from_toml_str` and passes two `&AssetReference` values where owned values are required. No full test-profile green result is claimed from that command.

The 2026-06-07 plugin-module visibility repair was validated through the Sound linked-feature provider manifest lane because that path compiles `zircon_runtime` before the Sound optional-feature tests. `rustfmt --edition 2021 --check` passed over `plugin_modules/availability.rs`, `plugin_modules/loader.rs`, `assembly/extension_inputs.rs`, and the touched Sound parity test. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-linked-feature-provider-manifest-0607 --message-format short --color never` passed with existing `zircon_runtime` warnings only after the helper visibility repair. The follow-up Sound `optional_feature_manifest` test run passed with 4 tests, 0 failures, and 177 filtered tests, so the lower shared runtime split compiled under the linked-provider feature manifest, registration-report, and runtime-module descriptor paths.

The 2026-06-07 root-facade cutover was validated with scoped formatting and source guards over the touched Rust files. `rustfmt --edition 2021 --check --config skip_children=true` passed for the runtime root, prelude, app entry, dynamic API session, runtime absorption tests, plugin-extension tests, and app source guard files changed by the cutover. Live Rust, docs, and plan scans found no remaining crate-root or `crate::` assembly-function paths for the builtin module helpers. The root-facade static guard confirmed `zircon_runtime/src/lib.rs` exposes `pub mod builtin;` but does not contain the flattened assembly helper names. `git diff --check` passed for the touched cutover paths with expected LF-to-CRLF warnings only. A focused `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-builtin-root-facade-0607 --message-format short --color never` reached compilation but failed in the active WGPU render lane at `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs:132` because `create_mesh_draw(...)` is currently called with 21 arguments against a 20-argument signature; that graphics file is owned by the concurrent WGPU render session and is not part of this root-facade cutover.

The 2026-06-17 stale root DTO consumer cleanup was validated indirectly through the editor UI M3.S2 Material divider slice. That historical check removed crate-root imports without flattening the DTOs. The 2026-07-13 Frameworks05 follow-up supersedes its target-mode owner conclusion: current source scans require `RuntimePluginId` from builtin and `RuntimeTargetMode` from `core::framework::platform`, while both remain absent from the crate root and runtime prelude.

The 2026-06-23 Plugins 12/13 M5 string-newtype cutover was validated with scoped formatting over `ids/plugin_id.rs`, `plugin_modules/loader.rs`, the runtime-module structure guard, and the plugin workspace shape guard. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-check --message-format short --color never` passed with existing warning noise, and `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-sdk-check --message-format short --color never` also passed. The focused runtime lib-test `runtime_plugin_id_accepts_external_keys_without_core_variant` is present, but the current runtime lib-test lane is blocked before running by unrelated missing test child modules, private glTF fixture re-exports, and existing WGPU test API drift, so no focused runtime test pass is claimed for this slice.

The 2026-06-07 registration test owner split was validated with `rustfmt --edition 2021 --check zircon_runtime\src\builtin\runtime_modules\tests\registration\mod.rs zircon_runtime\src\builtin\runtime_modules\tests\registration\behavior.rs zircon_runtime\src\builtin\runtime_modules\tests\registration\structure.rs`, which passed after formatter wrapping in `structure.rs`. The focused folder-backed registration test guard passed for deleted flat `tests/registration.rs`, parent module wiring, wiring-only `registration/mod.rs`, behavior assertions in `registration/behavior.rs`, source-shape assertions in `registration/structure.rs`, and docs/session coverage. `audit_runtime_structure.py --json` also completed with no plugin runtime gaps, no unclassified public runtime modules, and no unclassified large-file hotspots. No fresh Cargo result is claimed for this slice because it only moved Rust unit-test ownership and updated documentation metadata.
