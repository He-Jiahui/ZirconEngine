---
title: Runtime Builtin and Engine Module Composition Currentness Adoption
date: 2026-08-23
scope:
  - zircon_runtime/src/builtin
  - zircon_runtime/src/engine_module
status: static_complete_dynamic_pending
source_fingerprints:
  builtin: c417adec07238502458bd5baf73997c91f37e83093d6df760d4e2859bd774ef5
  engine_module: 3ee67543ad37a9a68b69e18866aafc20bbd7ee0c2cdcfb4e961962145d34a87d
canonical_owners:
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ModuleDescriptor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
---

# Runtime Builtin and Engine Module Composition Currentness Adoption

## 1. Current coverage

`zircon_runtime/src/builtin/**` currently contains **30/30 Rust files, 3,324 physical / 3,048 non-empty lines, 128,926 bytes, and 37 test markers**. `zircon_runtime/src/engine_module/**` contains **8/8 Rust files, 509 physical / 437 non-empty lines, 14,323 bytes, and 8 test markers**. Their ordered workspace-relative path + NUL + raw bytes + NUL SHA-256 values are `c417adec07238502458bd5baf73997c91f37e83093d6df760d4e2859bd774ef5` and `3ee67543ad37a9a68b69e18866aafc20bbd7ee0c2cdcfb4e961962145d34a87d`.

Runtime42 previously read all 30 builtin files and the catalog/App/dynamic consumer chain. The committed builtin implementation has not changed since that 2026-08-16 review; the current foreign working-tree delta only adds `UiDocumentImporter` to UI-capable default manifests and adds two focused manifest tests. Runtime46 read all 8 engine-module files and the Core/Runtime/App materialization chain; the only later semantic commit did not change production code, and the current foreign diff is import ordering in `engine_module/tests.rs`. These changes are preserved and not claimed by this pass.

The 38/38 current files were reread directly or reconciled against those full owner reviews. No Cargo test or current-source product executable was run; test-marker counts are inventory only.

## 2. MVP conclusion

These modules already contain useful foundations: generated typed runtime profiles, typed builtin IDs, dependency-closure selection, Core topology sorting, structured availability/load diagnostics, a runtime plugin catalog, and local descriptor caching during one sort. `EngineModule` and Core also provide a real module/service lifecycle rather than a name-only registry. Those pieces should be retained.

The main startup bottleneck is duplicated authority and materialization, not the cost of eight facade files. A single product launch can interpret profile/target/manifest/provider inputs in builtin assembly, construct a feature catalog, flatten extension registries into owned vectors, return only module trait objects, run App plugin-group resolution again, regenerate descriptors, clone them for bootstrap, patch a factory by string, and finally register a different descriptor projection in Core. The work and memory scale with selections, registrations, extension payload and descriptors; more importantly, the generations can disagree.

The current UI Document Importer selection is a necessary reachability change but not a composition closure. `active_plugin_registration_refs` still checks each registration's embedded selection instead of the effective project manifest. Feature assembly still matches all registrations with an available feature ID rather than the selected provider. A disabled or unselected provider can therefore still contribute importer/render extensions through the flatten path, and adding another required default selection increases the importance of resolving one authoritative plan.

## 3. Performance findings in dependency order

| Priority | Current fact | Performance/correctness consequence | Owner |
|---|---|---|---|
| P0 | lazy service/plugin factory panic is outside the activation panic boundary and can skip `Initializing` slot reset/notification | a waiter may have no terminal state; startup or lazy resolution can hang rather than degrade | Runtime46 M0 |
| P1 | Runtime selection caches descriptors locally but `RuntimeModuleLoadReport` drops the cache and returns module objects | App/Core regenerate and clone descriptor trees; call count and bytes grow across layers | Runtime46 M1-M4 |
| P1 | builtin, catalog and App each interpret overlapping profile/provider/module truth | startup performs repeated graph/catalog/projection work and cannot prove equal generations | Runtime42 M1-M4 |
| P1 | registration filtering ignores effective manifest; feature flatten ignores selected provider | inactive contributions add work and can change actual importer/render behavior | Runtime42 M1/M3 |
| P1 | extension merge clones each registry family into independent vectors and loses owner/generation/order provenance | total copied bytes scale with extension payload; conflicts are only partially typed and rollback is impossible | Runtime42 M3 |
| P1 | target/profile defaults and enum membership change under `cfg`; required capabilities are metadata only | missing implementations disappear from intent instead of producing a rejected row, so work may start before closure is known | Runtime42 M0-M2 |
| P1 | `EngineModule` exposes name/description plus owned `descriptor()` authorities; `EngineService` clone contracts have no production consumer | repeated DTO ownership and drift remain; optimizing wrapper allocations would preserve the wrong contract | Runtime46 M1-M3 |
| P1 | report type can carry modules alongside fatal diagnostics and legacy helpers discard errors | rejected partial work can continue into later startup phases and waste I/O/factory activation | Runtime42 M0/M4 |

Core topology sort remains the correct final dependency validator and local descriptor single-evaluation should be preserved. The target is not a new graph algorithm. The target is to compile the graph once, retain the immutable result across Runtime, App and Core, and keep author callbacks out of subsequent query/bootstrap paths.

## 4. Unreal source constraints

Unreal `FModuleDescriptor` binds module identity to host type, loading phase and platform/target/configuration allow/deny policy. `LoadModulesForPhase` checks the declared phase and `IsLoadedInCurrentConfiguration` before calling ModuleManager, records a typed load result, and has a paired phase unload path plus module compatibility check. The relevant constraint for Zircon is that role/build/phase eligibility is decided before activation and remains queryable; it is not inferred from whether code happened to compile.

Unreal PluginManager resolves enabled plugins and dependency/configuration state before module load. ModuleManager then owns load failure reason, query status, module-changed notification, pre-unload/shutdown, unload versus abandon, and shutdown unloading. Zircon does not need to copy Unreal's UObject or DLL class structure, but it needs the same separation between compiled selection plan, activation runtime state and lifecycle receipt.

The reference does not justify Zircon's repeated descriptor generation. Zircon can exceed the C++ model by making `RuntimeCompositionPlan` content-addressed and immutable, pairing each compiled module declaration and factory binding once, then passing an `Arc` generation through App and Core.

## 5. Structural optimization plan

### M0: terminal factory and result semantics

Add a no-unwind factory trampoline and RAII initialization claim in Core. Immediate, Lazy and Plugin factory panic must reset owner/state, notify waiters and return a typed terminal diagnostic. Freeze `Ready | Degraded | Rejected`; a rejected composition cannot expose a partial module vector.

### M1: single proposal and identity schema

Replace independent `module_name`, `module_description` and executable owned `descriptor()` authorities with one validated `ModuleProposal`. Separate deterministic metadata from `FactoryBindingKey`/host bindings. Keep stable wire identities present across feature-off BuildSets and report `NotCompiled` instead of deleting schema members.

### M2: one composition compiler

Use `RuntimePluginCatalog::project_plan_for` as an input to a single compiler over ProductRole, Platform, BuildSet, ProfileIntent and effective ProjectManifest. Produce one terminal row for every selection/provider/capability/module/extension, a canonical graph, source hashes and a plan generation/hash. Filtering must consume the effective manifest and the exact selected provider.

### M3: transactional extension and graph publication

Merge all `RuntimeExtensionRegistry` families once in dependency order into a staging generation, preserving provider/owner/provenance and explicit conflict policy. Compile `ResolvedModuleEntry { proposal, compiled descriptor, binding }` rather than parallel module/descriptor vectors. Validate then publish the plan atomically; failures expose only the previous complete generation.

### M4: Runtime-App-Core hard cut

`RuntimeModuleLoadReport`, App `EngineEntry`, dynamic sessions, Editor and export consume the same `Arc<RuntimeCompositionPlan>`/receipt. App supplies host bindings before compile and does not rerun builtin/plugin-group resolution, regenerate descriptors, append unplanned modules or patch factories by string. Core registers that exact generation and owns activation/rollback/teardown state.

## 6. Quantitative acceptance

1. Scale selections/providers/modules/extensions at `1/100/1,000/10,000`; record plan builds, catalog builds, descriptor calls, graph vertices/edges, extension visits, cloned bytes, peak RSS and wall p50/p95. Identical launch inputs require plan build count = 1 and author descriptor/proposal evaluation = 1 per owner.
2. Test extension payload at `1/1,000/100,000` rows and `1 KiB/1 MiB/64 MiB`; inactive/unselected provider visits and copied bytes after resolution must be zero. Canonicalization/order changes must not change plan hash.
3. Exercise missing required capability, disabled registration, duplicate provider, dependency cycle, phase violation, factory error/panic/hang, partial activation and rollback. Every row and operation must reach a typed terminal result; rejected plans publish zero new contributions.
4. Compare Client2D/3D, Editor, Dev, Server and headless BuildSets with graphics/script/ui on/off. Intent/schema stays stable; absent implementation is explicit and no `cfg` combination silently removes a required row.
5. WPR/ETW on the current-source product executable records startup CPU, main-thread blocking, locks/waits, factory thread, file/DLL I/O, module activation timeline, RSS and energy. RenderDoc only correlates the first visible frame with the same composition generation; it cannot prove composition CPU or startup power.

## 7. Current result

- Builtin 30/30 and EngineModule 8/8 have current-source composite static coverage.
- Foreign UI Document Importer manifest/tests and import-order changes are preserved; production/test edits by this pass are zero.
- Runtime42's 52 P1 / 42 gates and Runtime46's factory P0 / 48 P1 / 36 gates remain open under their canonical owners.
- No Cargo, product, WPR/ETW or RenderDoc evidence exists for this pass, so both folders remain dynamically pending.
