---
related_code:
  - zircon_runtime/src/ui/template/asset/resource_ref/resolver.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/resolution_report.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/mod.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/core/resource/registry.rs
  - zircon_runtime/src/core/resource/manager/resource_manager.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_ref.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_kind.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/fallback_policy.rs
  - zircon_runtime_interface/src/resource/locator.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime_interface/src/resource/untyped_handle.rs
  - zircon_runtime/src/ui/tests/asset_resource_resolver.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/mod.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/resource_ref/resolver.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/resolution_report.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/mod.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/tests/asset_resource_resolver.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/mod.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture plan code
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
tests:
  - zircon_runtime/src/ui/tests/asset_resource_resolver.rs
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\resource_ref\resolver.rs zircon_runtime\src\ui\template\asset\resource_ref\resolution_report.rs zircon_runtime\src\ui\template\asset\resource_ref\mod.rs zircon_runtime\src\ui\template\asset\mod.rs zircon_runtime\src\ui\template\mod.rs zircon_runtime\src\ui\tests\asset_resource_resolver.rs zircon_runtime\src\ui\tests\mod.rs
  - git diff --check -- zircon_runtime/src/ui/template/asset/resource_ref/resolver.rs zircon_runtime/src/ui/template/asset/resource_ref/resolution_report.rs zircon_runtime/src/ui/template/asset/resource_ref/mod.rs zircon_runtime/src/ui/template/asset/mod.rs zircon_runtime/src/ui/template/mod.rs zircon_runtime/src/ui/tests/asset_resource_resolver.rs zircon_runtime/src/ui/tests/mod.rs docs/zircon_runtime/ui/template/asset/resource_ref/resolver.md .codex/sessions/20260612-0904-editor-ui-architecture-implementation.md
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\resource_ref\resolver.rs zircon_runtime\src\ui\template\asset\resource_ref\mod.rs zircon_runtime\src\ui\template\asset\hot_reload_executor.rs zircon_runtime\src\ui\template\asset\mod.rs zircon_runtime\src\ui\template\mod.rs zircon_runtime\src\ui\tests\asset_resource_resolver.rs zircon_runtime\src\ui\tests\asset_hot_reload_executor.rs (2026-06-12: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-resource-refresh-0612-coremin --message-format short --color never (2026-06-12: passed, existing warnings only)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\resource_ref\resolver.rs zircon_runtime\src\ui\tests\asset_resource_resolver.rs (2026-06-20: passed)
  - git diff --check -- zircon_runtime/src/ui/template/asset/resource_ref/resolver.rs zircon_runtime/src/ui/tests/asset_resource_resolver.rs docs/zircon_runtime/ui/template/asset/resource_ref/resolver.md (2026-06-20: passed with LF/CRLF warnings only)
doc_type: module-detail
---

# UI Resource Resolver

`UiResourceResolver` is the consumer-level resolver for template resource references in the 05 UI asset-management plan. It sits after the path-level `UiResourcePathResolver`: path resolution proves whether a referenced file can be found under configured roots, while this resolver asks the runtime resource manager whether that reference has become a registered runtime resource.

The current implementation resolves to `UntypedResourceHandle`. It does not yet produce icon atlas slots, GPU texture views, or concrete font ids. Those renderer-facing handles belong to later icon, texture, and font consumer layers. This slice gives those layers a stable no-panic lookup surface and diagnostic model.

## Inputs And Outputs

The input is `UiResourceRef` from `zircon_runtime_interface`. It carries:

- `kind`: template-level resource kind (`font`, `image`, `media`, or `generic_asset`).
- `uri`: the primary resource URI.
- `fallback`: optional placeholder policy and fallback URI.

The output is `UiResolvedUiResource`:

- `Handle { handle, uri }` when the runtime resource registry contains the URI and its registered `ResourceKind` matches the expected UI kind.
- `Placeholder { handle, diagnostic_index }` when the primary resource cannot be used. `handle` is `Some(...)` only when a placeholder fallback URI resolves successfully.

The diagnostic index points into `UiResourceResolver::diagnostics()`. Callers can render a visible placeholder while still showing the authoring problem in editor panels or logs.

## Runtime Registry Lookup

The resolver uses `ResourceManager::registry().get_by_locator(...)` and never reads the file system directly. That keeps template consumption aligned with the runtime asset facade instead of creating a second UI-only IO path. `UiResourceResolverSchemeMap` is the only URI translation layer in this resolver; it maps UI template schemes to runtime `ResourceLocator` schemes before the registry lookup, but it still requires the target record to already exist in the resource manager.

UI kind mapping is intentionally small:

- `UiResourceKind::Font` expects `ResourceKind::Font`.
- `UiResourceKind::Image` expects `ResourceKind::Texture`.
- `UiResourceKind::Media` and `UiResourceKind::GenericAsset` expect `ResourceKind::Data`.

If the registry contains a record with the wrong kind, the resolver emits `KindMismatch` and refuses to return the mismatched handle. This prevents a missing icon or font from accidentally consuming an unrelated asset payload.

## URI Scheme Boundary

`UiResourceRef::validate(...)` accepts `res://`, `asset://`, and `project://`. The runtime core `ResourceLocator` currently supports `res://`, `lib://`, `package://`, `builtin://`, and `mem://`.

The resolver treats this as a boundary unless the runtime host supplies an explicit `UiResourceResolverSchemeMap`:

- `res://` and other `ResourceLocator` schemes can be looked up directly in the runtime registry.
- Without a scheme map, `asset://` and `project://` remain valid UI template schemes that produce `MissingPrimary` or `MissingFallback`, not `InvalidUri`.
- `UiResourceResolverSchemeMap::asset_to(...)` maps `asset://path#label` to the configured runtime scheme while preserving the path and optional label, for example `asset://ui/icons/run.svg#normal` -> `res://ui/icons/run.svg#normal`.
- `UiResourceResolverSchemeMap::project_to(...)` maps `project://path#label` to the configured runtime scheme, and `project_to_package(package_id)` maps it to `package://{package_id}/path#label`.
- malformed or unsupported non-UI URI schemes produce `InvalidUri`.

This keeps authoring validation and runtime registry lookup consistent: UI schemes are tolerated by default, and hosts that have imported editor/project-relative roots can opt into deterministic runtime locator lookup without adding file-system reads to the resolver.

## Fallbacks

When the primary resource fails:

- `fallback.mode = placeholder` tries to resolve the fallback URI with the same expected UI kind.
- a successful fallback returns `Placeholder { handle: Some(...), diagnostic_index: primary_failure }`.
- a missing fallback adds a separate `MissingFallback` diagnostic and returns `Placeholder { handle: None, diagnostic_index: primary_failure }`.
- `none` and `optional` return a placeholder without attempting fallback lookup.

The resolver does not panic on missing or mismatched resources. Missing primaries are warnings; invalid URIs and kind mismatches are errors. Missing placeholder fallbacks are errors because the author explicitly requested a concrete visual fallback.

## Cache

`UiResourceResolver` caches results by the full `UiResourceRef`. The cache avoids repeating registry lookup and duplicate diagnostics for identical references in one resolver lifetime. `clear_cache()` only clears resolved results; diagnostics remain queryable so editor consumers do not lose the authoring report for a frame.

`invalidate_uris(...)` removes cached references whose primary URI or fallback URI matches a changed resource URI. When `UiResourceResolverSchemeMap` is configured, invalidation also compares the primary and fallback UI URIs after converting them to their mapped runtime `ResourceLocator` strings. For example, a cached `asset://textures/checker.png` reference is invalidated by `res://textures/checker.png` when `asset://` is mapped to `ResourceScheme::Res`.

The invalidation report returns the requested URI list, removed cached reference count, and retained diagnostic count. Diagnostics intentionally remain available because editor panels may still need to show the authoring report for the frame that triggered the reload.

`UiAssetHotReloadExecutor` optionally accepts a mutable resolver. When a hot-reload plan has `resource_refresh_assets`, the executor calls `invalidate_uris(...)` and includes the report in `UiAssetHotReloadExecutionReport::resource_resolver_cache`. Plans without resource-refresh work leave the resolver untouched.

## Dependency Reports

`UiResourceResolver::resolve_dependencies(...)` is the batch surface for compiled template resource dependencies. It accepts the `UiResourceDependency` list produced by the compiler and returns `UiResourceResolutionReport`.

Each `UiResolvedResourceDependency` keeps the original dependency, the resolved handle or placeholder, and the diagnostic indices associated with that dependency. The original dependency includes the template path and source category, so editor panels can group failures by `imports.resources`, node props, style declarations, or imported widget/style documents without re-running collection.

The report preserves resolver cache semantics. If two dependency paths reference the same missing resource, only one diagnostic is emitted, but both dependency report rows point back to that diagnostic. Placeholder fallbacks are included in the lookup so missing fallback diagnostics remain attached to the dependency that requested them.

The report also exposes `resolved_count()`, `placeholder_count()`, and `has_errors()` as small summary helpers for hot-reload and editor diagnostics. These helpers do not classify severity beyond the diagnostics already produced by the resolver.

## Current Coverage

`zircon_runtime/src/ui/tests/asset_resource_resolver.rs` covers:

- successful lookup of a registered image reference as a texture resource handle.
- primary miss with a registered placeholder fallback handle.
- legal `asset://` UI scheme producing missing-resource diagnostics rather than invalid-URI diagnostics.
- configured `asset://` -> `res://` and `project://` -> `package://...` scheme mapping resolving registered runtime records.
- UI scheme mapping preserving `#label` subresource fragments.
- missing placeholder fallback producing a separate fallback diagnostic.
- kind mismatch refusing to return the wrong runtime handle.
- cache reuse for repeated references.
- primary and fallback URI cache invalidation with duplicate/blank invalidation requests ignored.
- primary and fallback cache invalidation when cached UI scheme references are refreshed through their mapped runtime locator.
- dependency-resolution reports preserving dependency paths, summary counts, and cached diagnostic index reuse for duplicate references.
- hot-reload execution invalidating the resolver cache for refreshed icon/font/texture resources when a resolver is supplied.

The first validation layer for this slice is formatting and diff hygiene. Focused runtime test execution remains part of the milestone testing stage because recent cold lib-test rebuilds in this workspace have timed out or stopped in unrelated scene/core test changes.

2026-06-20 update: `UiResourceResolverSchemeMap` added host-configured UI scheme mapping to the resolver while keeping default `asset://` / `project://` missing-not-invalid behavior. New behavior anchors cover `asset://` mapping to `res://`, `project://` mapping to `package://{package_id}/...`, preserving `#label` when a UI scheme URI is converted into a runtime locator, and invalidating mapped UI scheme cache entries when hot reload reports the runtime locator URI.
