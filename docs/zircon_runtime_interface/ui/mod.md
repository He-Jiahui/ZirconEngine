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
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/package_manifest.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/validate.rs
  - zircon_editor/src/ui/template/service.rs
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
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/package_manifest.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/validate.rs
  - zircon_editor/src/ui/template/service.rs
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
  - user: 2026-05-02 continue package/cache classification and editor template-service façade
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - Get-PSDrive -Name E (145.35 GB free before quality-fix validation)
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (pass after offline lockfile sync)
  - cargo test -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (16 passed; doc-tests 0)
  - cargo tree -p zircon_runtime_interface --locked (no crossbeam-channel, crossbeam-utils, zircon_runtime, zircon_editor, slint, or wgpu)
  - git diff --check -- "zircon_runtime_interface/Cargo.toml" "zircon_runtime_interface/src/tests/contracts.rs" "zircon_runtime_interface/src/ui/template/asset/style.rs" "zircon_runtime_interface/src/ui/template/asset/schema/report.rs" "docs/zircon_runtime_interface/ui/mod.md" (pass)
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs zircon_runtime/src/ui/template/asset/binding/mod.rs zircon_runtime/src/ui/template/asset/binding/validation.rs zircon_editor/src/ui/asset_editor/diagnostics/binding.rs zircon_editor/src/ui/asset_editor/binding/schema_projection.rs (pass for binding DTO owner cutover)
  - cargo test -p zircon_runtime --lib asset_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (16 passed; 0 failed; 642 filtered out)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (pass with existing warnings after stale render-extract call sites moved to runtime behavior)
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/template/asset/schema/report.rs zircon_runtime/src/ui/template/asset/loader.rs zircon_runtime/src/ui/template/asset/schema/migrator.rs zircon_runtime/src/ui/tree/hit_test.rs zircon_runtime/src/ui/tree/mod.rs zircon_runtime/src/ui/tree/node/{mod.rs,tree_access.rs,layout.rs,routing.rs,render_order.rs,interaction.rs,focus.rs,scroll.rs} (pass for schema/tree duplicate-owner cleanup)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (pass with warnings only after schema/tree duplicate-owner cleanup)
  - cargo test -p zircon_runtime --test ui_asset_binding_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (16 passed; 0 failed after schema/tree duplicate-owner cleanup)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-editor-check --message-format short --color never (pass with existing runtime graphics warnings and 3 editor warnings after editor tree DTO/runtime extension split)
  - cargo test -p zircon_runtime_interface ui_component_state_with_value_clears_reference_source_metadata --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (1 passed; 0 failed)
  - cargo test -p zircon_runtime --lib component_state --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-component-state --message-format short --color never (20 passed; 0 failed)
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (pass after component-state provenance cleanup)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-runtime-check --message-format short --color never (pass with warnings only after component-state provenance cleanup)
  - cargo test -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (17 passed; 0 failed; doc-tests 0)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-editor-check --message-format short --color never (pass with existing runtime graphics warnings and 3 editor warnings after component-state provenance cleanup)
  - Get-PSDrive -Name E (150.23 GB free before final focused UI runtime-interface validation)
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover-opencode --message-format short --color never (fresh pass)
  - cargo test -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover-opencode --message-format short --color never (17 passed; doc-tests 0)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover-opencode --message-format short --color never (fresh pass with existing graphics/plugin warnings)
  - cargo test -p zircon_runtime --test ui_asset_binding_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover-opencode --message-format short --color never (16 passed)
  - cargo test -p zircon_runtime --lib asset_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover-opencode --message-format short --color never (16 passed; 644 filtered)
  - cargo test -p zircon_runtime --lib asset_action_policy --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover-opencode --message-format short --color never (3 passed; 657 filtered)
  - cargo test -p zircon_runtime --lib asset_localization --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover-opencode --message-format short --color never (5 passed; 655 filtered)
  - cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover-opencode --message-format short --color never (9 passed; 651 filtered)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover-opencode --message-format short --color never (earlier fresh pass with existing runtime graphics warnings and 3 editor warnings)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-package-cache-opencode --message-format short --color never (2026-05-02 19:44 rerun passed with existing runtime graphics warnings and 3 editor warnings; the earlier `EditorPluginRegistrationReport.lifecycle` constructor blocker is stale in current source)
  - cargo tree -p zircon_editor --locked --depth 1 (direct dependencies include zircon_runtime for concrete services and zircon_runtime_interface for neutral contracts)
  - cargo build --workspace --locked --verbose --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-package-cache-opencode --message-format short --color never (package/cache closeout workspace build passed)
  - cargo test -p zircon_runtime --test virtual_geometry_debug_snapshot_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-package-cache-opencode --message-format short --color never --no-run (first blocked outside this UI DTO slice by `zircon_runtime/tests/support/mod.rs` using the stale 2-argument VG renderer fixture constructor; the follow-up fixture patch now supplies render features, no-op render-pass executor registrations, and a minimal virtual-geometry runtime provider)
  - cargo clean --target-dir E:\cargo-targets\zircon-ui-interface-package-cache-opencode (2026-05-03 follow-up cleanup of an inactive scoped target after E drive free space fell below the 50 GB Cargo threshold; removed 2700 files, 2.5 GiB)
  - cargo check -p zircon_runtime --test virtual_geometry_debug_snapshot_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-package-cache-opencode --message-format short --color never (2026-05-03 follow-up passed with existing runtime warnings after the shared VG fixture moved to the 4-argument constructor path)
doc_type: module-detail
---

# Runtime Interface UI Contracts

`zircon_runtime_interface::ui` owns the neutral UI DTO contract namespace used across runtime, editor, app, and future plugin seams. The module tree is backed by real files under `zircon_runtime_interface/src/ui/**`; it no longer path-includes `zircon_runtime/src/ui/**` source.

## Scope

The interface crate contains serializable declarations and narrow contract helpers for binding, component descriptors, dispatch inputs/results, event control/reflection, layout geometry, surface render extracts, template asset reports, localization/action-policy records, and tree node snapshots.

Runtime behavior remains outside this crate. Event managers, component registries and editor showcase catalogs, dispatchers, layout pass algorithms, surface orchestration, render extraction, text layout, tree mutation/query extensions, template loaders, compilers, validators, schema migrators, and graphics/plugin renderer execution still belong to `zircon_runtime` milestones.

## Module Families

`ui::binding` contains event binding DTOs plus parsing helpers needed on the contract type itself.

`ui::component` contains component category, value, drag/drop, event, validation state, descriptor, and data-binding DTOs. `UiComponentState` is data-only in the interface crate; event application and descriptor-backed state mutation remain runtime/editor behavior. Its direct `with_value(...)` helper preserves DTO invariants by clearing stale per-property drag/drop provenance whenever a retained value is replaced outside the runtime reducer.

`ui::dispatch` contains pointer and navigation dispatch context, invocation, effect, result, and pointer event DTOs, but no dispatchers.

`ui::event_ui` contains control request/response, reflection descriptors, stable scalar/string ID wrappers, and a serde JSON binding codec helper.

`ui::layout` contains constraints, geometry, scroll/container, and virtualization contract structures without layout-pass execution or virtualization window computation.

`ui::surface` contains focus/navigation/pointer DTOs and render command/list/style/text/extract declarations. `UiRenderExtract` and text layout records are data-only in the interface crate.

`ui::template` contains template document DTOs plus asset binding, action-policy, localization, compile-cache key, package header/cache-record/manifest/report, component-contract, invalidation, resource-ref, schema-report, selector, and asset document contract records. Selector parsing stays as a contract helper; selector matching stays outside the interface crate. Runtime owns compiler-state builders such as `compile_cache_key_from_compiler(...)`, runtime binary artifact encoding/decoding through `UiRuntimeCompiledAssetArtifact`, and package-manifest assembly from runtime artifacts. The interface `UiCompiledAssetArtifact` name is neutral DTO data only and does not carry a runtime `UiTemplateInstance` payload.

`ui::template::asset::binding` is the canonical source for M18 neutral binding target, expression, diagnostic, and report DTOs. The runtime binding module keeps validation behavior in `zircon_runtime::ui::template::asset::binding::validation` and imports these DTOs directly; the deleted runtime-local `diagnostic.rs`, `expression.rs`, and `target.rs` files are not compatibility surfaces.

`UiAssetDocument` exposes only declaration fields and minimal root-id accessors in this crate. Tree authority checks, style/node mutation, node traversal, template loading, and document validation are runtime/editor behavior and are intentionally absent from the interface source tree.

`ui::tree` contains data-only tree node declarations. Runtime tree mutation/query behavior remains in `zircon_runtime` and is exposed through `UiRuntimeTree*Ext` traits over the interface-owned `UiTree` and `UiTreeNode` DTOs.

`zircon_runtime::ui::surface::UiSurface` is still a runtime service type, but its `tree` field now stores `zircon_runtime_interface::ui::tree::UiTree` directly. Editor surface builders therefore import tree DTOs from the interface crate and import runtime tree extension traits only when they call behavior such as insertion, query, mutation, routing, focus, scroll, or render-order traversal.

## Milestone 1 Boundary

This milestone only materializes the interface-owned contract source tree and tests representative construction/serialization. Runtime and editor rewiring is intentionally deferred to later milestones in the UI runtime interface big cutover plan.

The focused runtime/editor gates below are unblock evidence, not workspace-wide acceptance. Subsequent M2 and tree/surface slices removed the runtime-local DTO shadows and old-path re-export shells for the touched seams, including the tree DTO family. A final residue audit found no live `#[path = ...]`, runtime-source include, shim, facade, or bridge residue in `zircon_runtime_interface/src/ui`, and no migration-only `pub use zircon_runtime_interface::ui` under `zircon_runtime/src/ui`. Remaining editor neutral DTO imports outside the tree family still need DTO-by-DTO hard-cutover around concrete runtime services rather than `pub use` compatibility shims.

## Milestone 1 Evidence

Milestone 1 acceptance is scoped to the interface crate: `cargo check -p zircon_runtime_interface`, `cargo test -p zircon_runtime_interface`, and `cargo tree -p zircon_runtime_interface` must pass after checking free space on the target drive.

These checks prove the interface UI namespace is implementation-free and dependency-light. They do not claim runtime, editor, graphics/plugin, or workspace-wide build and test success.

## Runtime And Editor Gate Evidence

The focused runtime checks also confirm that the M18 binding, M21 action-policy, M14 localization, package-validation, and component-state filters execute after the interface DTO namespace is materialized. The component-state provenance regression specifically keeps interface `UiComponentState::with_value(...)` aligned with runtime reducer value replacement by clearing stale `reference_sources` metadata. Fresh final validation on `E:\cargo-targets\zircon-ui-interface-big-cutover-opencode` passed the interface crate check/test, runtime lib check, binding integration test, and all four focused runtime filters listed in the header. The package/cache follow-up removed the remaining runtime duplicate `UiCompileCacheKey`, `UiCompiledAssetCacheRecord`, and `UiCompiledAssetPackageManifest` declarations; runtime now emits those interface DTOs through behavior helpers while keeping only `UiRuntimeCompiledAssetArtifact` as the runtime-owned binary artifact payload wrapper. An earlier editor library type-check gate also passed with existing warnings after stale `UiRenderExtract::from_tree(...)` call sites were moved to runtime-owned `extract_ui_render_tree(...)` behavior and after editor tree DTO construction moved to `zircon_runtime_interface::ui::tree` while retaining runtime tree extension traits for behavior calls. The editor template service follow-up adds `EditorTemplateRuntimeService` as the editor-owned façade over high-level runtime template loading, compilation, registration, instantiation, surface construction, render extraction, and binding diagnostic collection. The 2026-05-02 20:45 isolated rerun on `E:\cargo-targets\zircon-ui-interface-followup-opencode` confirms the interface check/test, runtime lib check, and editor lib check still type-check the package/cache and editor-template-service source with existing warnings only. The same rerun could not execute the `asset_package_validation` lib-test filter in the moved worktree because broader runtime lib-test compilation now fails first in active plugin/sound/export code, not in the UI package/cache owner seam. Broad workspace-test green is still unclaimed because validation is currently blocked by unrelated active lanes.

`zircon_editor` still depends on `zircon_runtime` through deliberate concrete runtime services such as UI behavior builders, event management, rendering submission, and host implementation. The tree DTO family is no longer part of that dependency debt, and the latest known-neutral stale-owner grep gates did not find DTO imports through `zircon_runtime::ui`; any remaining non-tree owner surfaces need a dedicated editor review instead of mechanical rewriting. The latest source audit found 134 `zircon_runtime::ui` hits and 431 `zircon_runtime_interface::ui` hits under `zircon_editor/src`; the current evidence does not claim every editor runtime UI dependency has been replaced by `zircon_runtime_interface::ui`.
