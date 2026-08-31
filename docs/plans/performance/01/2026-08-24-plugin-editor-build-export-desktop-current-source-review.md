---
title: Plugin Editor Build Export Desktop Current Source Review
date: 2026-08-24
scope:
  - zircon_plugins/editor_build_export_desktop
status: static_complete_product_and_dynamic_pending
canonical_owner:
  - docs/plans/optimize/zircon_plugins/03-desktop-export-native-window-source-dist-provider-integration-review.md
related_performance_owner:
  - docs/plans/performance/01/2026-08-22-editor-build-export-generation-and-background-job-architecture-review.md
references:
  - dev/UnrealEngine/Engine/Source/Editor/UATHelper/UATHelperModule.cpp
  - dev/UnrealEngine/Engine/Source/Developer/LauncherServices/Public/ILauncherProfile.h
  - dev/UnrealEngine/Engine/Source/Developer/LauncherServices/Private/Profiles/LauncherProfile.cpp
---

# Plugin Editor Build Export Desktop Current Source Review

## 1. Coverage

The current plugin Rust surface is **7/7 files**, **1,134 physical / 1,052 non-empty lines**, **47,860 bytes**, and **7 test markers**. Its workspace-relative `path + LF + decoded text + LF` SHA-256 is `906d17630fe0aa6e4eab21aced0184a0d81bb19672417702c02b4003b424d8ad`. The generated plugin manifest, both Cargo manifests, private UI/profile asset inventory, existing Editor build/export performance report and Plugins03 owner report were also checked. The plugin directory is clean in the shared worktree.

## 2. Runtime cost classification

This package does not execute build, cook, pack or export algorithms. It constructs small descriptor vectors at registration time, registers one view, seven command descriptors, three report templates, one asset type and one inspector customization, then re-exports the actual `zircon_editor` export session/job API. `export_wizard_descriptor()` allocates three bounded vectors and performs linear lookups over at most eight stages or three reports. These are startup/view-construction costs, not the MVP export bottleneck.

The dist is metadata-only: it has no command invocation, bridge method, state, unload or host-ready behavior. Its registration manifest declares no serialized extensions. The source plugin registers operation descriptors, but no event or operation factory in this package owns their execution. Optimizing descriptor construction would therefore improve a path that does not make the advertised plugin product executable.

## 3. Structural findings

1. **Duplicate product owner:** the core Editor already owns a BuildExport view, wizard session and job pipeline; the plugin declares a second view/command/template/profile surface. Stable identity, capability and presentation cannot converge while both remain owners.
2. **Descriptions without behavior:** all seven operation commands require invocation, but the package provides no factory/bridge. Source registration can describe actions that cannot execute.
3. **Native distribution is empty behavior:** NativeDynamic can pass ABI/manifest checks while contributing no executable Editor behavior. Schema conformance is not product availability.
4. **Default product reachability:** first-party Editor catalog wiring does not establish this source package as the unique selected provider. Package selection, published capability, visible core UI and actual execution are separate authorities.
5. **Profile/report data has no package owner:** the profile component/controller and report summary keys are registered here, while the actual pipeline and retained panel use core models. There is no single versioned request/report schema spanning authoring to execution.
6. **Downstream UI costs remain:** the related 9-file Editor report found UI-time metadata polling, duplicate wizard representations and non-virtualized target rows. The existing `EditorJobSystem` background boundary is sound and must be retained.

No production edit is justified inside this clean package. The first change must be Plugins03's owner decision and product-level failing fixtures, not local allocation work.

## 4. Unreal source constraints

Unreal's `ILauncherProfile` owns typed build configuration, cook mode, serialization and validation errors/warnings. `FUATHelperModule::CreateUatTask` validates the tool path, creates a serialized monitored process, binds output/cancel/completion/launch-failure callbacks and launches packaging outside the UI work itself. Cancellation stops the process tree and notification/result publication returns through TaskGraph work.

The transferable boundary is: a versioned profile/request model, explicit validation, one execution owner, monitored asynchronous process work, bounded event publication, cancellation and terminal receipts. Zircon already has much of the generic job boundary in `zircon_editor`; the desktop plugin must either provide a real target adapter to it or cease advertising an independently installable product.

## 5. Dependency-ordered plan

### M0: decide and enforce the owner

Choose either `core framework + desktop target provider plugin` or a fully builtin desktop export feature. Add product tests showing current source registration reachability, empty dist contribution, duplicate core/plugin entry and unowned operation invocation. Remove the losing view/command/template/profile authority rather than keeping aliases indefinitely.

### M1: one executable contribution

Generate source and dist contribution inventory from one declaration. Every operation must have exactly one event, in-process factory or ABI bridge owner before capability publication. A dist package must materialize a non-empty contribution bundle and bind behavior; otherwise it remains unavailable.

### M2: canonical profile and resolved request

Define one versioned desktop export profile consumed by the target provider. Validation produces typed diagnostics and a `ResolvedExportRequest` containing platform, build configuration, packaging strategy, asset filter, plugin/features and output policy. Remove hard-coded or unconsumed profile/report fields.

### M3: background execution and retained presentation

Preserve `EditorJobSystem` for process execution, cancellation, bounded output and terminal state. Complete the related Editor performance plan: watcher-owned source generations, zero stable UI filesystem polling, shared target/job/wizard generations and visible-range rows. Do not create a plugin-private worker pool.

### M4: product and performance qualification

Enable/disable/update must atomically add or revoke provider capability, commands, profile tools and views. Source and dist paths execute the same minimal project and produce equivalent typed receipts/artifact manifests. WPR measures UI thread, child process CPU/I/O, event queue/drain, RSS and energy on one current-source executable.

## 6. Acceptance

1. Package selection reaches one provider; package absence or failed admission publishes zero feature capability and zero plugin-owned UI.
2. Each of seven retained operations has exactly one executable owner; invocation returns accepted/progress/cancel/terminal receipts.
3. Source and dist contribution inventories and behavior results are equivalent; the dist contribution count is non-zero when declared available.
4. A real minimal project validates, builds, cooks, packs and emits a launchable desktop artifact. Placeholder reports or manifest-only DLLs do not pass.
5. Build/cook/pack work and filesystem scans consume zero editor-frame execution time; UI completion drain is bounded and cancellation kills the process tree.
6. Stable BuildExport refresh performs zero filesystem metadata calls and zero target/wizard reconstruction; scrolling is `O(visible rows)`.
7. Record cold/warm wall/CPU p50/p95/p99, child-process throughput, queue wait, output events/dropped bytes, cancellation latency, artifact bytes, allocations, peak RSS and energy. Compare with Unreal only on matched projects, targets, hardware and build modes.

## 7. Validation status

- Static per-Rust-file review: **7/7 complete**.
- Global plugin structure audit: **pass** for manifest/schema/registration/dist boundaries; it does not prove executable contribution or export behavior.
- `rustfmt --check`: **fail** on one import-order difference in clean `editor/src/plugin.rs`; no unrelated formatting edit was made.
- Cargo, source/dist product equivalence, real export, WPR/ETW, power and launch validation: **pending**.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.
