---
title: Sound Plugin Assembly and Capability Current-Source Performance Review
date: 2026-08-24
status: static_complete_dynamic_pending_no_source_change
scope:
  - zircon_plugins/sound/runtime/src/{capability,components,config,lib,module,plugin,poison_recovery}.rs
  - zircon_plugins/sound/runtime/src/package
  - zircon_plugins/sound/runtime/src/runtime_plugin
canonical_owners:
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/48-runtime-wide-state-next-state-transition-hook-history-schedule-scope-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/58-runtime-plugin-interface-bridge-slot-generation-strong-weak-native-vm-lifecycle-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerModule.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerDevice.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AudioDevice.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AudioPluginUtilities.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
---

# Sound Plugin Assembly and Capability Current-Source Performance Review

## 1. Status and frozen scope

The Sound root/package/runtime-plugin assembly slice completed E3 current-worktree static review over **15/15 Rust files** at revision `2a1299f8bf8e5a3012860ff07a6fcf528e4721d8`:

| Module slice | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| plugin assembly, options, capability and module registration | 15/15 | 741 / 692 | 29,521 | 0 / 0 | `a24bd5618b5c86bcc59e9083b89f6ae18128ef64083ae1c33c1451def3282cfd` |

All files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; the scope is clean and scoped diff check passes. Managed Windows Cargo, a current-source plugin-selection/startup executable, native package load and ETW/power evidence remain unavailable. No source changed.

## 2. Per-module review ledger

| Module | Reviewed files | Static result |
|---|---|---|
| root/capability/component/config | 7 files | Runtime capability is marked Partial, but component schema and defaults expose advanced unavailable features. Project options are converted to `SoundConfig` only by helpers, not module construction. |
| `package` | 5/5 | Dependencies/options/events/components are rebuilt as owned vectors/strings. Defaults enable unsupported paths and `sound.enabled` is absent. |
| `runtime_plugin` | 3/3 | Descriptor declares two optional feature bundles and native distributions whose current-source provider crates are absent. Provider/applied status is not coupled to component/options registration. |

## 3. Structural findings

### P0: project plugin options do not construct the registered Sound manager

`SoundConfig::from_plugin_options` exists, but the module manager factory calls `DefaultSoundManager::from_weak_core`, which delegates to `SoundConfig::default()`. Repository-wide production scan found no path that injects the selected project's `SoundPluginOptions` into this manager. The manifest can therefore show user/project choices while runtime starts with default backend/rate/capacities/features.

This also makes performance comparison invalid: build/project receipts cannot explain the actual manager configuration. Module construction must receive an immutable resolved plugin-config generation, validate it against the selected backend/providers, and publish requested/applied/last-good values.

### P0: capability/schema/defaults advertise functions that the product cannot execute

The always-registered component schema exposes HRTF profile, occlusion, convolution sends, Doppler, audio volumes and mixer targets. Package defaults set convolution, timeline integration and dynamic events to true. Earlier complete source reviews established:

- spatial/environment has no production render consumer and current advanced algorithms are not acceptable;
- active Kira automation returns Unsupported;
- dynamic events have no product consumer and their enable flag is unused;
- convolution/HRTF/ray budgets and default preset fields have no production config consumer.

`SoundConfig` field-use review confirms `hrtf_enabled`, `convolution_enabled`, `convolution_budget`, `ray_tracing_quality`, `default_mixer_preset`, `timeline_integration` and `dynamic_events_enabled` are only assigned/exposed in config/options/tests, not applied by production runtime. Component properties and Editor authoring must be capability/provider-profile filtered; unavailable advanced paths cannot be default-enabled.

### P0: optional feature distributions point to absent current-source providers

The descriptor publishes timeline-animation and ray-traced-convolution feature bundles with runtime/editor/native crate names and NativeDynamic distribution entries. No matching provider directory or Cargo manifest is present in the repository; `zircon_plugins/sound/dist` is empty. These bundles may be future package contracts, but they are not a launchable current-source capability and cannot enter Ready/default packaging.

Selection must resolve a concrete provider artifact with ABI/version/platform/capability receipt before exposing the feature. Missing providers remain Unavailable and their editor modules/options are hidden or read-only with diagnostics.

### P0: disable semantics are incomplete

`SoundConfig.enabled` is checked by output lifecycle, but `sound_options()` does not declare `sound.enabled`, so the package configuration surface cannot set it through the same manifest. Advanced feature booleans are declared but not enforced. One applied config transaction must govern module/service activation, option visibility, queue admission and device start; a disabled Sound plugin must not initialize output or accept background work.

### P0: poisoned state is treated as valid state

`lock_recover` claims to recover the last valid state but simply calls `PoisonError::into_inner`. A panic may occur midway through graph, source, device, timeline or plugin registry mutation. Continuing all public operations on that state can amplify invariant damage, trigger retries and produce unpredictable CPU/resource behavior.

The manager needs immutable last-good generations and a supervisor failure state. Poison/panic transitions to Failed/Recovering, rejects or queues bounded commands, rebuilds from last-good where supported and publishes diagnostics. It must not silently bless the partially mutated guard.

### P1: manifest and registry construction repeatedly allocates immutable metadata

Components, options, event catalogs, dependencies, feature bundles and package manifests create new vectors and owned strings on every helper call. `package_manifest`, `runtime_selection`, `plugin_registration` and explicit `register` can reconstruct overlapping metadata. This is startup/tooling cost, not the current MVP bottleneck, but immutable declaration data should be generated once and shared by manifest, selection and registration to prevent drift and measure exact bytes/time.

Do not prioritize `OnceLock` caching until the capability/config contract is unified; caching contradictory manifests would make drift persistent rather than correct.

### P1: package and module dependency models are not one resolved graph

The package declares asset/scene required plus optional ray-query/timeline dependencies. The module descriptor depends on asset and its local driver, while optional feature manifests use different provider/capability identities (`physics` raycast and `animation` event track). The runtime component registration is unconditional. Multiple partially overlapping graphs increase selection work and allow package-ready/module-unavailable combinations.

Compile one resolved dependency/capability graph per project/target. Module startup receives that graph; component schema, options, providers and distribution artifacts derive from the same generation.

## 4. Positive baseline retained

The main Sound capability is explicitly `Partial`, the expensive manager is Lazy, and output startup checks `config.enabled`. These are useful foundations. The immediate driver is stateless; static review found no device creation in module registration itself. The plan preserves this lazy shape while making config/provider truth authoritative.

## 5. Unreal-primary policy adopted

- `AudioMixerModule.cpp:12-15` explicitly loads required AudioMixerCore and SignalProcessing modules at module startup.
- `AudioMixerDevice.cpp:1277-1305` derives plugin initialization from actual source count/sample rate/buffer/device state, initializes only valid provider interfaces, then initializes the source manager.
- `AudioMixerDevice.cpp:1855-1895` requires a valid reverb provider/effect and disables plugin processing when the provider contract fails.
- `AudioDevice.cpp:156-180` distinguishes available spatial plugins from the active plugin rather than presenting all authored choices as active.
- `AudioMixerDevice.cpp:1366-1392,1747` unregisters device listeners and pumps final source/plugin commands during shutdown.
- `ModuleManager.h:296-325,531` and `ModuleManager.cpp:1316-1344` provide explicit unload callbacks/state before releasing code.

Zircon adopts provider-resolved capability, actual-device initialization and supervised shutdown. It still measures its own startup/memory/power rather than inheriting Unreal numbers.

## 6. Required optimization plan

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Capability truth | Current base/MVP/optional provider matrix with Unavailable/Partial/Ready status. | Component schema, options, Editor and runtime expose the same applied provider generation. |
| M1 Config injection | Project selection resolves/validates `SoundConfig` and injects it into module manager creation. | Requested/applied/last-good config receipt matches runtime device/graph/capacities. |
| M2 Default hard cutover | Unsupported convolution/timeline/events/ray/HRTF paths default unavailable/off. | Enabling without a concrete provider fails before persistence/startup; no false Ready. |
| M3 Provider packaging | Feature bundles resolve existing source/library/native artifacts and ABI/platform dependencies. | Every selected artifact exists, loads and registers; missing provider never reaches export. |
| M4 Lifecycle/failure | Disable, reload, poison, provider failure and shutdown are supervised transitions. | No work accepted while disabled; panic never continues partial state; unload drains all callbacks/commands. |
| M5 Immutable declaration | One generated/cached metadata generation feeds manifest, registry and selection. | Metadata parity tests; startup allocation/time recorded without duplicate reconstruction. |
| M6 Dynamic qualification | Current-source base and selected provider startup/use/reload/disable/export. | Record startup P50/P95, config resolution, allocations/RSS, threads/handles, CPU idle/load, wakeups, device latency, unload time and power. |

## 7. Direct-fix decision

No production edit is made. Flipping defaults or caching vectors independently would leave module config injection, provider truth and lifecycle unresolved and could break project manifests without Cargo. The first implementation change must make resolved project config/provider generation an input to the manager factory, with tests proving requested/applied behavior.

Static review is complete only for these 15 files. Dynamic acceptance, a Git milestone commit and quantified WeCom notification are not warranted.
