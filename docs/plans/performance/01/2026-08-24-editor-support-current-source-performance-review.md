---
title: Editor Support Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/editor_support
status: static_complete_product_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Editor/LandscapeEditor/Private/LandscapeEditorModule.cpp
---

# Editor Support Current Source Performance Review

## 1. Coverage

The current Rust surface is **1/1 file**, **334 physical / 316 non-empty lines**, **12,860 bytes** and **1 test marker**. Its workspace-relative `path + LF + decoded text + LF` SHA-256 is `6358d9b02b4d78041e797e620ab0b149537d8bc4bf7e139f938e865d763036a8`. The package directory is clean.

The helper's 21 consumer files and 54 call-site matches, `EditorPluginRegistrationReport`, package candidate registration, active extension materialization, manager activation/generation paths, Editor extension/store owner plans and the Unreal Landscape Editor lifecycle were cross-checked. These helpers run during provider registration or catalog-generation rebuild, not per frame.

## 2. Retained foundation

`register_authoring_extensions`, `register_authoring_contribution_batch` and `register_authoring_surface` centralize common descriptor construction and propagate validation errors. Registration order alone initially appears non-transactional, but `EditorPluginRegistrationReport::from_plugin` supplies a fresh `candidate_extensions` registry and publishes it only when the plugin callback succeeds. A package-local helper failure therefore discards the whole candidate; this behavior should be preserved.

The use of borrowed static surface metadata and move-consumed vectors is appropriate for startup-only construction. Replacing the `BTreeMap` registries or removing individual strings before product profiling would be premature.

## 3. Structural performance findings

### P0: Active can mean only partially materialized

The manager marks an eligible registration Active after lifecycle callbacks, then `build_active_extensions` clones every descriptor from all Active registrations into a fresh composite registry. Duplicate IDs or other cross-plugin materialization errors are appended to `EditorExtensionCatalogReport::diagnostics`, but successful contributions remain in the registry and the conflicting plugin stays Active.

The resulting product can expose some commands/views/assets from a plugin while dropping others. It also cannot attribute a rejected contribution to an activation receipt strongly enough to revoke the owner. Correctness, startup cost and reload behavior therefore depend on merge order.

### P1: every manager generation clones and revalidates the full active set

Providers first allocate vectors and insert descriptors into package-local `BTreeMap`s. Any manager generation then enumerates all active registrations, clones descriptors and inserts/validates them again in a new composite registry. Enable, disable, reload or phase changes rebuild the complete active contribution set rather than applying an owner-qualified delta.

The current two-provider catalog hides this cost. Once the missing first-party Editor catalog closure is repaired, the same path expands toward the 21 known helper consumer files and additional native providers. Its scale is approximately `O(total active contributions * log registry size)` plus descriptor/string/Arc clones per generation. This is a static complexity finding; no product allocation/timing measurement is claimed.

### P1: the shared batch is not a complete owner/lifecycle contract

`EditorAuthoringContributionBatch` carries ten descriptor families: commands, menu items, asset importers/types, inspector customizations, scene modes, graph editors/palettes and timeline editors/tracks. Drawer, UI template and view/surface registration use a second helper. Settings pages, UI pane data sources, viewport overlay providers, operation factories and runtime-event consumers use other paths.

The batch has no package owner ID, provider generation, loading phase, required Runtime generation, removal token or resource/lifecycle receipt. The manager must recover ownership from the outer registration and rebuild globally. Adding more vectors to this DTO without an owner-qualified store would preserve the same architecture.

### P1: registration and revocation are asymmetric

The helpers describe registration only. Plugin lifecycle callbacks report Loaded/Enabled/Disabled, but contributions are not mounted/unmounted through matching per-owner handles; the manager derives revocation by recomputing the whole active registry. Cross-plugin conflicts and a failed generation therefore lack atomic rollback to the previous complete composite generation.

## 4. Unreal source constraints

Unreal's `LandscapeEditorModule.cpp` registers commands, editor mode, widgets, detail customizations, property sections and file formats in `StartupModule`, and explicitly unregisters the corresponding owned registrations in `ShutdownModule`. It tracks registered property sections and file-format objects so removal targets the exact owner resources.

Zircon should not copy Unreal's global singleton registries. The transferable constraints are symmetric owner-qualified registration/revocation, phase-aware lifecycle and no published half-module. Zircon can improve on the reference by staging an immutable composite generation and atomically swapping it only after full validation.

## 5. Dependency-ordered plan

### M0: preflight one complete composite generation

Convert each plugin's candidate registry into an immutable, owner/generation-qualified `ContributionBatch`. Before changing manager state to Active, validate the complete candidate product graph: cross-owner ID conflicts, command/factory pairs, view/template dependencies, asset types, capabilities, Runtime provider generation and all supported contribution families.

Publish a new `EditorExtensionCatalogReport` only when the required candidate set is complete. On failure, retain the previous complete generation, keep the new provider non-Active/faulted, and attach diagnostics to the exact owner and contribution ID. No partial contribution from a failed generation may become visible.

### M1: incremental owner-indexed materialization

Store immutable batches behind `Arc` and index contributions by owner/generation. Enable/reload computes and validates only the changed owner's additions/replacements plus affected dependency edges. Disable/unload removes by owner token. Reuse unchanged descriptors and derived indices; do not clone and revalidate every active package on each generation.

The composite snapshot remains immutable for readers. Build off-thread only where validation is thread-safe and bounded; publication is one short generation swap. Editor frame code reads a stable snapshot and never performs registration, discovery or conflict resolution.

### M2: converge all contribution families and lifecycle

Replace the split helper/side-channel surface with one typed contribution bundle that includes views/drawers/templates, pane data sources, settings, overlays, operation factories and event consumers in addition to the existing ten families. Every entry carries stable identity, owner, generation, required capabilities/Runtime generation and loading phase.

Keep ergonomic constructors in `editor_support`, but make them builders for the canonical bundle rather than direct mutators of a live-style registry. Native/source/generated forms must materialize the same bundle or reject unsupported types.

### M3: quantify startup and reload cost

With a current-source Editor executable, measure cold startup and one-provider enable/disable/reload at fixed provider/contribution counts. Record preflight/materialization/swap CPU p50/p95/p99, descriptors visited/cloned/reused, allocations/bytes, lock/wait time, generation count, diagnostics, RSS and time-to-usable UI.

WPR/ETW must show zero registration/materialization work in stable frames and bounded work on reload. Fault injection must prove a late duplicate or invalid descriptor leaves the previous complete generation byte-for-byte/currentness equivalent.

## 6. Acceptance

1. `Active` implies every required contribution for that provider generation passed composite preflight and is visible; partial Active state is impossible.
2. A failed candidate publishes zero contribution and preserves the previous complete generation. Diagnostics identify owner, generation, family and contribution ID.
3. Enable/disable/reload touches the changed owner and affected dependency edges; unchanged batches are reused without full descriptor clones/revalidation.
4. All contribution families share one owner/generation/lifecycle bundle and have symmetric revocation. Source/generated/native forms are equivalent or fail admission.
5. Stable Editor frames perform zero catalog registration, merge or conflict work. Startup/reload publishes bounded timing/allocation/count telemetry.
6. Managed Windows tests and current-source WPR/ETW evidence pass before protected-ledger promotion.

## 7. Validation status

- Static per-Rust-file review: **1/1 complete**.
- Consumer surface: **21 files / 54 helper references** checked.
- Package-local candidate failure atomicity: **present statically** and retained.
- Cross-plugin composite atomicity: **failed statically** because conflicts yield diagnostics while partial registry content and Active state remain.
- Incremental generation materialization: **absent statically**; all active descriptors are cloned/revalidated into a fresh registry.
- `rustfmt --check --config skip_children=true`: **pass**.
- Cargo/tests: **not run** because the managed Windows validation session is not executable.
- Current-source Editor startup/reload, WPR/ETW, allocations, memory and power evidence: **pending**.
- No production code was changed; a helper-local clone transaction would add work and would not repair manager-level composite semantics.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.
