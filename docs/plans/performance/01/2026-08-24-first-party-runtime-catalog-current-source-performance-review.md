---
title: First-Party Runtime Catalog Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/first_party_runtime_catalog
status: static_complete_product_and_dynamic_pending_source_recheck_required
canonical_owners:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
---

# First-Party Runtime Catalog Current Source Performance Review

## 1. Coverage and currentness

The current Rust surface is **5/5 files**, **1,239 physical / 1,186 non-empty lines**, **43,872 bytes** and **11 test markers**. Its workspace-relative `path + LF + decoded text + LF` SHA-256 is `0b7c2a16cfe5739b1c427240ebaa3667b1b64db86b5664c1100e538fb19d9377`.

The directory has four pre-existing/shared modified files. Their current diff adds UI Document Importer Cargo wiring, one registration branch and focused tests, increasing current Runtime provider coverage from the owner report's previous 14 to **15**. This review treats those changes as current input, does not modify them and requires a source recheck before implementation or acceptance.

All five Rust files, Cargo feature groups, the complete current diff, thirty manifest-declared Runtime packages, App adapters/features, six profile presets, `cargo-zircon` catalog checking, Plugins06 and Runtime42 were read. Projection occurs during Runtime/Editor composition rather than per frame.

## 2. Current cost truth

The projection itself is not the MVP bottleneck. It lazily scans enabled target selections once, preallocates a dedup `HashSet` and result `Vec`, and performs at most fifteen feature-gated provider comparisons per unique selection. Its current complexity is `O(selections * linked branches)` with a small startup-only branch count.

The more material cost is provider construction: each match eagerly calls `plugin_registration()`, which can build module/system/importer/feature descriptors and diagnostics. Coarse Cargo groups also link code that the selected profile may never instantiate: `base-runtime-plugins` links eight providers and `advanced-render-runtime-plugins` links four. Fine-grained buildsets should control code/link size while project selection controls activation; micro-optimizing the ID clone would not address either cost.

The 39 static manifest strings and the hand-written TOML parser live under `#[cfg(test)]`, so they do not bloat production binaries. The audit helper uses `OnceLock` to run its Python process once, although several tests clone/parse the captured output. This is test cost, not Runtime frame cost.

## 3. Structural bottlenecks

### P0: only 15 of 30 Runtime source packages are reachable

Thirty top-level plugin manifests declare a Runtime module. The current catalog exposes fifteen provider branches: AI, Sound, Texture, Net, Navigation, Particles, Animation, Rendering, glTF Importer, UI Document Importer, Neural, Virtual Geometry, Hybrid GI, Solari and Zr VM Language.

The remaining fifteen include the five `asset_importer.*` packages, Audio/OBJ/Opus/Shader WGSL/Texture importers, Physics, Prefab Tools, Terrain, Tilemap 2D and the native fixture. The fixture should be product-excluded and some importers may be generated/export-only, but production packages cannot simply disappear from source composition. Each needs an explicit linked/generated/native/externalized/unsupported decision.

### P0: standard profile intent and compiled provider closure contradict each other

Client2D and Client3D declare Sound and Rendering required, with `allow_externalized_required_plugins = false`, but their App features use `target-client` and do not enable `first-party-runtime-plugins`/the base catalog. Editor and Dev declare the same default required plugins, yet their App features enable only advanced-render, Navigation, two Editor providers and UI through the UI feature; they also omit the base Runtime catalog that contains Sound and Rendering. Server lists Net optional while linking no first-party Runtime catalog.

The current UI Document Importer wiring closes one real UI dependency, but does not repair these profile contradictions. A build can compile successfully while the declared required provider is physically absent.

### P0: required missing providers are deliberately erased

The catalog silently continues for unparseable IDs, duplicate IDs and missing compiled branches, returning only successful registration reports. Selection index, required flag, packaging, expected crate/artifact and missing reason are lost. The no-provider-feature test explicitly creates a required Sound selection and asserts that the result is an empty vector; the test therefore codifies the incorrect fail-open product behavior.

Provider snapshot tests verify only branches compiled into the current feature set. They cannot prove that every requested selection produced a resolution outcome, and they do not fail on profile/build-feature closure gaps.

### P1: coarse feature groups couple unrelated product costs

Enabling one base provider links all eight base crates; enabling one advanced rendering provider links Hybrid GI, Neural, Solari and Virtual Geometry. This pushes build time, binary size and platform dependencies into products that may not select those packages, while omitting the group removes required packages entirely. It encourages manual special-case features such as Navigation, UI Document Importer and Zr VM instead of one generated BuildSet.

### P1: source, generated export and native discovery remain separate authorities

The hand-written source catalog returns Rust registration reports, generated exports use provider function pointers and native discovery parses/loads artifacts. They do not share one requested/resolved/admitted/activated receipt, dependency closure or provider generation. Profiling one path cannot establish the shipped path's startup, memory or lifecycle cost.

## 4. Unreal source constraints

Unreal `FPluginManager::ConfigureEnabledPlugins()` keeps required-plugin truth false until compile-time, command-line, target and program plugin resolution succeeds. `LoadModulesForEnabledPlugins()` first requires that configuration result, then loads only enabled modules by an explicit loading phase. `AreRequiredPluginsAvailable()` reuses the same configuration truth.

Zircon need not copy Unreal's plugin manager, but it needs the same ordering: resolve build/target/profile/dependencies and required availability before activation, expose one enabled set, and phase lifecycle work. Returning an empty success collection for a missing required package is incompatible with that architecture.

## 5. Dependency-ordered plan

### M0: make every profile/build closure fail closed

Generate a BuildSet/provider matrix from synchronized manifests and `runtime-feature-presets.toml`. For each profile, prove that every required/default package is linked in the selected packaging form or is explicitly allowed externalized. Client2D/3D/Editor/Dev must not reach Ready without real Sound and Rendering providers. Optional missing packages remain visible but non-fatal.

Replace the empty-vector contract with one resolution receipt per input selection: disabled, target mismatch, duplicate/conflict, not linked, unsupported packaging, registration failed, admitted or activated. Required unresolved receipts are fatal. Update the current no-provider test to assert structured required failure, not disappearance.

### M1: generate fine-grained provider factories

Generate one immutable provider-factory table and per-product BuildSet rather than hand-maintaining fifteen branches and coarse 8/4 groups. Link only the factories required by the artifact's declared package closure. Selection invokes only selected factories; unselected linked providers perform no registration, task, IO or GPU/device work.

Factory lookup should be direct match/indexed and `O(selections)`. Keep provider construction lazy until selection/admission, then cache the immutable registration/provider generation for downstream consumers. Do not repeatedly rebuild reports at multiple bootstrap layers.

### M2: converge source, generated and native resolution

Feed linked factories, generated export providers and native artifacts into one `ResolvedProductPluginGraph`. Preserve package ID/version, packaging, capabilities, dependencies, module/system set, provider generation, loading phase, health and unload hooks. Unsupported contribution kinds reject admission.

Runtime and Editor consume the same immutable graph. Profile overlays may add policy intent but cannot silently invent a provider that the BuildSet does not contain or erase a required missing receipt.

### M3: startup/build performance qualification

For Minimal, Client2D, Client3D, Editor, Dev and Server, capture the resolved Cargo package/feature graph, binary size and linked provider count. On current-source executables, record cold/warm discovery, selection, registration, activation and first usable frame: CPU p50/p95/p99, allocations/bytes, module/system counts, DLL/file IO, RSS, waits, power and energy-to-ready.

WPR/ETW must show catalog resolution once per composition generation and zero inactive-provider/frame work. Compare source/generated/native packaging with the same selection set and require equivalent receipts before comparing timings.

## 6. Acceptance

1. All 30 manifest-declared Runtime packages have an explicit product classification; every production package is reachable through its declared supported packaging or reports why not.
2. Every selection produces one typed receipt. Missing required providers block Ready; no parse/provider/build-feature failure becomes an empty success vector.
3. All six profile BuildSets close their required/default packages and capabilities against the actual resolved Cargo/artifact graph.
4. Fine-grained generated factories replace coarse group-or-absent behavior. Unselected providers add no registration/runtime work and only declared BuildSet code cost.
5. Source/generated/native forms resolve equivalent package/provider generations, capabilities, dependencies, lifecycle and diagnostics.
6. Current-source Windows WPR/ETW evidence publishes startup, allocation, memory, IO and power data; stable frames perform zero catalog work.

## 7. Validation status

- Static per-Rust-file review: **5/5 complete** against current shared changes.
- Runtime provider catalog coverage: **15/30 (50%)**; fifteen packages need explicit product classification.
- Profile/build-feature closure: **failed statically** for required Sound/Rendering in Client2D/3D/Editor/Dev.
- Required missing behavior: **failed statically** and currently codified by an empty-vector unit test.
- `rustfmt --check --config skip_children=true`: **pass** for all 5 current files.
- Cargo/tests: **not run** because the managed Windows validation session is not executable.
- Current-source executables, WPR/ETW, startup, binary size, memory and power evidence: **pending**.
- No production code was changed; four shared modified files were preserved exactly.
- Source recheck is required before implementation because the catalog is under concurrent/shared modification.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.
