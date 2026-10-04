---
title: Plugin UI Document Importer Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/ui_document_importer
status: static_complete_shared_source_preserved_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
references:
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/WidgetBlueprintCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/WidgetBlueprintGeneratedClass.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/UserWidget.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Blueprint/WidgetBlueprintGeneratedClass.h
  - dev/slint/internal/compiler/lib.rs
  - dev/slint/internal/interpreter/api.rs
  - dev/slint/internal/interpreter/dynamic_item_tree.rs
---

# Plugin UI Document Importer Current Source Performance Review

## 1. Coverage and evidence state

The package review covers **4/4 Rust files**, **696 physical / 624 non-empty lines**, **25,357 bytes**, **12 tests** and **1 ignored performance test**. The package-relative `path + NUL + LF-normalized bytes + NUL` SHA-256 is `6151743066ee99a4b9a0efed1458f795fcdf187048f01796562f74d3cbd99017`.

| Area | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| Dist | 1 | 108 | Exports ABI-v3 registration metadata, but has no import command or bridge method. Diagnostics explicitly say the importer remains Runtime-hosted. |
| Runtime capability/registration | 2 | 171 | Declares one priority-120 `.zui` importer and stable source/native packaging metadata. The first-party Runtime catalog links the source provider behind `ui-document-importer`. |
| Runtime importer/tests | 1 | 417 | Borrows the already-read source bytes, parses TOML into a UI v2 AST and wraps it as a view, component or style imported asset. It does not produce a compiled Runtime UI artifact. |

`runtime/src/lib.rs` and `runtime/src/plugin.rs` already had shared changes at review start and were preserved without editing or formatting. Per-file `rustfmt --check --edition 2021 --config skip_children=true` passes **3/4** files. `runtime/src/plugin.rs` retains a pre-existing import-order/vector-layout formatting diff.

The product trace included the Runtime UI loader, artifact-cache payload, prototype store, compiler, component instancer, surface builder, direct-reference extraction, editor file cache, editor view projection/template host and first-party Runtime catalog. A structured read of the current non-`dev` UI corpus found **318/318 parseable `.zui` files**, **3,215,413 source bytes**, **5,594 nodes**, **248 component definitions**, **708 widget imports**, **162 style imports** and **8 resource imports**. All 708 current widget imports are URI-based, so the editor file cache's bare-asset-ID full-root scan is a dormant scale risk, not an observed current-corpus cost.

Managed Rust tests, WPR/ETW and RenderDoc were not run. The current session has no executable managed Windows validator identity and no launchable current-source engine/editor binary. The findings below are source-proven algorithm and ownership findings; absolute latency, frame-time and power claims remain pending.

## 2. Structural performance findings

### P0: the artifact is source text in another envelope, not a cooked UI product

The importer correctly avoids one redundant UTF-8 allocation by calling `AssetImportContext::source_str()`, then parses `.zui` into `UiV2AssetDocument`. The artifact cache subsequently serializes that AST back to `document_toml: String`. Runtime loading deserializes the cached payload and calls `UiV2ViewAsset::from_toml_str`, `UiV2ComponentAsset::from_toml_str` or `UiV2StyleAsset::from_toml_str`, parsing TOML into the same authoring AST again.

The current product pipeline is therefore:

`source bytes -> TOML parse/validate -> authoring AST -> TOML serialize -> artifact payload -> TOML parse/validate -> authoring AST -> component expansion/validation -> compiled arena -> surface tree`

This makes source syntax and authoring maps part of Runtime startup, duplicates parser/allocator work and prevents DDC from publishing the actual product consumed by the renderer/input/layout systems. The ignored package benchmark measures only `source_text()` cloning versus `source_str()` borrowing over invalid 2 MiB filler data; it never parses a document or exercises artifact load, dependency closure, compile, surface construction, layout or pixels.

MVP needs a versioned cooked UI artifact containing validated dependency identities, resolved component/style/resource tables, compact node/property data and compiler/schema generations. Source AST remains an editor/import input. Runtime must not serialize it back to source TOML and parse it again.

### P0: Runtime startup synchronously loads every registered UI artifact before checking reachability

`RuntimeUiSurfaceSet::load` calls `project_ui_prototype_store` on the session-loading path. That function walks every asset-registry entry of `UiLayout`, `UiWidget` or `UiStyle`, synchronously calls `project.load_artifact`, converts every matching artifact to a document and inserts it into a store. Only after all UI artifacts are loaded does `build_for_roots` traverse and validate the requested roots. It returns the full store rather than a reachable subset.

For `A` registered UI artifacts and `R` requested roots, startup pays O(A) artifact I/O/deserialization before O(reachable dependency closure), then independently compiles and builds each of the R root surfaces. Unreferenced editor, experimental and package UI cannot fail root validation, but they still consume I/O, parsing and resident memory. This is the opposite of the comment's claimed root-oriented loading behavior.

The required boundary is an asset-registry dependency graph plus root-driven async loading: resolve only the transitive closure, admit bounded parallel reads/decompression, compile/cache by closure generation and publish each root only when its exact dependency generation is complete. A registry-wide scan is acceptable for an explicit catalog/index build, not normal Runtime UI startup.

### P0: component compilation repeatedly clones and revalidates whole document graphs

`UiV2DocumentCompiler` first materializes a new document through `UiV2ComponentInstancer`, then creates a string-keyed handle map, revalidates reachability, clones nodes into an arena and clones identity/parent/child data again into a component graph.

The amplification is worse for component-only documents. For each of `C` component definitions, the compiler clones the complete document, installs that component as a temporary root, expands it and validates typed events; after the loop it returns another full document clone. Each component occurrence encountered during expansion calls `validate_source_graph` on its prototype again. `resolve_component` also linearly reparses/scans the document's widget-import list for ordinary imported component names.

With `D` authoring-document bytes, `N/E` nodes/edges, `K` expanded component occurrences, maximum nesting depth `H` and `W` widget imports, the current source establishes these avoidable bounds:

- component-library validation retains an O(C x D) full-document clone term before expanded output;
- repeated prototype graph validation can approach O(K x (N + E) log N) with the current `BTreeSet` traversal;
- ordinary imported-component lookup is O(K x W), plus repeated reference parsing;
- per-task component-stack cloning/linear cycle lookup adds O(K x H) string movement/search;
- arena and component-graph construction add further O(N + E) cloning after expansion.

The fix is not a container micro-tune. Import/cook should resolve a per-document symbol table, validate each prototype graph once per source/dependency generation and compile stable node/component indices. Runtime instantiation should copy only mutable instance state and explicit overrides, while sharing immutable prototype/style/property data where ownership permits.

### P0: game Runtime and Editor use two different compile/cache authorities

Game Runtime uses registry artifacts, reparses cached TOML, loads all UI documents and recompiles each root without a compiled-document cache. Editor view projection and template Runtime instead use `UiV2PrototypeStoreFileCache`, which can persist a bincode record containing the source documents and `UiV2CompiledDocument`.

The editor cache is still caller-synchronous. Its non-event path probes canonical path, mtime and length for sources under a process-global mutex; a miss reads/parses dependencies, compiles and may write the persistent cache while that mutex remains held. View projection can use watcher-driven `load_store_cached`, but `EditorUiHost` calls `load_store` directly. Bare asset-ID references can additionally trigger a sorted recursive scan and parse of every `.zui` under the asset root. The present 318-file corpus does not use that fallback, but the path has O(all UI files) first-use behavior and is a design trap for third-party content.

One compiler service and one artifact identity must serve import/cook, game Runtime, editor shell, authored preview and package content. File watchers should invalidate generations only. I/O, dependency resolution and compilation belong in cancellable background jobs; the UI thread installs an immutable matching generation and retains last-good output on failure.

### P1: dependency/currentness truth is indirect and can silently lose invalid references

The importer returns only the root imported asset. A later generic `ImportedAsset::direct_references` pass extracts widget, style and resource locators. That is a valid ownership split, but `push_reference` silently returns when `ResourceLocator::parse` or normalized locator construction fails. Widget-import syntax is later parsed by the prototype-store builder, while invalid resource locators can disappear from dependency/currentness data instead of producing an import diagnostic.

The cooked artifact needs a typed dependency table produced during semantic compilation. Every reference must resolve to a stable asset identity or emit a source-located diagnostic; content hash/currentness keys must include the exact dependency generations. Runtime should not rediscover or reinterpret dependency strings.

### P1: native packaging advertises more behavior than the native entry executes

The package is marked stable and supports `native_dynamic`, and its Dist entry reports the importer registration manifest. However, `invoke_command` is `None`, `bridge_methods` is empty and the diagnostic says the importer remains hosted by the Runtime module. The source first-party catalog does link and register the functional importer, so source builds have a real path; a native-only package does not.

Capability admission must distinguish registration metadata from executable import behavior. Native dynamic packaging either needs a host bridge that supplies source/context and returns a versioned compiled artifact, or it must fail closed and require source/library embedding. Metadata parity is not product parity.

### P1: current tests qualify registration and tiny documents, not the product hot path

Most tests assert manifests, provider registration and minimal one-node parsing. There is no scale fixture for dependency fan-out, many component instances, deep nesting, component libraries, multiple roots, cold/warm artifact load, cache invalidation, edit bursts or failure recovery. No counters expose bytes parsed/serialized/cloned, artifacts considered/loaded, dependency closure size, component lookups, graph validation visits, cache generation, compile queue/wait time or UI-thread blocking.

The existing borrowed-source correction is useful and should stay, but it removes only one allocation before the much larger text/artifact/compiler pipeline. It is not evidence that UI startup or editor preview is qualified.

## 3. Reference-engine constraints

Unreal is the primary architecture constraint:

- `WidgetBlueprintCompiler.cpp` performs full compilation work in the editor/compiler boundary and duplicates the authored WidgetTree into `UWidgetBlueprintGeneratedClass` as the class archetype. Runtime does not reopen and reparse the authoring source to discover the widget graph.
- `WidgetBlueprintGeneratedClass.h/.cpp` owns the tree of widget templates, animations and bindings. `InitializeWidgetStatic` instantiates from that class-owned tree and then binds properties/navigation/delegates against the created tree.
- `UserWidget.cpp` uses an object-instancing graph to duplicate and initialize the prepared archetype. Necessary per-instance work remains, but semantic compilation and source parsing are outside that path.
- Unreal precomputes `bWidgetTreeContainsSubstitutableArchetypes`; the common path pays one boolean check instead of scanning every widget for variant substitution. Zircon should likewise compile feature/capability bits rather than rediscover them per instance.

Slint independently reinforces the same boundary. `compile_syntax_node` recursively loads dependencies, builds the object tree and runs compiler passes. The interpreter exposes a reusable compiled `ComponentDefinition`, which instantiates an `ItemTreeDescription`; it does not parse the declarative source on each instance creation.

These references do not justify copying Unreal object mechanics into Rust. They constrain ownership: authoring syntax and semantic compilation precede Runtime instantiation; dependencies and feature decisions are compiled; Runtime consumes a reusable product artifact.

## 4. Dependency-ordered optimization plan

### M0: establish one executable capability and measurement contract

Make source/library/native provider selection explicit and fail native-only admission until the importer can execute. Add fixed cold/warm fixtures covering source import, artifact load, dependency closure, compile and surface creation. Record artifact/source/compiler/dependency generations and exact bytes/visits/allocations before changing algorithms.

### M1: define the canonical cooked UI artifact

Introduce a versioned `UiV2CompiledPackage` (name provisional) produced from `.zui` plus its dependency closure. It should contain stable asset/component/node indices, resolved imports, validated event/slot/parameter contracts, compact immutable prototype data, precomputed feature bits and schema/compiler/target/profile identity. Preserve authoring AST and source locations in editor-only artifacts or sidecars, not shipping Runtime payloads.

Replace TOML-in-artifact round trips with deterministic binary/structured serialization of this product artifact. DDC keys must use source content, dependency generations, compiler/schema versions, target/profile and relevant component/theme catalog generations rather than path mtime/length alone.

### M2: make loading root-driven, asynchronous and generation-safe

Publish UI dependencies into the asset registry during import. Resolve the transitive closure from requested roots, load only that closure with bounded concurrency and share immutable artifacts across roots. Remove the registry-wide UI artifact load from session startup.

Use cancellation, priority, deadlines and latest-wins generation rejection. Keep the last-good compiled surface during editor reload. No global UI cache mutex may cover filesystem I/O, parsing, compilation or persistent-store writes.

### M3: compile prototype graphs once and instantiate by indices

Build per-document component/import symbol tables once. Validate each prototype graph once per artifact generation, cache cycle/reachability results and remove whole-document-per-component clones. Replace string import scans and per-task string stacks with stable indices plus bounded generation-aware visit state.

Compile root surfaces by `(root artifact, dependency closure, component catalog, theme/style schema)` identity. Share immutable prototype/property/style tables; allocate only instance nodes, dynamic values and explicit patches. Preserve deterministic diagnostics and authored IDs through source maps rather than retaining the whole source-shaped graph in each instance.

### M4: converge Editor, game Runtime and package paths

Make editor view projection, `EditorUiHost`, game Runtime and packaged content consume the same compiler service and artifact schema. Watchers invalidate asset generations; background jobs rebuild; UI-thread code only swaps an accepted immutable result. Retire the bare-ID asset-root scan after registry identity lookup is available, and remove duplicate parser/validator authorities.

Native Dist must either call the same compiler contract through a bounded ABI or truthfully require the source provider. Preview and game Runtime must install the same artifact bytes for the same key.

### M5: profile and accept fixed workloads

Instrument source/artifact bytes, parser/serializer calls, artifacts considered/loaded, reachable assets, component occurrences, import lookups, graph visits, cloned bytes, allocations, cache hit/source, job queue/wait/run time, stale/cancel counts and surface build/layout time. Add cold/warm and edit-burst fixtures across the current 318-document corpus plus synthetic fan-out/depth/instance scale cases.

Once a managed current-source executable exists, use WPR/ETW for CPU stacks, waits, file I/O, allocation, scheduler latency and energy/frame. Use RenderDoc only after an actual UI scene renders, to verify draw/batch/clip/atlas/resource and pixel parity; it cannot qualify TOML parsing, main-thread stalls or job scheduling.

## 5. Acceptance gates

1. Source, library-embedded and admitted native providers execute one truthful importer/compiler contract; registration-only native packaging fails closed.
2. Shipping/runtime UI artifacts contain compiled product data and never serialize an AST back to TOML for Runtime reparsing.
3. Runtime UI startup loads only the requested roots' dependency closure. Unreachable UI asset count does not affect artifact I/O, parse count or resident prototype-store size.
4. Every prototype graph and semantic contract is validated once per artifact generation; component-library compile has no O(C x D) whole-document clone term.
5. Imported component lookup uses compiled indices, not per-occurrence string parsing and linear import scans. Instance work scales with expanded instance data plus explicit overrides.
6. Editor and game Runtime use the same artifact identity and compiler implementation. No process-global mutex encloses file I/O, parse, compile or persistent-cache writes.
7. Invalid widget/style/resource references produce source-located import diagnostics and cannot silently disappear from dependency/currentness keys.
8. Fixed cold/warm/fan-out/depth/multi-root/edit-burst tests report p50/p95/p99 plus bytes, visits, allocations, queue/wait time and cache generations.
9. Managed current-source tests pass, WPR/ETW shows no unbounded caller-thread UI load/compile stall, and RenderDoc verifies the corresponding rendered UI output before protected-ledger promotion, milestone commit or WeCom completion notification.

## 6. Dynamic validation boundary and source disposition

No production code was changed in this review. The already-shared borrowed-source change is retained, but the dominant defects cross importer, asset artifact, Runtime UI compiler/session, editor cache/job and native-provider boundaries. A local importer-only patch would preserve the wrong architecture.

The next dynamic gate requires a managed Windows current-source executable under an approved non-C target root. Absolute timing, power and comparison with Unreal/Slint remain unclaimed until fixed hardware/build/workload receipts exist. Static completion alone does not justify a milestone commit or WeCom completion message.
