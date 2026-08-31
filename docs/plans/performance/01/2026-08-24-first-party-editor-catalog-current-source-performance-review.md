---
title: First-Party Editor Catalog Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/first_party_editor_catalog
status: static_complete_product_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
---

# First-Party Editor Catalog Current Source Performance Review

## 1. Coverage

The current Rust surface is **3/3 files**, **222 physical / 196 non-empty lines**, **7,832 bytes** and **6 test markers**. Its workspace-relative `path + LF + decoded text + LF` SHA-256 is `5c8c45029084969c33546a7c2414ef2d37c28adc6135a8d517d7aa5d743895d2`. The package directory is clean.

`catalog.rs`, `lib.rs`, `tests.rs`, Cargo features/dependencies, App feature composition and startup call site, `ProjectPluginManifest::enabled_for_target`, `cargo-zircon` scaffold/check logic, all top-level plugin manifests with Editor modules, Plugins06/08 and Runtime42 owners were checked. The catalog is called once while preparing Editor startup; it is not polled per frame.

## 2. Current performance truth

The local projection algorithm is reasonable for its current size. It performs one lazy linear scan over enabled selections, preallocates one `HashSet` and result `Vec`, deduplicates plugin IDs, and performs two feature-gated provider comparisons. This is startup-only work. Removing the ID clone or replacing the two branches with a map would not materially improve the MVP.

The App microbenchmark runs 21 samples of 1,024 resolutions and accepts P95 up to 250,000 microseconds per sample, roughly 244 microseconds per one-item resolution. It is a test-only coarse guard, includes repeated registration construction/assertions, has not run in this review and does not measure full Editor startup, allocations or provider activation. It must not be cited as product timing evidence.

## 3. Structural bottlenecks

### P0: the static Editor product catalog closes only 2 of 25 declared packages

Twenty-five top-level `plugin.toml` files declare an Editor module and the workspace contains forty Editor crates, including feature children. The static catalog has optional dependencies and registration branches only for Navigation and Neural. `zircon_app`'s `target-editor-host` likewise enables only those two Editor provider features. Fixtures/examples should remain profile-excluded, but production packages such as Rendering, Sound, Physics, Animation, AI, Texture, Terrain, Material Editor and authoring tools are also absent from this source product path.

The result is not just missing UI. Runtime selection, Editor capabilities, tools/documents/commands, native discovery and generated export can observe different provider sets. Work is then repeated through multiple composition authorities, while the visible capability set depends on how the executable was built rather than one resolved project/profile graph.

### P0: unresolved selections disappear instead of failing closed

The catalog silently continues when a selected ID cannot be parsed, is not compiled into the catalog or lacks a registration branch. The return type is only `Vec<EditorPluginRegistrationReport>`; it loses selection index, `required`, packaging, requested crate/artifact, target reason and missing-provider reason. A required Editor plugin can therefore vanish before startup readiness is evaluated, and an optional missing provider cannot be diagnosed accurately.

Deduplication also occurs before a receipt is created. Duplicate selections correctly avoid duplicate registration, but the product has no canonical conflict/override decision to report. Performance data cannot distinguish intentionally disabled work from missing compiled code.

### P1: scaffolding and validation preserve the incomplete catalog

`cargo-zircon plugin new` wires the Editor catalog only when `PluginKind::Editor` is selected. A system package that legitimately owns both Runtime and Editor modules is wired only into the Runtime catalog. This matches most of the current missing production providers.

`cargo-zircon plugin check` validates features, optional dependencies and registration branches already present in the Editor catalog, but it does not compare all manifest-declared Editor modules against the compiled provider set. The incomplete 2/25 state is therefore internally consistent and passes this class of check. Adding more hand-written `if` branches would increase drift and lookup work without repairing the generator authority.

### P1: three composition authorities prevent one measurable startup path

The hand-written first-party source catalog, generated export function pointers and native discovery/load each materialize providers differently. They do not share one requested/resolved/admitted/activated receipt, provider generation or lifecycle. Editor startup can measure catalog projection while native discovery and extension activation remain outside the same cost/health record.

The product should resolve selections once, then feed the same immutable result to Runtime and Editor. Otherwise later optimizations risk profiling one path while shipping another.

## 4. Unreal source constraints

Unreal `FPluginManager::ConfigureEnabledPlugins()` starts with required-plugin truth false, resolves compile-time/command-line/target/program plugins, and only marks all required plugins available after the configuration phases succeed. `LoadModulesForEnabledPlugins()` refuses to treat loading as successful when configuration fails, then loads enabled modules by explicit `ELoadingPhase`; `ModuleDescriptor.h` also makes current-configuration eligibility and load/unload phases first-class.

Zircon need not copy Unreal's class layout or module count. The transferable constraints are: resolve target/profile/dependencies before activation, fail required omissions, preserve explicit loading phase/lifecycle, and expose one enabled set. Silent `continue` from a required selection violates those constraints.

## 5. Dependency-ordered plan

### M0: generate complete catalog truth

Derive Editor provider candidates from synchronized plugin declarations/manifests and product buildset/profile policy. Classify every one of the 25 declared packages as linked, generated, native-only, unsupported, fixture/example-only or intentionally excluded. Extend `cargo-zircon check` to fail when a production Editor module has no reachable provider or explicit exclusion reason.

Do not enable all providers by default. Build features determine which provider factories are linked; the project/profile manifest determines which linked factories are selected. Unselected providers create no registration, task, view, resource or per-frame cost.

### M1: replace silent vectors with a typed resolution receipt

Produce one `EditorPluginResolutionReport` per input selection containing selection identity/index, required flag, target/profile, requested packaging/crate/artifact, provider source, provider generation, and status such as disabled, target mismatch, not linked, unsupported packaging, registration failed, admitted or activated. Required unresolved entries block Editor Ready; optional entries remain visible as unavailable.

Deduplicate by canonical package identity after conflict policy is applied. Keep one immutable provider factory table generated at build time, using direct indexed/match lookup or a prebuilt map. Resolution remains `O(selections)` with bounded startup allocations and zero frame polling.

### M2: converge source, generated export and native loading

Feed source-linked factories, generated export providers and native discoveries into one `ResolvedProductPluginGraph`. Runtime and Editor consume the same selected package/generation/dependency closure. Activation is phased and transactional; unload/hot update revokes the generation and contributions symmetrically.

Capabilities are published only from admitted/activated registrations, never from manifest metadata alone. Editor authoring providers must bind their required Runtime provider generation or report the exact unavailable dependency.

### M3: startup performance qualification

Measure current-source Editor cold/warm startup with fixed project/profile/provider sets. Record discovery, selection resolution, dependency closure, registration, activation and first usable frame separately: CPU p50/p95/p99, allocations/bytes, DLL/file IO, main/worker waits, loaded module count, registrations/contributions, RSS, power and energy-to-ready.

WPR/ETW must show catalog resolution occurs once and no inactive provider performs frame work. The existing 21x1,024 microbenchmark can remain a unit guard only after tightening it around deterministic factory lookup; it cannot replace end-to-end startup evidence.

## 6. Acceptance

1. Every manifest-declared Editor module has a generated reachable provider or an explicit product exclusion reason; fixtures/examples remain excluded without disappearing from diagnostics.
2. Every selection produces one resolution receipt. Missing required providers block Ready; no parse/provider/packaging failure is silently dropped.
3. Source-linked, generated and supported native forms resolve the same package ID, capability set, provider generation, dependencies and lifecycle result.
4. Runtime and Editor consume one immutable resolved graph. Provider resolution runs once per composition generation and performs zero per-frame polling or inactive-provider work.
5. Startup lookup scales linearly with selections plus linked providers, uses bounded allocations, and publishes phase timings/counts. No hand-written branch count is used as the product-performance target.
6. Current-source WPR/ETW and Editor first-frame evidence pass on a managed Windows build before protected-ledger promotion.

## 7. Validation status

- Static per-Rust-file review: **3/3 complete**.
- Catalog closure: **failed statically**, 2 linked top-level providers versus 25 packages declaring Editor modules; **23 require explicit product classification**.
- Silent missing/required resolution: **failed statically** because parse/provider misses are dropped from a vector-only result.
- `cargo-zircon` completeness enforcement: **failed statically**; it validates existing wiring but not manifest-to-Editor-catalog coverage.
- `rustfmt --check --config skip_children=true`: **pass** for all 3 Rust files.
- Cargo/tests and the App microbenchmark: **not run** because the managed Windows validation session is not executable.
- Current-source Editor executable, WPR/ETW, first usable frame, memory and power evidence: **pending**.
- No production code was changed; complete generation/resolution requires coordinated Plugins06/App/Runtime changes rather than more hand-written branches.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.
