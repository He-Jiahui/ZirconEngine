---
related_code:
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle/external_source.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle/v2_projection.rs
  - zircon_editor/src/ui/asset_editor/session/hierarchy_projection.rs
  - zircon_editor/src/ui/asset_editor/session/promotion_state.rs
  - zircon_editor/src/ui/asset_editor/promote_widget.rs
  - zircon_editor/src/ui/asset_editor/style/theme_authoring.rs
  - zircon_editor/src/ui/asset_editor/style/theme_authoring/promotion.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/hydration.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh.rs
  - zircon_editor/src/tests/support.rs
  - zircon_editor/src/tests/ui/component_adapter.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs
  - zircon_runtime_interface/src/ui/v2/asset.rs
  - zircon_editor/src/tests/host/manager/ui_asset_reference_and_promotion.rs
  - zircon_editor/src/tests/host/manager/ui_asset_reference_and_promotion/theme.rs
  - zircon_editor/src/tests/host/manager/ui_asset_session_preview.rs
implementation_files:
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle/external_source.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle/v2_projection.rs
  - zircon_editor/src/ui/asset_editor/session/promotion_state.rs
  - zircon_editor/src/ui/asset_editor/style/theme_authoring/promotion.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/hydration.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh.rs
  - zircon_runtime_interface/src/ui/v2/asset.rs
plan_sources:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - ui::asset_editor::session::lifecycle::v2_projection::tests::v2_projection_roundtrip_preserves_reference_component_and_named_mount
  - ui::asset_editor::session::lifecycle::v2_projection::tests::component_projection_uses_component_root_without_view_root
  - ui::asset_editor::session::lifecycle::v2_projection::tests::v2_serializer_rejects_component_assets_with_multiple_components
  - tests::host::manager::ui_asset_reference_and_promotion
  - tests::host::manager::ui_asset_session_preview::editor_manager_opens_and_saves_ui_asset_editor_sessions
  - tests::ui::component_adapter::asset_editor_component_adapter
  - tests::ui::ui_asset_editor::bootstrap_assets
  - tests::editing::ui_asset
---

# UI Asset V2 Authoring Boundary

The editor keeps `UiAssetDocument` as its in-memory authoring projection while every `.zui` source boundary is `UiV2AssetDocument`. This projection is an internal editing model, not a compatibility format: `.zui` load, canonical save, promoted external assets, and undo restoration all serialize or validate the V2 schema.

## Ownership

`session/lifecycle/v2_projection.rs` owns both projection directions and the single V2 serializer. The serializer reparses its output through `UiZuiAssetLoader` before returning, so schema and profile errors fail before any file effect is applied. `lifecycle.rs` selects the source schema and uses that serializer for canonical source. `lifecycle/external_source.rs` owns external widget/style source snapshots and chooses the V2 serializer for `.zui` restoration. `promotion_state.rs` uses the same boundary for promoted widget and theme files, while host `editing/node_ops.rs` uses it for the initial external write, so initial write, redo, and restoration cannot diverge.

`host/asset_editor_sessions/imports.rs` owns production import loading. It normalizes fragment references before selecting a loader: `.zui` imports use `UiZuiAssetLoader` only, while non-`.zui` imports use the legacy document loader. Each V2 import contributes both its untouched `UiV2AssetDocument` to the preview prototype store and its internal authoring projection to hierarchy/Inspector tooling. `hydration.rs` and `refresh.rs` commit all four widget/style maps through one `replace_resolved_imports` call before revalidation, so preview and authoring views cannot observe different import generations.

Reference nodes preserve their full asset locator in V2 `node.component`; local component instances preserve the local component name; slot placeholders use `component = "Slot"` plus `props.name`; child mounts use `slot.name`. V2-to-editor projection restores those distinctions for hierarchy, Inspector, reference navigation, and promotion commands. Hierarchy labels resolve native widget type, external reference, then local component name, so local components and `Slot` placeholders cannot collapse to an untyped `Node` label.

`UiV2ComponentDefinition.contract` is the authoritative serialized `UiComponentPublicContract`. Both projection directions preserve the complete contract, including `root_class_policy`; the default contract is omitted from TOML, while non-default policy survives canonical save and reload. This is part of schema 2 authoring, not a legacy compatibility side channel.

Asset kind controls root ownership. View assets may declare `[root]`. Component assets expose roots only through `[components.<name>].root` and must not declare a view root. Style and theme-token assets have neither a view root nor component root. Every serialized V2 document writes `UI_V2_ASSET_SCHEMA_VERSION`.

## Hard Cutover

Production `.zui` code does not fall back to `UiWidgetAsset`, `UiStyleAsset`, or the legacy layout loader. Production host hydration also keeps the parsed V2 document instead of rebuilding preview imports from the authoring projection. Tests that begin from compact legacy DTO fixtures convert them only inside `tests/support.rs` before writing a `.zui`; assertions over emitted `.zui` files use `UiZuiAssetLoader`. This keeps fixture migration out of production and makes invalid V2 output fail at the real loader boundary.

## Validation Status

The current Windows binary passes the V2 projection tests 3/3, manager UI Asset tests 44/44, UI Asset editing session group 168/168, V2 bootstrap group 14/14, and asset-editor component adapter group 3/3. These runs cover reference/local-component/Slot/named-mount roundtrip, component-root profile validation, promotion/save/undo/redo, hierarchy labels, and non-default component public-contract save/reload. The full 2928-test crate gate completed as 2754 passed / 140 failed / 34 ignored; remaining failures are outside this authoring group and Plan 01 M1 stays open until the declared full-crate gate is green.
