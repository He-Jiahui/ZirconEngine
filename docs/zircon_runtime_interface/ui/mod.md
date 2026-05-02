---
related_code:
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/binding/mod.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_call.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value.rs
  - zircon_runtime_interface/src/ui/binding/model/event_binding.rs
  - zircon_runtime_interface/src/ui/binding/model/event_kind.rs
  - zircon_runtime_interface/src/ui/binding/model/event_path.rs
  - zircon_runtime_interface/src/ui/binding/model/mod.rs
  - zircon_runtime_interface/src/ui/binding/model/parse_error.rs
  - zircon_runtime_interface/src/ui/binding/model/parser.rs
  - zircon_runtime_interface/src/ui/component/mod.rs
  - zircon_runtime_interface/src/ui/component/category.rs
  - zircon_runtime_interface/src/ui/component/data_binding/adapter_error.rs
  - zircon_runtime_interface/src/ui/component/data_binding/adapter_result.rs
  - zircon_runtime_interface/src/ui/component/data_binding/binding_target.rs
  - zircon_runtime_interface/src/ui/component/data_binding/data_source.rs
  - zircon_runtime_interface/src/ui/component/data_binding/event_envelope.rs
  - zircon_runtime_interface/src/ui/component/data_binding/mod.rs
  - zircon_runtime_interface/src/ui/component/data_binding/projection_patch.rs
  - zircon_runtime_interface/src/ui/component/descriptor/component_descriptor.rs
  - zircon_runtime_interface/src/ui/component/descriptor/default_node_template.rs
  - zircon_runtime_interface/src/ui/component/descriptor/fallback_policy.rs
  - zircon_runtime_interface/src/ui/component/descriptor/host_capability.rs
  - zircon_runtime_interface/src/ui/component/descriptor/mod.rs
  - zircon_runtime_interface/src/ui/component/descriptor/option_descriptor.rs
  - zircon_runtime_interface/src/ui/component/descriptor/palette_metadata.rs
  - zircon_runtime_interface/src/ui/component/descriptor/prop_schema.rs
  - zircon_runtime_interface/src/ui/component/descriptor/render_capability.rs
  - zircon_runtime_interface/src/ui/component/descriptor/slot_schema.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime_interface/src/ui/component/state.rs
  - zircon_runtime_interface/src/ui/component/validation.rs
  - zircon_runtime_interface/src/ui/component/value.rs
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/context.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/invocation.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/context.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/event.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/invocation.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/event_ui/mod.rs
  - zircon_runtime_interface/src/ui/event_ui/codec.rs
  - zircon_runtime_interface/src/ui/event_ui/control.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/virtualization.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs
  - zircon_runtime_interface/src/ui/surface/navigation/mod.rs
  - zircon_runtime_interface/src/ui/surface/navigation/route.rs
  - zircon_runtime_interface/src/ui/surface/navigation_state.rs
  - zircon_runtime_interface/src/ui/surface/pointer/button.rs
  - zircon_runtime_interface/src/ui/surface/pointer/event_kind.rs
  - zircon_runtime_interface/src/ui/surface/pointer/mod.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/command_kind.rs
  - zircon_runtime_interface/src/ui/surface/render/extract.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/render/visual_asset_ref.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime_interface/src/ui/template/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/host_policy.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/report.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/side_effect_class.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/target.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/cache/cache_key.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/cache/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/cache_record.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/manifest.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/package_manifest.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/profile.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/report.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/binding_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/focus_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/public_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/public_part.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/root_class_policy.rs
  - zircon_runtime_interface/src/ui/template/asset/document.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/change.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/fingerprint.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/impact.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/report.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/stage.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/localized_text_ref.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/report.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/text_direction.rs
  - zircon_runtime_interface/src/ui/template/asset/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/dependency.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/fallback_policy.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/report.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_kind.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_ref.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/policy.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/report.rs
  - zircon_runtime_interface/src/ui/template/asset/style.rs
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/tree/mod.rs
  - zircon_runtime_interface/src/ui/tree/node/dirty_flags.rs
  - zircon_runtime_interface/src/ui/tree/node/input_policy.rs
  - zircon_runtime_interface/src/ui/tree/node/layout_cache.rs
  - zircon_runtime_interface/src/ui/tree/node/mod.rs
  - zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_error.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
implementation_files:
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/binding/mod.rs
  - zircon_runtime_interface/src/ui/binding/model/mod.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_call.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value.rs
  - zircon_runtime_interface/src/ui/binding/model/event_binding.rs
  - zircon_runtime_interface/src/ui/binding/model/event_kind.rs
  - zircon_runtime_interface/src/ui/binding/model/event_path.rs
  - zircon_runtime_interface/src/ui/binding/model/parse_error.rs
  - zircon_runtime_interface/src/ui/binding/model/parser.rs
  - zircon_runtime_interface/src/ui/component/mod.rs
  - zircon_runtime_interface/src/ui/component/category.rs
  - zircon_runtime_interface/src/ui/component/data_binding/adapter_error.rs
  - zircon_runtime_interface/src/ui/component/data_binding/adapter_result.rs
  - zircon_runtime_interface/src/ui/component/data_binding/binding_target.rs
  - zircon_runtime_interface/src/ui/component/data_binding/data_source.rs
  - zircon_runtime_interface/src/ui/component/data_binding/event_envelope.rs
  - zircon_runtime_interface/src/ui/component/data_binding/mod.rs
  - zircon_runtime_interface/src/ui/component/data_binding/projection_patch.rs
  - zircon_runtime_interface/src/ui/component/descriptor/component_descriptor.rs
  - zircon_runtime_interface/src/ui/component/descriptor/default_node_template.rs
  - zircon_runtime_interface/src/ui/component/descriptor/fallback_policy.rs
  - zircon_runtime_interface/src/ui/component/descriptor/host_capability.rs
  - zircon_runtime_interface/src/ui/component/descriptor/mod.rs
  - zircon_runtime_interface/src/ui/component/descriptor/option_descriptor.rs
  - zircon_runtime_interface/src/ui/component/descriptor/palette_metadata.rs
  - zircon_runtime_interface/src/ui/component/descriptor/prop_schema.rs
  - zircon_runtime_interface/src/ui/component/descriptor/render_capability.rs
  - zircon_runtime_interface/src/ui/component/descriptor/slot_schema.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime_interface/src/ui/component/state.rs
  - zircon_runtime_interface/src/ui/component/validation.rs
  - zircon_runtime_interface/src/ui/component/value.rs
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/context.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/invocation.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/context.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/event.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/invocation.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/event_ui/mod.rs
  - zircon_runtime_interface/src/ui/event_ui/codec.rs
  - zircon_runtime_interface/src/ui/event_ui/control.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/virtualization.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs
  - zircon_runtime_interface/src/ui/surface/navigation/mod.rs
  - zircon_runtime_interface/src/ui/surface/navigation/route.rs
  - zircon_runtime_interface/src/ui/surface/navigation_state.rs
  - zircon_runtime_interface/src/ui/surface/pointer/button.rs
  - zircon_runtime_interface/src/ui/surface/pointer/event_kind.rs
  - zircon_runtime_interface/src/ui/surface/pointer/mod.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/command_kind.rs
  - zircon_runtime_interface/src/ui/surface/render/extract.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/render/visual_asset_ref.rs
  - zircon_runtime_interface/src/ui/template/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/host_policy.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/report.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/side_effect_class.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/target.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/cache/cache_key.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/cache/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/cache_record.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/manifest.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/package_manifest.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/profile.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/report.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/binding_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/focus_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/public_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/public_part.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/root_class_policy.rs
  - zircon_runtime_interface/src/ui/template/asset/document.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/change.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/fingerprint.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/impact.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/report.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/stage.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/localized_text_ref.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/report.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/text_direction.rs
  - zircon_runtime_interface/src/ui/template/asset/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/dependency.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/fallback_policy.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/report.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_kind.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_ref.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/policy.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/report.rs
  - zircon_runtime_interface/src/ui/template/asset/style.rs
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/tree/mod.rs
  - zircon_runtime_interface/src/ui/tree/node/dirty_flags.rs
  - zircon_runtime_interface/src/ui/tree/node/input_policy.rs
  - zircon_runtime_interface/src/ui/tree/node/layout_cache.rs
  - zircon_runtime_interface/src/ui/tree/node/mod.rs
  - zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_error.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
plan_sources:
  - docs/superpowers/specs/2026-05-02-ui-runtime-interface-big-cutover-design.md
  - docs/superpowers/plans/2026-05-02-ui-runtime-interface-big-cutover.md
  - user: 2026-05-02 approve subagent-driven UI runtime interface big cutover
  - user: 2026-05-02 continue active UI runtime-interface cutover
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - Get-PSDrive -Name E (145.35 GB free before quality-fix validation)
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (pass after offline lockfile sync)
  - cargo test -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (16 passed; doc-tests 0)
  - cargo tree -p zircon_runtime_interface --locked (no crossbeam-channel, crossbeam-utils, zircon_runtime, zircon_editor, slint, or wgpu)
  - git diff --check -- "zircon_runtime_interface/Cargo.toml" "zircon_runtime_interface/src/tests/contracts.rs" "zircon_runtime_interface/src/ui/template/asset/style.rs" "zircon_runtime_interface/src/ui/template/asset/schema/report.rs" "docs/zircon_runtime_interface/ui/mod.md" (pass)
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs zircon_runtime/src/ui/template/asset/binding/mod.rs zircon_runtime/src/ui/template/asset/binding/validation.rs zircon_editor/src/ui/asset_editor/diagnostics/binding.rs zircon_editor/src/ui/asset_editor/binding/schema_projection.rs (pending for binding DTO owner cutover)
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (pending for binding DTO owner cutover)
  - cargo test -p zircon_runtime --lib asset_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (pending for binding DTO owner cutover)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (pending for binding DTO owner cutover; earlier run was blocked by unrelated js-sys/web-sys lock drift)
doc_type: module-detail
---

# Runtime Interface UI Contracts

`zircon_runtime_interface::ui` owns the neutral UI DTO contract namespace used across runtime, editor, app, and future plugin seams. The module tree is backed by real files under `zircon_runtime_interface/src/ui/**`; it no longer path-includes `zircon_runtime/src/ui/**` source.

## Scope

The interface crate contains serializable declarations and narrow contract helpers for binding, component descriptors, dispatch inputs/results, event control/reflection, layout geometry, surface render extracts, template asset reports, localization/action-policy records, and tree node snapshots.

Runtime behavior remains outside this crate. Event managers, component registries and editor showcase catalogs, dispatchers, layout pass algorithms, surface orchestration, render extraction, text layout, tree mutation/query extensions, template loaders, compilers, validators, schema migrators, and graphics/plugin renderer execution still belong to `zircon_runtime` milestones.

## Module Families

`ui::binding` contains event binding DTOs plus parsing helpers needed on the contract type itself.

`ui::component` contains component category, value, drag/drop, event, validation state, descriptor, and data-binding DTOs. `UiComponentState` is data-only in the interface crate; event application and descriptor-backed state mutation remain runtime/editor behavior.

`ui::dispatch` contains pointer and navigation dispatch context, invocation, effect, result, and pointer event DTOs, but no dispatchers.

`ui::event_ui` contains control request/response, reflection descriptors, stable scalar/string ID wrappers, and a serde JSON binding codec helper.

`ui::layout` contains constraints, geometry, scroll/container, and virtualization contract structures without layout-pass execution or virtualization window computation.

`ui::surface` contains focus/navigation/pointer DTOs and render command/list/style/text/extract declarations. `UiRenderExtract` and text layout records are data-only in the interface crate.

`ui::template` contains template document DTOs plus asset binding, action-policy, localization, package-report, component-contract, invalidation, resource-ref, schema-report, selector, and asset document contract records. Selector parsing stays as a contract helper; selector matching stays outside the interface crate.

`ui::template::asset::binding` is the canonical source for M18 neutral binding target, expression, diagnostic, and report DTOs. The runtime binding module keeps validation behavior in `zircon_runtime::ui::template::asset::binding::validation` and imports these DTOs directly; the deleted runtime-local `diagnostic.rs`, `expression.rs`, and `target.rs` files are not compatibility surfaces.

`UiAssetDocument` exposes only declaration fields and minimal root-id accessors in this crate. Tree authority checks, style/node mutation, node traversal, template loading, and document validation are runtime/editor behavior and are intentionally absent from the interface source tree.

`ui::tree` contains data-only tree node declarations. Runtime tree mutation/query behavior remains in `zircon_runtime`.

## Milestone 1 Boundary

This milestone only materializes the interface-owned contract source tree and tests representative construction/serialization. Runtime and editor rewiring is intentionally deferred to later milestones in the UI runtime interface big cutover plan.

The focused runtime/editor gates below are unblock evidence, not hard-cutover acceptance. A 2026-05-02 source audit still found runtime-local duplicate neutral DTO declarations under `zircon_runtime/src/ui/**` and broad editor imports of neutral UI DTOs through `zircon_runtime::ui`. The next cutover slice must remove those duplicate owner paths directly instead of adding `pub use` compatibility shims.

## Milestone 1 Evidence

Milestone 1 acceptance is scoped to the interface crate: `cargo check -p zircon_runtime_interface`, `cargo test -p zircon_runtime_interface`, and `cargo tree -p zircon_runtime_interface` must pass after checking free space on the target drive.

These checks prove the interface UI namespace is implementation-free and dependency-light. They do not claim runtime, editor, graphics/plugin, or workspace-wide build and test success.

## Runtime And Editor Gate Evidence

The focused runtime checks also confirm that the M18 binding, M21 action-policy, M14 localization, and package-validation filters execute after the interface DTO namespace is materialized. The editor library type-check gate passes with existing warnings only after stale `UiRenderExtract::from_tree(...)` call sites were moved to runtime-owned `extract_ui_render_tree(...)` behavior.

`zircon_editor` still depends on `zircon_runtime` through deliberate concrete runtime services such as UI behavior builders, event management, rendering submission, and host implementation. The same audit also found unresolved neutral DTO import debt in editor source, so the current evidence only clears compile/test blockers; it does not claim editor neutral UI imports are fully routed through `zircon_runtime_interface::ui` yet.
