---
title: Editor Contribution Fixture Current-Source Product Performance Review
date: 2026-08-24
status: static_complete_shared_change_preserved_dynamic_pending
scope:
  - zircon_plugins/editor_contribution_fixture
canonical_owners:
  - docs/plans/optimize/zircon_plugins/20-plugin-sdk-example-native-editor-fixture-test-carrier-artifact-isolation-product-truth-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Plugins/Tests/TestFramework/TestFramework.uplugin
  - dev/UnrealEngine/Engine/Plugins/Tests/ModularTestFrameworkTests/ModularTestFrameworkTests.uplugin
---

# Editor Contribution Fixture Current-Source Product Performance Review

## 1. Status and frozen scope

The Editor Contribution Fixture completed E3 current-worktree static review over **1/1 Rust file** at revision `ad677990bd85466771a90096b646526a3daf0837`:

| Module folder | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| `zircon_plugins/editor_contribution_fixture/native` | 1/1 | 145 / 139 | 5,596 | 0 / 0 | `6f69d39ef5880c5a08c7be432da925aea57f4604fe8b627180b25bce22dae443` |

The fingerprint is SHA-256 over sorted `repository-relative-path|sha256(file-bytes)` rows joined by LF. The Rust file passes standalone `rustfmt --check --edition 2021 --config skip_children=true` and the package passes `git diff --check`.

The file contains a shared worktree change from native callback byte-slice/status V2 types to V3 types. This review preserves that change and binds evidence to current bytes; it does not claim or modify ownership. The change does not alter the fixture's command/contribution behavior.

`python -B tools/tests/test_editor12_editor_contribution_fixture_contract.py` ran **3/3 tests** successfully with bytecode generation disabled. These are static source/manifest/JSON tests. Managed Windows Cargo is unavailable, so the cdylib was not built or loaded and there are no Rust behavior tests. No current-source Editor exists for WPR/ETW or rendered fixture work for RenderDoc.

## 2. Per-file review result

| Module | Reviewed file | Result |
|---|---|---|
| Native declaration, ABI and payload | `native/src/lib.rs` | One experimental Standard package publishes a six-row serialized contribution batch, an empty V4 command table, an always-DENIED command callback and an OK unload callback. It has no view/data factory, setting provider, asset provider, state or host-ready behavior. |

The three-file package also contains a cdylib Cargo manifest and generated `plugin.toml`. The manifest declares a normal Editor module, native-dynamic packaging and no TestFixture role, visibility, explicit-load or Shipping exclusion.

## 3. Verified host path

The current host path proves the command mismatch end to end:

1. `native/src/lib.rs:43-49` declares command-manifest schema V4 with `commands = []`.
2. Lines 51-101 publish view, drawer, menu, command, asset type and settings page. The menu targets `editor.contribution_fixture.open` and the contribution batch publishes that command.
3. `zircon_editor/src/core/plugin/materializer.rs:101-115` maps serialized menu/command rows directly to normal Editor operation descriptors. It does not bind a native behavior callback or command slot.
4. `zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs:116-185` builds the native command table only from the V4 manifest. Lines 268-288 reject names absent from that table before FFI invocation.
5. If reached by another path, fixture lines 135-140 return DENIED for every slot.

The visible menu/command is therefore non-executable by construction. The static Python tests only parse the contribution JSON and assert its six kinds; they do not cross-check the command table, build/load the DLL or dispatch from Editor UI.

## 4. Structural performance findings

### P0: a test fixture is admitted as a normal product candidate

Like Plugin SDK Examples, this first-level package has an Editor module and is discovered by `zircon_editor/build.rs` regardless of role or maturity. `PluginPackageKind` has no TestFixture variant, so the package defaults to Standard and can enter product catalog/status/artifact work.

Plugins20 M0/M1 must make TestFixture hidden, disabled, explicitly selected and denied from Shipping. The default MVP Editor must perform zero manifest row, native discovery/load, materialization, menu/view registration or retained-state work for this fixture. A test build may opt in through a source-bound fixture inventory and artifact variant receipt.

### P1: contribution admission and behavior admission are separate algorithms

The host validates/materializes the serialized contribution batch independently from the V4 command table. A candidate can therefore publish a command/menu without an executable slot. Late user dispatch then fails, after catalog, DLL, parse, registry clone/materialization, menu construction and interaction costs have already been paid.

Editor50/Plugins01 must compile one atomic `NativeEditorContributionPlan`: every executable contribution references a validated owner/generation-qualified behavior binding; menu visibility depends on command readiness; unsupported rows reject the whole candidate before publication. Unload revokes contribution and callback leases together.

### P1: view/drawer rows have no presentation provider

The materializer creates descriptor-only view/drawer entries. The fixture provides no view factory, pane data source, template/resource or render contract. Opening such a surface cannot verify native presentation and may leave empty/fallback UI that still participates in layout, command routing and retained state.

Either add the minimum host-safe factory/data callback contract and test a real frame, or remove view/drawer from the fixture's supported kinds. Metadata-only materialization must be named and kept out of user-visible availability.

### P1: settings and asset rows cannot execute a lifecycle

The settings row has no schema/default/read/write/scope/restart/persistence provider. The asset type has no serializer, importer, document, creation, thumbnail provider or unload migration. Publishing them tests only registry DTO decoding, not a usable native extension.

Plugins20 G28/G29 must require one typed setting roundtrip and one minimum asset lifecycle, or remove these rows. Each provider is owner/generation qualified and revoked atomically on unload/reload.

### P1: unload success is not fixture cleanup evidence

The unload callback returns OK without owning state. Static materialization removes a registry from a temporary package map and later merges descriptors into Editor registries; this fixture has no end-to-end assertion that disabling/reloading revokes view, drawer, menu, command, asset type and settings rows or fences stale callbacks.

The acceptance test must load the real DLL, materialize through the product manager, dispatch behavior, disable/unload, and prove every owner lease disappears. Reload must replace generation without invoking old function pointers.

### P1: current validation measures source text, not artifact behavior

The three passing Python tests confirm workspace strings, macro fields and JSON shape. They do not compile the current shared V3 signature change, validate ABI layout, parse the command table with the loader, materialize with the Editor, dispatch, collide, deny capabilities, reload or unload.

Keep the static test as a cheap schema smoke test. Product qualification requires a real cdylib and the same loader/materializer/operation dispatcher used by Editor. Negative artifacts receive distinct identities and cannot enter production/export catalogs.

### P2: the fixture has no recurring algorithm to micro-optimize

The 145-line package performs constant-size descriptor publication and constant-time denied callbacks. Its performance value is as a carrier/lifecycle test, not as an MVP feature. Optimizing JSON literals or registry choice here would not improve frame time; excluding it from the default graph and making explicit test behavior truthful are the relevant optimizations.

## 5. Unreal evidence and adopted policy

Unreal separates test/developer modules from normal product loading:

- `PluginDescriptor.h:131-163` exposes enablement, hidden and explicit-load policy independently from plugin discovery.
- `ModuleDescriptor.h:102-107,163,187,236` makes DeveloperTool, loading phase, configuration deny and compile eligibility explicit.
- `PluginManager.cpp:2909-2917` does not phase-load explicitly loaded plugins; lines 3351-3718 own explicit mount/unmount.
- `TestFramework.uplugin` is disabled by default and uses a DeveloperTool module. `ModularTestFrameworkTests.uplugin` is disabled, tagged as a test plugin and denied in Shipping.

Zircon should adopt the policy boundary rather than copy field names: candidate discovery is not activation; fixture artifacts are explicitly selected by test builds; behavior readiness precedes UI contribution publication; one owner receipt controls mount, callbacks, contributions and reverse-order cleanup.

## 6. Required optimization sequence

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Fixture isolation | TestFixture role, hidden/disabled/explicit load, Shipping denial and explicit test inventory. | Default MVP Editor and Shipping artifact contain zero fixture row/module/symbol/load/materialization work. |
| M1 Atomic contribution/behavior plan | Cross-validate serialized contribution rows with V4 command/provider bindings before publication. | Current empty-table/nonempty-command carrier is rejected before menu/view publication with typed diagnostic. |
| M2 Executable command and presentation | Dense command slot plus native operation adapter; minimum real view/drawer factory/data behavior. | UI dispatch reaches the real DLL and produces bounded typed output; surface presents a real provider frame or unsupported rows are removed. |
| M3 Settings/asset truth | Typed settings lifecycle and minimum asset provider workflow, or narrower declared fixture scope. | Read/write/persist/reload and asset create/load/save/unload paths execute through product owners. |
| M4 Real artifact lifecycle | Build/load the current cdylib through product manager; collision, denial, disable, unload and generation replacement. | All six contribution kinds and callbacks revoke atomically; stale calls fail without old-code execution. |
| M5 Dynamic qualification | Measure default-excluded and explicitly loaded test builds. | Default overhead is zero; publish BuildSet-bound DLL bytes/load time, parse/materialization/dispatch/unload latency, CPU/RSS/allocation/wakeups and retained-row counts. |

## 7. Direct-fix decision and dynamic status

Adding a command table row alone would still publish a test fixture into the product graph and leave view/settings/asset rows false. Removing rows would defeat the carrier coverage before the atomic behavior contract exists. The required first change is shared package-role/catalog policy, so this review does not edit the shared-modified source.

Static review and the three source-level Python tests are complete. Cargo, real DLL, product materialization/dispatch, unload/reload, WPR/ETW and power acceptance remain pending. No Git milestone commit or quantified WeCom notification is warranted.
