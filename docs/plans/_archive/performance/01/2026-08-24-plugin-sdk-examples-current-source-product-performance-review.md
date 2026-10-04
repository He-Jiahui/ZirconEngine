---
title: Plugin SDK Examples Current-Source Product Performance Review
date: 2026-08-24
status: static_complete_product_unavailable_dynamic_pending
scope:
  - zircon_plugins/plugin_sdk_examples
canonical_owners:
  - docs/plans/optimize/zircon_plugins/20-plugin-sdk-example-native-editor-fixture-test-carrier-artifact-isolation-product-truth-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Plugins/Tests/TestSamples/TestSamples.uplugin
  - dev/UnrealEngine/Engine/Plugins/Tests/TestFramework/TestFramework.uplugin
  - dev/UnrealEngine/Engine/Plugins/Tests/ModularTestFrameworkTests/ModularTestFrameworkTests.uplugin
---

# Plugin SDK Examples Current-Source Product Performance Review

## 1. Status and frozen scope

The Plugin SDK Examples package completed E3 current-source static review over **7/7 Rust files** at revision `8a5bd5580000debd99bdd96e437cc7bc017468a7`:

| Module folder | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| `zircon_plugins/plugin_sdk_examples/editor` | 6/6 | 498 / 462 | 19,264 | 3 / 0 | included below |
| `zircon_plugins/plugin_sdk_examples/dist` | 1/1 | 106 / 94 | 4,191 | 2 / 0 | included below |
| **Total** | **7/7** | **604 / 556** | **23,455** | **5 / 0** | `d9d0f8a2bb47a47564cee64757fd6062350d9686806c9606c705208a98f70e12` |

The fingerprint is SHA-256 over sorted `repository-relative-path|sha256(file-bytes)` rows joined by LF. Six files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; `editor/src/extensions.rs` has one import-order-only formatting mismatch. The package passes `git diff --check` and is clean. No format-only source churn was introduced.

Managed Windows Cargo remains unavailable, so none of the five tests ran. No current-source Editor containing this sample is launchable for WPR/ETW, and no real window/import/render workflow exists for RenderDoc. This scope is statically reviewed but is not an available SDK sample or product plugin.

## 2. Per-file review result

| Module | Reviewed files | Result |
|---|---|---|
| Declaration and IDs | `editor/src/{capability,extension_ids,lib}.rs` | Declares an experimental Editor package with three capabilities and source/library/native packaging, but no Sample/TestFixture role, visibility, explicit-load or Shipping exclusion. Native projection contains zero extensions. |
| Source contributions | `editor/src/extensions.rs` | Builds two views, four commands, three menu items, one Model importer descriptor, two asset-type contributions, two UI templates and one inspector customization. It implements none of the command, import, document, template or presentation behavior. |
| Package projection | `editor/src/plugin.rs` | Advertises absent `assets` and `examples` roots and all three packaging forms; source registration is only reachable by directly constructing this crate's plugin object. |
| Editor tests | `editor/src/tests.rs` | Proves descriptor strings in an isolated registry and packaging declarations. It does not resolve resources, dispatch commands, import a model, open/save a document, compare carriers or exercise product catalog selection. |
| Native dist | `dist/src/lib.rs` | Editor-only stateless descriptor with empty command/event manifests, no bridge, ready, state or unload callback; diagnostics explicitly say extension behavior remains in the source Editor module. |

The source contribution count and native `extensions: []` are mutually exclusive product contracts. The checked-in manifest also says `sdk_api_version = "0.2.0"` while the source package test expects `0.1.0`, and advertises importer `output_kind = "Data"` while source registration declares `ResourceKind::Model`.

## 3. Current product composition

`zircon_editor/build.rs:55-156` scans every first-level `zircon_plugins/*/plugin.toml` and publishes any package containing an Editor module into the generated Editor catalog. It reads only ID, display name, category, crate name and capabilities; it does not gate on maturity, role, visibility, enablement, packaging closure or resource existence.

The executable source-provider catalog at `zircon_plugins/first_party_editor_catalog/src/catalog.rs:33-46` maps only Navigation and Neural. It has no Plugin SDK Examples branch. Therefore the generated catalog can expose sample metadata while the executable contribution object is absent. The runtime manifest type defaults this package to `PluginPackageKind::Standard`; the only kinds are `Standard` and `FeatureExtension`.

This creates a four-stage false-positive path:

`directory exists -> generated builtin metadata -> product status/selection -> no source provider or equivalent native behavior`.

It adds avoidable manifest discovery, generated-catalog size, status/materialization work, native artifact build/load attempts and failure diagnostics to the MVP Editor path. More importantly, it prevents the build graph from proving that samples and destructive fixtures are absent from Shipping.

## 4. Structural performance findings

### P0: sample/fixture identity is missing from the package algorithm

`maturity = experimental` is descriptive metadata, not an artifact role. The package is a normal Standard plugin, discovered by physical location and eligible for the same source/library/native pathways as production packages. Adding more examples scales default catalog and qualification work with repository directory count rather than selected product capability.

Plugins20 M0/M1 must introduce a mutually exclusive role such as `Production | Sample | TestFixture | DeveloperTool`, plus visibility, default enablement, explicit load and target/configuration policy. Product catalogs are explicit allowlists over resolved package definitions. Samples default hidden, disabled and explicitly loaded; Sample/TestFixture are denied from Shipping artifact closure.

### P0: no advertised carrier provides the declared workflow

Source/Library has the descriptor construction code but no first-party product provider mapping. Native has a loadable ABI shell but publishes no extension, command or bridge. None of the forms can execute the weather window, model import, inspector, settings creation or document workflow. Carrier selection can only change which incomplete half is observed.

Choose one truth first: this package should be an explicitly imported SDK sample, not an MVP builtin. Then make one small golden workflow executable end-to-end and project the same behavior through source, library and native carriers. Availability follows a behavior receipt, not a package manifest row or non-null entry report.

### P1: unresolved resources and divergent schemas are admitted before work

The declared `assets` and `examples` roots do not exist. Neither `editor/model_inspector.zui`, `editor/model_import_settings.zui` nor `examples/model_import_settings.toml` exists. Source says the importer produces Model while generated manifest says Data; source and checked-in manifest disagree on SDK API version.

Canonical package compilation must resolve every required root/URI and compare declaration, generated manifest and carrier projections before catalog admission. Failure is deterministic and cached by source digest. Do not create placeholder resources or hand-edit the generated manifest: that would turn one visible mismatch into a false green product.

### P1: commands and importer are metadata without algorithms

Four operation paths have no package implementation or product handler. The importer registers extensions/output/priority but never reads, parses, validates or publishes glTF/GLB. The templates and inspector customization have no documents/controllers. These rows can create dead menu entries and failed dispatch, but cannot supply performance evidence for import or authoring.

The sample should reuse the canonical glTF/model import provider rather than implement a second parser. A golden workflow must bind operation factories, resolve resources, run one valid and one invalid import, publish a typed artifact, open the toolkit, edit/save/reopen a settings document and return typed receipts. Until then the contributions remain unavailable.

### P1: native packaging compiles through the full source Editor crate but carries no behavior

The `cdylib` depends on `zircon_plugin_sdk_examples_editor`, which depends on the full Editor and Runtime contracts, only to import constants and a serialized empty registration manifest. This expands compile/link dependency work without producing carrier parity. Static review cannot quantify final binary retention, but the dependency graph is unnecessary by construction.

After the carrier contract is selected, move declaration/schema constants into a narrow ABI-safe projection crate. Native callbacks must carry the executable host-safe contribution subset and explicitly reject unsupported behavior. Measure build timings, artifact size, DLL load/materialization and retained RSS before accepting parity.

### P1: existing tests optimize for local descriptor success

All five tests assert isolated DTO/ABI properties. They do not test role/shipping exclusion, product catalog visibility, missing roots, command execution, importer output, native/source parity, unload cleanup or failure rollback. This makes metadata generation cheap to validate while the costly product failure path remains untested.

Required RED coverage starts at the resolved product graph: the default MVP Editor has zero sample rows/providers/artifacts; explicit developer selection produces exactly one ready sample provider; missing resource or incomplete carrier fails before view/menu publication; unload revokes every contribution and callback generation.

### P2: this declaration scope is not a runtime hot path

There is no tick, query, parser, renderer or worker in these 604 lines. Import ordering and collection micro-tuning cannot affect frame time. The first performance optimization is removing unselected sample work from the MVP product graph; the second is making the explicitly selected workflow real and measuring its provider/import/render costs.

## 5. Unreal evidence and adopted policy

Unreal's current source provides explicit product/test separation:

- `PluginDescriptor.h:131-163` separates enabled-by-default, installed, hidden, sealed and explicitly-loaded policy from supported target platforms.
- `ModuleDescriptor.h:102-107,163,187,236` separates DeveloperTool modules, loading phase, configuration deny lists and compile-in-configuration decisions.
- `PluginManager.cpp:421-429,1742-1754` resolves enabled-by-default policy; lines 2185-2191 compile only modules allowed by target/configuration; lines 2909-2917 skip explicitly loaded plugins during normal phase loading; lines 3351-3718 own explicit mount/unmount.
- `TestSamples.uplugin` and `TestFramework.uplugin` set `EnabledByDefault: false`; the latter uses `DeveloperTool`. `ModularTestFrameworkTests.uplugin` additionally declares `TestPlugin: true` and denies Shipping.

Zircon should adopt these policy dimensions, not copy Unreal's JSON names. Repository discovery produces candidates only. A resolved ProductPluginCatalog selects packages by role/target/configuration and emits an eligibility receipt. Normal startup sees only selected production packages; developer tools mount samples explicitly; Shipping rejects fixture/sample identities at graph and final-artifact gates.

## 6. Required optimization sequence

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Carrier isolation | Add Sample/TestFixture identity and fail-closed default visibility/loading/Shipping rules; use an explicit product catalog. | Default MVP Editor contains zero SDK sample row/provider/resource/DLL; Shipping graph and artifact scan contain no sample/fixture marker. |
| M1 Canonical package compilation | One definition generates manifest/catalog/carrier projections; resolve roots/URIs and compare SDK/output/capability schemas. | Missing roots and current Data/Model or 0.1/0.2 drift fail before catalog publication with source-bound diagnostics. |
| M2 Executable golden sample | Explicitly imported sample reuses canonical model importer and implements one window/import/toolkit/settings vertical slice. | Valid/invalid import, open/edit/save/reopen and unload complete with typed receipts and no dead command/menu. |
| M3 Source/library/native parity | Narrow projection crate plus equivalent executable contributions, resources, callbacks and lifecycle for all declared forms. | Golden workflow produces the same capability/contribution/resource/artifact/state/unload receipt in each carrier. |
| M4 Dynamic performance qualification | Measure default-unselected and explicit-selected Editor builds and workloads. | Default sample overhead is zero; publish BuildSet-bound build/link time, artifact bytes, DLL load/materialization, startup CPU/RSS/wakeups, import latency/allocation/I/O and explicit window frame cost. |

## 7. Direct-fix decision and dynamic status

Simple local edits such as adding empty resources, wiring the sample into the first-party catalog, changing `Data` by hand or serializing descriptors without callbacks would each preserve the wrong product algorithm. Role/catalog isolation must precede behavior and carrier work, so this review makes no source change.

Static review is complete. Cargo, product-graph exclusion, resource compilation, carrier behavior, WPR/ETW, rendered workflow and power acceptance remain pending. No Git milestone commit or quantified WeCom notification is warranted.
