---
related_code:
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
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
  - zircon_runtime/src/builtin/runtime_modules/ids/target_mode.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/diagnostics.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/missing.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/report.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/availability.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
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
  - zircon_runtime/src/builtin/runtime_modules/ids/target_mode.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/diagnostics.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/missing.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/report.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/availability.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
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
  - rustfmt --edition 2021 --check zircon_runtime/src/builtin/runtime_modules/plugin_modules/availability.rs zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/builtin/runtime_modules/ids.rs zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs zircon_runtime/src/builtin/runtime_modules/ids/target_mode.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/builtin/runtime_modules/load_report.rs zircon_runtime/src/builtin/runtime_modules/load_report/diagnostics.rs zircon_runtime/src/builtin/runtime_modules/load_report/missing.rs zircon_runtime/src/builtin/runtime_modules/load_report/report.rs
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

`zircon_runtime::builtin::runtime_modules` owns runtime module assembly for target modes and runtime profiles. It converts a target/profile plus project plugin manifest and registration reports into a `RuntimeModuleLoadReport` containing built-in engine modules, warnings, fatal diagnostics, and structured runtime plugin availability.

This boundary is runtime-owned. `zircon_app` calls the public assembly functions through `zircon_runtime::builtin::{...}`, but optional plugin implementation fan-out now sits behind the plugin-workspace `zircon_first_party_runtime_catalog` rather than process entry code. The `zircon_runtime` crate root deliberately does not re-export the assembly functions.

## Module Layout

- `runtime_modules.rs` is the facade. It declares child modules and re-exports the stable public API.
- `ids.rs` is the public id boundary. It declares the `plugin_id` and `target_mode` children and re-exports the stable runtime module id API.
- `ids/plugin_id.rs` owns `RuntimePluginId`, including string-newtype storage, built-in associated constants, key validation, label, parse, and serde behavior.
- `ids/target_mode.rs` owns `RuntimeTargetMode`.
- `load_report.rs` is the public load-report boundary. It declares the report children and re-exports the stable report API.
- `load_report/report.rs` owns `RuntimeModuleLoadReport`, construction, availability replacement, and private missing-provider storage.
- `load_report/missing.rs` owns `RuntimeRequiredPluginMissing`.
- `load_report/diagnostics.rs` owns effective missing-provider aggregation, missing-provider summaries, effective errors, and fatal-diagnostic detection.
- `core_modules.rs` owns built-in core module vector construction for target modes and minimal profiles.
- `manifest.rs` owns default target manifests, profile manifests, and manifest baseline overlay behavior.
- `availability.rs` owns structured runtime plugin availability reports for profiles, targets, manifests, and registration reports.
- `plugin_modules.rs` is the private plugin-module boundary; it declares the availability and loader children and re-exports only the narrow helpers consumed by target assembly.
- `plugin_modules/availability.rs` owns linked-plugin availability checks and built-in runtime-domain availability diagnostics.
- `plugin_modules/loader.rs` owns built-in plugin module loading and externalized runtime-plugin warning messages.
- `assembly.rs` owns the stable public facade functions and delegates specialized target/profile/report assembly to private child owners.
- `assembly/extension_inputs.rs` owns plugin extension-registry traversal for asset importer registries, render feature descriptors, render pass executors, runtime prepare collectors, and runtime provider registrations.
- `assembly/feature_reports.rs` owns runtime plugin catalog construction for feature dependency reports, active feature registration filtering, and blocked-feature diagnostic projection into module load reports.
- `assembly/profile_modules.rs` owns runtime-profile assembly flow, minimal-profile module construction, profile manifest lookup, and profile availability replacement.
- `assembly/registration_inputs.rs` owns the `RuntimeModuleRegistrationInputs` data object, linked-plugin id collection from active registration reports, and handoff from report selections into extension-input aggregation.
- `assembly/registration_reports.rs` owns target/profile registration-report assembly flow, active plugin report filtering, asset-importer error projection, and registration-report availability updates.
- `assembly/target_modules.rs` owns target/manifest module-list construction, linked-provider checks, built-in/externalized plugin selection, and required-missing diagnostics.
- `tests/` mirrors the behavior split: manifest baseline behavior, availability reporting, folder-backed registration/bootstrap behavior, and shared fixtures.
- `tests/registration/mod.rs` is wiring-only for registration/bootstrap tests.
- `tests/registration/behavior.rs` owns registration/bootstrap behavior assertions.
- `tests/registration/structure.rs` owns runtime-module source-shape guards.

## Architecture Notes

The split follows the M2 runtime module assembly decision in the runtime architecture review plan. It keeps Bevy-style profile/plugin composition in one runtime-owned facade, follows Fyrox-style Rust subsystem modules for runtime implementation details, and preserves Unreal-style separation between runtime assembly, plugin implementation domains, and editor/process hosts.

The current slice is intentionally a structural cutover rather than a behavior rewrite. It preserves the existing public function names and report types while removing the previous monolithic file shape.

The follow-up M2 provider slice moved linked first-party registration into `zircon_first_party_runtime_catalog`. The runtime assembly facade still consumes registration reports and stays independent of concrete plugin crates; the app wrapper only projects config and render-profile selections before delegating provider lookup to the catalog.

The 2026-06-07 M2 registration-input split moved extension registry traversal out of `assembly.rs`. Profile and target assembly now call a private registration-input owner for active plugin reports, linked provider ids, asset importer registration, render feature descriptors, render pass executors, runtime prepare collectors, and runtime provider registrations. This keeps `assembly.rs` as the public facade boundary while preventing it from regrowing plugin extension aggregation behavior.

The same M2 assembly pass moved target/manifest module selection into `assembly/target_modules.rs`. That child owns `ProjectPluginManifest::enabled_for_target(...)` traversal, linked-provider availability, built-in versus externalized plugin module handoff, and required-missing detection. `assembly.rs` now composes profile, manifest, plugin registration, and feature dependency flow without owning the target module-selection loop.

The 2026-06-07 feature-report split moved `RuntimePluginCatalog::from_registration_reports(...)`, `feature_dependency_report(...)`, blocked-feature warning/error projection, and active feature registration filtering into `assembly/feature_reports.rs`. `assembly.rs` still decides which public API path is being served, but it no longer owns feature dependency report construction or blocked-feature diagnostic loops.

The same assembly pass moved registration-report target/profile flow into `assembly/registration_reports.rs`. That child now owns active plugin report filtering, plugin/feature registration input handoff, asset-importer error projection, target/profile availability replacement for registration reports, and delegation to the feature-report and target-module owners. `assembly.rs` keeps the stable public API surface but no longer owns report-backed availability or asset-importer diagnostic loops.

The 2026-06-07 profile split moved runtime-profile assembly into `assembly/profile_modules.rs`. Minimal profile module construction, `RuntimeProfileDescriptor::for_id(...)`, profile manifest lookup, empty registration-input target assembly, and profile availability replacement now live behind that child owner. The public facade keeps the stable profile API functions, but no longer owns profile-specific branching or minimal-profile assembly details.

The same registration-input pass split extension registry traversal into `assembly/extension_inputs.rs`. `registration_inputs.rs` now owns the data shape and linked-plugin id materialization, while `extension_inputs.rs` owns `RuntimeExtensionRegistry` traversal, asset importer aggregation, render feature descriptors, render pass executors, runtime prepare collectors, and runtime provider registration collection.

The same hard-cut pass removed the stale private `extensions.rs` owner and then repaired the remaining child-owner import that still reached back to it. Asset importer aggregation and the rest of extension registry traversal now have a single owner at `assembly/extension_inputs.rs`.

The 2026-06-07 plugin-module split converted `plugin_modules.rs` into a private boundary file. Linked-plugin availability checks and built-in-domain diagnostics now live in `plugin_modules/availability.rs`, while the concrete optional UI module loader and externalized plugin messages live in `plugin_modules/loader.rs`. This keeps target assembly dependent on the plugin-module owner without letting one file mix availability policy and module construction.

The 2026-06-07 visibility repair keeps those plugin-module helpers internal to `zircon_runtime::builtin::runtime_modules` instead of crate-public, but widens the child-owner functions enough for `plugin_modules.rs` to re-export them to sibling assembly owners. This preserves the split boundary while allowing `assembly/target_modules.rs` to consume linked-provider checks, built-in-domain messages, and plugin module loading through the private plugin-module facade.

The same M2 id split converted `ids.rs` into a structural boundary. `ids/plugin_id.rs` now owns runtime plugin identifiers plus string key/label/parse behavior, while `ids/target_mode.rs` owns target-mode declarations. The public DTO path is `zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode}` through the runtime module facade, and the implementation no longer mixes target-mode declarations with plugin id parsing.

The 2026-06-23 Plugins 12 M3/T1 importer-family cutover first added temporary runtime-plugin ids for `AssetImporterData`, `AssetImporterModel`, `AssetImporterShader`, and `OpusImporter` so the importer packages could finish their trait-backed registration entry migration. The later Plugins 12/13 M5 string-newtype cutover keeps those built-in ids as associated constants but removes the need for future core enum branches: `RuntimePluginId::parse_key(...)` now accepts any syntactically valid external package key, `RuntimePluginId::new(...)` provides the infallible construction path for already-validated static ids, and serialization stays a plain string.

Runtime 15 M3 D6 RuntimePluginId open string-newtype review sync: status `d6_runtime_plugin_id_open_string_newtype_review_static_passed_cargo_deferred` is locked by `tests/runtime_absorption/code_review_findings/plugin_importer_dx/d6_runtime_plugin_id.rs::review_d6_runtime_plugin_id_accepts_external_string_keys`. The guard ties this module doc to `RuntimePluginId`, `runtime_plugin_id_accepts_external_keys_without_core_variant`, the D6 review row, Runtime 15 status output, and the rule that third-party legal keys do not need a core enum branch.

The same M5 cutover keeps `plugin_modules/loader.rs` as the built-in-vs-externalized handoff owner. The optional UI module remains the only concrete built-in plugin module loaded here; all other known first-party plugin ids still produce their existing externalized diagnostics, and unknown-but-valid third-party ids now use the same externalized warning fallback instead of requiring a new `match` arm in engine core.

The same load-report split converted `load_report.rs` into a structural boundary. `load_report/report.rs` owns the report data shape and private required-missing storage, `load_report/missing.rs` owns the missing-provider DTO, and `load_report/diagnostics.rs` owns the effective diagnostic projection. Target assembly now records required missing providers through `RuntimeModuleLoadReport::push_required_missing(...)` instead of mutating report storage directly.

The same runtime-module test cutover converted the former flat `tests/registration.rs` owner into `tests/registration/`. `tests/registration/mod.rs` now only declares behavior and structure child owners, `behavior.rs` keeps runtime module registration/bootstrap behavior coverage, and `structure.rs` keeps the source-shape guard that prevents assembly, id, load-report, plugin-module, and registration-input logic from drifting back into root or facade files.

The 2026-06-07 root-facade cutover made `zircon_runtime::builtin` the direct public owner for assembly helpers such as `builtin_runtime_modules()` and `runtime_modules_for_target(...)`. The crate root exposes the `builtin` module but does not forward assembly functions or runtime module id/report DTOs. App entry, dynamic API startup, runtime plugin-extension tests, and structural guards import assembly helpers and DTOs through `zircon_runtime::builtin` or `crate::builtin`.

The 2026-06-17 stale root DTO consumer cleanup keeps that same root-facade boundary without restoring crate-root DTO aliases for `RuntimePluginId` and `RuntimeTargetMode`. The concrete definitions still live under `builtin/runtime_modules/ids/{plugin_id.rs,target_mode.rs}`, `zircon_runtime::builtin` owns the public DTO facade, and runtime platform/plugin/profile/project-manifest consumers now import `crate::builtin::{RuntimePluginId, RuntimeTargetMode}` or `zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode}` directly.

The same M2 hard-cut pass tightened internal id/report ownership without changing the public `builtin` DTO facade. Implementation files under `runtime_modules` now import `RuntimePluginId` and `RuntimeTargetMode` from the `ids` owner and import `RuntimeModuleLoadReport` from the `load_report` owner directly, instead of depending on parent `runtime_modules.rs` re-export wiring. Public callers should use the curated `zircon_runtime::builtin` DTO facade; crate-root DTO forwarding is not kept as a compatibility path.

## Invariants

- Root `runtime_modules.rs` must stay structural: child module declarations, curated re-exports, and test module wiring only.
- `zircon_runtime::builtin` is the public namespace for runtime module assembly helpers and runtime module DTOs. `zircon_runtime` crate root must not forward `builtin_runtime_modules()`, `runtime_modules_for_target(...)`, profile assembly helpers, manifest assembly helpers, `RuntimePluginId`, `RuntimeTargetMode`, `RuntimeModuleLoadReport`, or `RuntimeRequiredPluginMissing`.
- Internal implementation files below `runtime_modules` must import id and report DTOs from their direct owners: `ids::{RuntimePluginId, RuntimeTargetMode}` and `load_report::RuntimeModuleLoadReport`. They must not route sibling implementation dependencies through the parent facade exports in `runtime_modules.rs`.
- The assembly facade may expose target/profile/plugin registration entry points, but runtime module id re-export wiring belongs in `ids.rs`, plugin identity parsing belongs in `ids/plugin_id.rs`, target-mode declaration belongs in `ids/target_mode.rs`, load-report re-export wiring belongs in `load_report.rs`, load-report data storage belongs in `load_report/report.rs`, missing-provider DTOs belong in `load_report/missing.rs`, effective diagnostic projection belongs in `load_report/diagnostics.rs`, structured availability reports belong in `availability.rs`, plugin-domain availability policy belongs in `plugin_modules/availability.rs`, plugin module loading belongs in `plugin_modules/loader.rs`, extension-registry traversal belongs in `assembly/extension_inputs.rs`, feature dependency report handling belongs in `assembly/feature_reports.rs`, profile assembly flow belongs in `assembly/profile_modules.rs`, registration-report assembly flow belongs in `assembly/registration_reports.rs`, registration-input data assembly belongs in `assembly/registration_inputs.rs`, target/manifest module selection belongs in `assembly/target_modules.rs`, manifest defaults belong in `manifest.rs`, and concrete built-in module vector construction belongs in `core_modules.rs`.
- The only built-in plugin module loaded from this boundary remains the optional UI module behind `ui`; other runtime plugin implementations remain externalized to `zircon_plugins/*`.
- `RuntimePluginId` is an open string-newtype. First-party built-ins may add associated constants for ergonomics, but external plugin ids must not require editing engine core unless they need genuinely new built-in loader behavior.
- Generated export code must consume this facade or runtime/plugin catalog APIs; it must not duplicate profile assembly, required-missing diagnostics, plugin-domain mapping, or linked-provider crate fan-out.
- Render extension inputs exist only when the `graphics` feature is compiled. A `target-server` build constructs the core module vector without render descriptors/providers and without `ScriptModule`; it must not retain placeholder graphics/UI slots or runtime target checks as compatibility behavior.

## Validation

Frameworks 03 M1 server hard cutover passed WSL nightly checks for both `--no-default-features --features target-server` and the default feature set. The server dependency-tree gate found no wgpu, winit, taffy, glyphon, naga, swash, fontsdf, or woff2-patched packages. A later support-first review found that a default/client-compiled binary still selected `ScriptModule` for `ServerRuntime`; both graphics and non-graphics core-module assembly paths now exclude Script by target as well as compile feature. `frameworks_03_server_profile` passes 1/1 under the full default feature set (test 0.01s, cold command 26m52s), the fresh target-server lib check passes in 8m36s, and the server static guard passes 5/5. The broader per-domain and full-test matrix remains tracked by the Frameworks 03 plan.

The current implementation slice ran focused `zircon_runtime` checking after formatting. Workspace-wide validation remains a milestone testing-stage task because other active sessions are running concurrent Cargo lanes.

The 2026-06-07 plugin-module visibility repair was validated through the Sound linked-feature provider manifest lane because that path compiles `zircon_runtime` before the Sound optional-feature tests. `rustfmt --edition 2021 --check` passed over `plugin_modules/availability.rs`, `plugin_modules/loader.rs`, `assembly/extension_inputs.rs`, and the touched Sound parity test. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-linked-feature-provider-manifest-0607 --message-format short --color never` passed with existing `zircon_runtime` warnings only after the helper visibility repair. The follow-up Sound `optional_feature_manifest` test run passed with 4 tests, 0 failures, and 177 filtered tests, so the lower shared runtime split compiled under the linked-provider feature manifest, registration-report, and runtime-module descriptor paths.

The 2026-06-07 root-facade cutover was validated with scoped formatting and source guards over the touched Rust files. `rustfmt --edition 2021 --check --config skip_children=true` passed for the runtime root, prelude, app entry, dynamic API session, runtime absorption tests, plugin-extension tests, and app source guard files changed by the cutover. Live Rust, docs, and plan scans found no remaining crate-root or `crate::` assembly-function paths for the builtin module helpers. The root-facade static guard confirmed `zircon_runtime/src/lib.rs` exposes `pub mod builtin;` but does not contain the flattened assembly helper names. `git diff --check` passed for the touched cutover paths with expected LF-to-CRLF warnings only. A focused `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-builtin-root-facade-0607 --message-format short --color never` reached compilation but failed in the active WGPU render lane at `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs:132` because `create_mesh_draw(...)` is currently called with 21 arguments against a 20-argument signature; that graphics file is owned by the concurrent WGPU render session and is not part of this root-facade cutover.

The 2026-06-17 stale root DTO consumer cleanup was validated indirectly through the editor UI M3.S2 Material divider slice. Source scans found no live `crate::RuntimePluginId`, `crate::RuntimeTargetMode`, `zircon_runtime::RuntimePluginId`, or `zircon_runtime::RuntimeTargetMode` imports after consumers were aligned to the `builtin` owner, `zircon_runtime/src/lib.rs` exposes no runtime module DTO forwards, `cargo fmt -p zircon_runtime -p zircon_editor` passed, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` passed with existing warning noise only.

The 2026-06-23 Plugins 12/13 M5 string-newtype cutover was validated with scoped formatting over `ids/plugin_id.rs`, `plugin_modules/loader.rs`, the runtime-module structure guard, and the plugin workspace shape guard. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-check --message-format short --color never` passed with existing warning noise, and `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-sdk-check --message-format short --color never` also passed. The focused runtime lib-test `runtime_plugin_id_accepts_external_keys_without_core_variant` is present, but the current runtime lib-test lane is blocked before running by unrelated missing test child modules, private glTF fixture re-exports, and existing WGPU test API drift, so no focused runtime test pass is claimed for this slice.

The 2026-06-07 registration test owner split was validated with `rustfmt --edition 2021 --check zircon_runtime\src\builtin\runtime_modules\tests\registration\mod.rs zircon_runtime\src\builtin\runtime_modules\tests\registration\behavior.rs zircon_runtime\src\builtin\runtime_modules\tests\registration\structure.rs`, which passed after formatter wrapping in `structure.rs`. The focused folder-backed registration test guard passed for deleted flat `tests/registration.rs`, parent module wiring, wiring-only `registration/mod.rs`, behavior assertions in `registration/behavior.rs`, source-shape assertions in `registration/structure.rs`, and docs/session coverage. `audit_runtime_structure.py --json` also completed with no plugin runtime gaps, no unclassified public runtime modules, and no unclassified large-file hotspots. No fresh Cargo result is claimed for this slice because it only moved Rust unit-test ownership and updated documentation metadata.
