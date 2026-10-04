---
title: Material Editor Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/material_editor
status: static_complete_shared_source_preserved_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Editor/MaterialEditor/Private/MaterialEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/MaterialEditor/Private/MaterialGraph/MaterialGraphSchema.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Materials/MaterialShared.cpp
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/godot/drivers/gles3/storage/material_storage.cpp
---

# Material Editor Current Source Performance Review

## 1. Coverage

The current package surface is **6/6 Rust files**, **1,263 physical / 1,163 non-empty lines**, **46,420 bytes**, **13 tests** and **1 ignored performance test**. Its package-relative `path + LF + raw bytes` SHA-256 is `faacb01f407e5d2431a84be81bfee6f6e82858611be42a976258f45bedcd8d4a`.

Every Dist and Editor Rust file was indexed and parsed. Capability/extension declarations, plugin/package registration, operations, asset toolkits, graph/palette descriptors, graph validation, material lowering, recursive expression evaluation, native registration and all tests were reviewed. The Runtime authoring importer, both Runtime `MaterialGraphAsset` definitions, `MaterialAsset` contract and shader/pipeline ownership were traced to establish product truth. `README.md`, Editor `lib.rs` and Editor `tests.rs` already had shared changes at review start; they were treated as current source and were not edited or formatted by this review.

| Area | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| Dist | 1 | 102 | Publishes registration metadata, but has zero bridge methods and no command invocation path. |
| Editor | 5 | 1,161 | Registers an authoring surface and contains a test-only reachable base-color folder; no product handler, document or preview installs its result. |

## 2. Structural performance findings

### P0: the default Editor product cannot execute or display this package

The default first-party Editor catalog projects Navigation and Neural only; Material Editor is not a linked provider. The package registers six operations (`open`, `open_material`, `validate`, `compile`, `preview`, `create`), but workspace-wide searches find no operation factory/handler or product caller. Native Dist exposes a registration manifest while `invoke_command` is absent and its bridge method list is empty.

The registered surface and creation template reference `plugins://material_editor/editor/graph.zui` and `plugins://material_editor/templates/default_material_graph.toml`; neither resource exists anywhere in the package. The graph/palette descriptors reach registries and snapshots, but there is no material graph canvas consumer. Performance claims about the current compiler cannot describe MVP Editor behavior because the behavior is not executable.

The resolved profile must materialize one source/native operation and resource contract or fail capability admission. Registration metadata and success-shaped menu text cannot substitute for a working document/compiler/preview product.

### P0: “compile” is not a material/shader graph compiler

`compile_material_graph` follows only the output node's `base_color` input. It folds scalar/vector defaults and Add/Multiply into one `[f32; 4]`, or copies one texture reference. It then creates a `MaterialAsset` with normal, metallic, roughness, occlusion, emissive, alpha, two-sided, property values, texture slots, options and queue all hard-coded to defaults/empty values. It generates no shader source/IR, reflection, layout, variant, pipeline/PSO recipe or versioned derived artifact.

The authoring graph therefore cannot express the Runtime material contract it claims to target. Runtime loads a graph as `ImportedAsset::MaterialGraph`; it never invokes this Editor function to derive a `MaterialAsset`. Graphics/shader also defines another unrelated `MaterialGraphAsset` containing only `name + output_domain`. These duplicate owners have no canonical conversion, schema version or shared compiler.

The target must be one Runtime-owned semantic compiler that lowers a versioned graph to a typed material artifact plus shader artifact/recipe consumed by the same Runtime pipeline and preview. Constant folding is one compiler pass, not the product output.

### P0: palette, validation and evaluator disagree on types and pins

The palette declares Add/Multiply as `float + float -> float`, while tests feed a `vec4` and `float`, expect component-wise broadcast, and connect the declared float result to the output's required `vec4` pin. Validation checks only that pin strings are non-empty; it does not verify that a pin exists, direction is correct, its type is compatible or `from_pin` is the selected output. Evaluation ignores `from_pin` entirely. A link from an arbitrary/nonexistent source pin can therefore compile.

Parameter map type mismatches silently fall back to node defaults, and non-finite numeric values are not rejected. Texture-backed Add/Multiply is advertised by the untyped topology but fails only during recursive evaluation. The importer is weaker still: it accepts any graph containing at least one output node, so duplicate IDs, multiple outputs, invalid pins, cycles and incompatible types can enter the asset/resource system before Editor validation.

Pin/cardinality/type/domain rules must be schema-owned and applied at edit connection time, import, compile, cook and Runtime admission. Unreal's `MaterialGraphSchema.cpp` resolves input/output value types, checks compatibility and replaces an existing non-exec input connection through one schema path; Zircon currently has no equivalent semantic authority.

### P0: recursive DAG evaluation can be exponential

The shared change correctly builds one borrowed node/incoming-link index, removing the old node-map reconstruction and full link scan from every recursive visit. However, `evaluate_color_input` memoizes nothing. A valid DAG where both inputs of each Add/Multiply point at the same previous node evaluates that shared subgraph twice at every level.

For depth `D`, this creates roughly `2^(D+1)-1` node visits despite only O(D) unique nodes. Depth 20 is **2,097,151 visits** and depth 30 is **2,147,483,647 visits**, each including ordered-map/set work. A deep linear graph also retains recursive stack risk. The current ignored benchmark uses a 256-node linear chain with one shared constant; it cannot detect exponential fan-out or stack exhaustion.

The real compiler needs one non-recursive topology pass and one result per dense node slot, yielding O(V+E) lowering/evaluation after name resolution. Common subexpressions should be represented once in typed IR and constant-folded/memoized once. Explicit node/edge/depth/diagnostic/artifact budgets must fail boundedly.

### P1: the local indexed optimization is useful but not product evidence

The current borrowed `MaterialGraphIndex` reduces legacy recursive behavior from rebuilding a BTree node map and linearly scanning links on every visit to one O((V+E) log V) index build plus ordered lookups. This is a sound local correction and the multiple-incoming-link diagnostic closes one ambiguity.

Its performance test is ignored, uses `Instant` inside the unit-test binary, performs only two evaluations per sample, has no managed hardware/build receipt, and compares against an artificial legacy helper that no longer drives the product. It has not been executed in this review because the managed Windows validator is unavailable. It can guard the removed regression later, but cannot prove Editor responsiveness, compile scale or Runtime rendering.

### P1: compile/edit/preview work has no lifecycle or currentness

There is no document revision, dependency graph, debounce/coalescing, compile ticket, cancellation, priority, stale-result rejection, last-good artifact, DDC key or preview generation. If the registered operation were simply wired to the current synchronous function, graph traversal would run on the UI path and repeated edits could enqueue/repeat whole compilation without a bounded latest-wins policy.

Unreal separates graph change/update from shader-map identity, async DDC/compile jobs, cancellation and final installation. Bevy exposes `Queued/Creating/Ok/Err` pipeline states, requeues shader dependents on shader changes and uses the async compute pool where supported. Godot queues material dirty updates and distinguishes uniform/texture invalidation. Zircon needs the same ownership boundaries, with its own typed receipts and scheduler rather than copying any one engine's object model.

### P1: no telemetry can distinguish graph, shader, pipeline or preview cost

There are no counters/timings for graph index/topology/type inference, node visits versus unique nodes, constant-fold/cache hits, generated source/IR/artifact bytes, Naga/backend compilation, shader variants, PSO creation, queue wait, cancellation/stale drops, preview install or last-good fallback. There are also no correctness images proving that a preview is rendering the current artifact generation.

Instrumentation must follow the artifact pipeline. RenderDoc is useful only after a real preview exists, to verify shader/pipeline/draw/resource binding and pixel parity; WPR/ETW owns CPU scheduling, waits, UI stalls, allocations and power evidence.

## 3. Reference-engine constraints

Unreal is the primary architecture constraint:

- `MaterialGraphSchema.cpp` owns connection direction, cardinality and material value-type compatibility before graph mutation. Graph UI and compiler do not invent separate pin semantics.
- `MaterialEditor.cpp::UpdateMaterialAfterGraphChange` relinks material expressions, invalidates preview/code/stat generations and refreshes node previews according to live/realtime policy. Zircon must preserve the generation boundary without copying Unreal's synchronous `FlushRenderingCommands` behavior.
- `MaterialShared.cpp::BeginCacheShaders` builds a shader-map identity, checks inline/in-process/DDC results and starts compilation only on miss. `CancelCompilation` cancels outstanding shader-map IDs; `FinishCompilation` is an explicit blocking boundary rather than incidental UI work.
- Shader-map/pipeline artifacts are the executable result. A constant `base_color` summary cannot stand in for compilation.

Secondary engine checks reinforce the execution model. Bevy's pipeline cache publishes explicit pending/success/error states, retries unavailable shader dependencies and moves pipeline creation to `AsyncComputeTaskPool` when supported. Godot's material storage queues dirty materials and tracks uniform/texture dirtiness instead of recompiling every material property path unconditionally.

## 4. Dependency-ordered optimization plan

### M0: close product and capability truth

Add Material Editor to an explicit resolved Editor profile only when its source/native implementation, ZUI/template resources, document toolkit and six operation factories exist. Make unsupported packaging fail closed. Native Dist must execute an equivalent versioned command/registration contract or stop advertising authoring behavior.

Replace fixed registration-only success with typed operation results carrying source/document generation, artifact generation and failure reason.

### M1: establish one versioned graph schema and compiler authority

Hard-cut over the duplicate material-graph types to one versioned authoring schema with stable node/edge IDs, typed pins, cardinality, domain, parameters, dependencies, migrations and unknown-node preservation. The same structural/semantic validator must run on edit, import, compile, cook and Runtime load.

Build a non-recursive reachability/topology pass, reject/diagnose cycles by one policy, allocate dense node slots and evaluate/lower every reachable node once. Add explicit graph scale and diagnostic budgets. Remove the Editor-only recursive evaluator after consumers adopt the shared compiler.

### M2: lower to executable material/shader artifacts

Produce a typed material IR covering the actual `MaterialAsset` domains and outputs. Constant-fold pure nodes and retain dynamic parameters/textures as typed bindings. Lower shader-dependent expressions to validated shader IR/WGSL, source maps, reflection/layout, variants and pipeline recipes through Runtime91's compiler owner.

Key immutable artifacts by normalized source, transitive dependency revisions, schema/compiler version, target/backend/profile and options. Store them in the configured non-C DDC. Import/cook/preview/packaging must consume the same artifact schema and preserve last-good artifacts on failed revisions.

### M3: make Editor compilation incremental and cancellable

Publish immutable document generations and dependency changes. Debounce/coalesce edit bursts, schedule latest-wins compiler work off the UI thread, cancel superseded jobs and reject stale completions. Track affected-node/dependency closure so pure layout edits cause zero semantic compile and local value changes avoid unrelated graph work.

Install the exact current artifact into an isolated preview session. Expose current/pending/failed/last-good generations, diagnostics and compile/pipeline state without blocking input or render threads.

### M4: instrument and qualify current-source behavior

Create deterministic linear, deep, wide, diamond/fan-out, cyclic, invalid-pin/type, texture/parameter-heavy and dependency-burst fixtures. Measure cold/warm graph compile, DDC, shader/backend compile, PSO and preview install p50/p95/p99; record node/edge/unique visits, artifact/source bytes, allocations, queue delay, cancellations, stale drops and cache outcomes.

After a managed Windows current-source executable exists, capture real edit bursts and stable preview with WPR/ETW. Use RenderDoc to verify that the current material artifact selects the expected shader/pipeline, bindings, draws and pixels. Report CPU/GPU timings, UI latency, frame pacing and energy/frame against fixed scene/hardware/power settings.

## 5. Acceptance gates

1. The default selected profile opens a real Material Graph document and every advertised operation executes, or admission fails with a typed reason.
2. All referenced package resources exist and source/native packaging resolves equivalent authoring behavior.
3. One versioned schema/compiler owns edit, import, cook, preview and Runtime semantics; duplicate placeholder `MaterialGraphAsset` types are removed or renamed to their real contract.
4. Pin existence/direction/cardinality/type/domain and numeric finiteness are validated identically before mutation and compilation.
5. Compiler work is O(V+E) after name resolution; each reachable node is lowered/evaluated once, diamond graphs do not exhibit exponential visits and deep graphs do not recurse on the process stack.
6. Output is an immutable executable material/shader artifact with reflection/layout/variant/pipeline identity, not a base-color-only `MaterialAsset` summary.
7. Edit bursts are latest-wins and cancellable; stale results never replace current/last-good preview generations and stable preview frames perform zero compilation.
8. Cold/warm DDC, shader compile, PSO and preview evidence reports p50/p95/p99, queue/cancel/stale/cache data, allocations/bytes, frame latency and energy/frame.
9. Managed tests, current-source launchability, WPR/ETW evidence and RenderDoc correctness parity pass before protected-ledger promotion, milestone commit or WeCom completion notification.

## 6. Validation status

- Static per-Rust-file review: **6/6 complete** for the captured source fingerprint.
- Runtime/product trace: authoring import, duplicate graph types, `MaterialAsset` and shader/pipeline boundaries were read to verify the product split.
- `rustfmt --check --config skip_children=true`: Dist, `capability.rs`, `extension_ids.rs` and `plugin.rs` pass; shared current changes in Editor `lib.rs` and `tests.rs` have formatting-only diffs and were preserved.
- Shared indexed optimization: statically confirmed; it removes recursive map rebuild/full-link scans but does not memoize DAG evaluation.
- Source changes by this review: **none**.
- Product closure: **failed statically** because the default catalog, resources, operation handlers, graph consumer and native invocation path are absent.
- Algorithm scale: **failed statically** because valid shared-subgraph DAGs can cause exponential recursive visits and deep graphs retain stack risk.
- Cargo/test/ignored performance gate: **pending** because the managed Windows validation session is not executable; no raw Cargo lane was substituted.
- Current-source executable, WPR/ETW timing/power and RenderDoc preview qualification: **pending**. No current-source executable exists, so those tools were not run.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.

