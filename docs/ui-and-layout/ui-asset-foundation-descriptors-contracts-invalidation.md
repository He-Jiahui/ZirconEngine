---
related_code:
  - zircon_runtime/src/ui/component/mod.rs
  - zircon_runtime/src/ui/component/descriptor/mod.rs
  - zircon_runtime/src/ui/component/catalog/mod.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/target.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/mod.rs
  - zircon_runtime_interface/src/ui/binding/mod.rs
  - zircon_runtime/src/ui/template/asset/binding/mod.rs
  - zircon_runtime/src/ui/template/asset/binding/validation.rs
  - zircon_runtime/src/ui/template/asset/style.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/template/asset/compiler/compile.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_props.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/mod.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/cache_key.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/outcome.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/mod.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/cache_record.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/header.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/manifest.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/package_manifest.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/profile.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/report.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/validate.rs
  - zircon_runtime/src/ui/template/asset/action_policy/mod.rs
  - zircon_runtime/src/ui/template/asset/action_policy/validate.rs
  - zircon_runtime/src/ui/template/asset/localization/mod.rs
  - zircon_runtime/src/ui/template/asset/localization/collect.rs
  - zircon_runtime/src/ui/template/asset/component_contract/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/binding_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/focus_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/public_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/public_part.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/root_class_policy.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/mod.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/collect.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_kind.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/fallback_policy.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_ref.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/dependency.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/diagnostic.rs
  - zircon_runtime/src/ui/template/asset/invalidation/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/change.rs
  - zircon_runtime/src/ui/template/asset/invalidation/diagnostic.rs
  - zircon_runtime/src/ui/template/asset/invalidation/fingerprint.rs
  - zircon_runtime/src/ui/template/asset/invalidation/graph.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/impact.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/report.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/stage.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/report.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
  - zircon_runtime/src/ui/template/asset/schema/legacy_template.rs
  - zircon_runtime/src/ui/tests/asset_compile_cache.rs
  - zircon_runtime/src/ui/tests/asset_component_contract.rs
  - zircon_runtime/src/ui/tests/asset_invalidation.rs
  - zircon_runtime/src/ui/tests/asset_binding.rs
  - zircon_runtime/tests/ui_asset_binding_contract.rs
  - zircon_runtime/src/ui/tests/asset_action_policy.rs
  - zircon_runtime/src/ui/tests/asset_localization.rs
  - zircon_runtime/src/ui/tests/asset_resource_refs.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/asset_editor/document_diff.rs
  - zircon_editor/src/ui/asset_editor/palette/mod.rs
  - zircon_editor/src/ui/asset_editor/tree/tree_editing.rs
  - zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution.rs
  - zircon_editor/src/ui/asset_editor/session/palette_state.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/presentation_state.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/mod.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/contract.rs
  - zircon_editor/src/ui/asset_editor/presentation.rs
  - zircon_editor/src/ui/asset_editor/session/promotion_state.rs
  - zircon_editor/src/tests/support.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/palette_descriptor_registry.rs
implementation_files:
  - zircon_runtime_interface/src/ui/component/descriptor/component_descriptor.rs
  - zircon_runtime_interface/src/ui/component/descriptor/prop_schema.rs
  - zircon_runtime_interface/src/ui/component/descriptor/slot_schema.rs
  - zircon_runtime_interface/src/ui/component/descriptor/option_descriptor.rs
  - zircon_runtime_interface/src/ui/component/descriptor/host_capability.rs
  - zircon_runtime_interface/src/ui/component/descriptor/render_capability.rs
  - zircon_runtime_interface/src/ui/component/descriptor/palette_metadata.rs
  - zircon_runtime_interface/src/ui/component/descriptor/default_node_template.rs
  - zircon_runtime_interface/src/ui/component/descriptor/fallback_policy.rs
  - zircon_runtime/src/ui/component/descriptor/validation.rs
  - zircon_runtime/src/ui/component/catalog/registry.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
  - zircon_runtime/src/ui/component/catalog/palette_view.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/target.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/action_policy/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/localization/mod.rs
  - zircon_runtime_interface/src/ui/binding/mod.rs
  - zircon_runtime/src/ui/template/asset/binding/mod.rs
  - zircon_runtime/src/ui/template/asset/binding/validation.rs
  - zircon_runtime/src/ui/template/asset/style.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/template/asset/compiler/compile.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_props.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/cache_key.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/outcome.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/mod.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/cache_record.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/header.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/manifest.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/package_manifest.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/profile.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/report.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/validate.rs
  - zircon_runtime/src/ui/template/asset/action_policy/mod.rs
  - zircon_runtime/src/ui/template/asset/action_policy/validate.rs
  - zircon_runtime/src/ui/template/asset/localization/mod.rs
  - zircon_runtime/src/ui/template/asset/localization/collect.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/binding_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/focus_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/public_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/public_part.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/root_class_policy.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/mod.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/collect.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_kind.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/fallback_policy.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_ref.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/dependency.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/change.rs
  - zircon_runtime/src/ui/template/asset/invalidation/diagnostic.rs
  - zircon_runtime/src/ui/template/asset/invalidation/fingerprint.rs
  - zircon_runtime/src/ui/template/asset/invalidation/graph.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/impact.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/report.rs
  - zircon_runtime_interface/src/ui/template/asset/invalidation/stage.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/report.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
  - zircon_runtime/src/ui/template/asset/schema/legacy_template.rs
  - zircon_editor/src/ui/asset_editor/palette/entry.rs
  - zircon_editor/src/ui/asset_editor/palette/build.rs
  - zircon_editor/src/ui/asset_editor/palette/instantiate.rs
  - zircon_editor/src/ui/asset_editor/palette/placement.rs
  - zircon_editor/src/ui/asset_editor/document_diff.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/mod.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/contract.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/presentation_state.rs
  - zircon_editor/src/ui/asset_editor/presentation.rs
  - zircon_editor/src/tests/support.rs
plan_sources:
  - user: 2026-05-01 implement Milestone 1 M13 Descriptor Registry And Descriptor-Driven Palette
  - docs/superpowers/specs/2026-05-01-ui-foundation-m10-m12-m13-design.md
  - docs/superpowers/plans/2026-05-01-ui-foundation-m10-m12-m13.md
  - .codex/plans/M16 UI Compiled Artifact And Package Validation Implementation Plan.md
  - docs/superpowers/plans/2026-05-01-ui-compiled-artifact-package-validation.md
  - .codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md
  - .codex/plans/UI 后续产品化与验证归档计划.md
  - docs/superpowers/plans/2026-05-01-ui-productization-editor-binding-parity.md
  - docs/superpowers/specs/2026-05-01-ui-asset-resource-refs-m15-design.md
  - docs/superpowers/plans/2026-05-01-ui-asset-resource-refs-m15.md
  - docs/superpowers/specs/2026-05-02-ui-binding-expression-semantics-m18-design.md
  - docs/superpowers/plans/2026-05-02-ui-binding-expression-semantics-m18.md
  - docs/superpowers/specs/2026-05-02-ui-runtime-interface-big-cutover-design.md
  - docs/superpowers/plans/2026-05-02-ui-runtime-interface-big-cutover.md
  - user: 2026-05-02 execute M21 then M14 runtime foundation
tests:
  - zircon_runtime/src/ui/tests/asset_binding.rs
  - zircon_runtime/tests/ui_asset_binding_contract.rs
  - zircon_runtime/src/ui/tests/asset_resource_refs.rs
  - zircon_runtime/src/ui/tests/asset_compile_cache.rs
  - zircon_runtime/src/ui/tests/asset_component_contract.rs
  - zircon_runtime/src/ui/tests/asset_invalidation.rs
  - zircon_runtime/src/ui/tests/asset_action_policy.rs
  - zircon_runtime/src/ui/tests/asset_localization.rs
  - zircon_runtime/src/ui/tests/asset_package_validation.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/palette_descriptor_registry.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/contract_diagnostics.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/component/mod.rs zircon_runtime/src/ui/component/descriptor/*.rs zircon_runtime/src/ui/component/catalog/*.rs zircon_runtime/src/ui/tests/component_catalog.rs zircon_editor/src/ui/asset_editor/mod.rs zircon_editor/src/ui/asset_editor/tree/tree_editing.rs zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution.rs zircon_editor/src/ui/asset_editor/session/palette_state.rs zircon_editor/src/ui/asset_editor/palette/*.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/palette_descriptor_registry.rs
  - cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
  - cargo test -p zircon_editor --lib palette_descriptor_registry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/component/descriptor/default_node_template.rs zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs zircon_runtime/src/ui/template/asset/schema/legacy_template.rs zircon_editor/src/ui/asset_editor/tree/tree_editing.rs zircon_editor/src/ui/asset_editor/palette/instantiate.rs zircon_editor/src/tests/ui/ui_asset_editor/palette_descriptor_registry.rs zircon_editor/src/tests/support.rs zircon_editor/src/ui/asset_editor/document_diff.rs
  - cargo test -p zircon_runtime --lib runtime_component_catalog_contains_showcase_v1_controls --locked --jobs 1 --target-dir E:\cargo-targets\zircon-srp-rhi-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-srp-rhi-main-chain --message-format short --color never
  - cargo test -p zircon_editor --lib palette_descriptor_registry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-srp-rhi-main-chain --message-format short --color never
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/mod.rs zircon_runtime/src/ui/template/mod.rs zircon_runtime/src/ui/template/asset/document.rs zircon_runtime/src/ui/template/asset/style.rs zircon_runtime/src/ui/template/asset/compiler/compile.rs zircon_runtime/src/ui/template/asset/component_contract/api_version.rs zircon_runtime/src/ui/template/asset/component_contract/binding_contract.rs zircon_runtime/src/ui/template/asset/component_contract/focus_contract.rs zircon_runtime/src/ui/template/asset/component_contract/mod.rs zircon_runtime/src/ui/template/asset/component_contract/public_contract.rs zircon_runtime/src/ui/template/asset/component_contract/public_part.rs zircon_runtime/src/ui/template/asset/component_contract/root_class_policy.rs zircon_runtime/src/ui/template/asset/component_contract/validation.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/asset_component_contract.rs zircon_runtime/src/ui/component/descriptor/default_node_template.rs zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs zircon_runtime/src/ui/template/asset/schema/legacy_template.rs
  - cargo test -p zircon_runtime --lib asset_component_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-contract-fresh --message-format short --color never (12 passed)
  - cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-contract-fresh --message-format short --color never -- --nocapture (60 passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-contract-fresh --message-format short --color never (passed with unrelated graphics/plugin warnings)
  - cargo test -p zircon_runtime --lib asset_invalidation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-contract-fresh --message-format short --color never (post-review rerun: 5 passed)
  - cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-contract-fresh --message-format short --color never (post-review rerun: 7 passed)
  - cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-contract-fresh --message-format short --color never -- --nocapture (post-review rerun: 74 passed)
  - cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-contract-fresh --message-format short --color never (post-review rerun: 39 passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-contract-fresh --message-format short --color never (post-review rerun passed with unrelated graphics/plugin warnings)
  - cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-m16-ui-package-validation --message-format short --color never (5 passed)
  - cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\cargo-targets\zircon-m16-ui-package-validation --message-format short --color never (7 passed)
  - cargo test -p zircon_runtime --lib asset_invalidation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-m16-ui-package-validation --message-format short --color never (5 passed)
  - cargo test -p zircon_runtime --lib asset_component_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-m16-ui-package-validation --message-format short --color never (12 passed)
  - cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-m16-ui-package-validation --message-format short --color never (39 passed)
  - cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-m16-ui-package-validation --message-format short --color never (67 passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-m16-ui-package-validation --message-format short --color never (passed with unrelated graphics/plugin warnings)
  - cargo test -p zircon_runtime --lib asset_component_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never (15 passed)
  - cargo test -p zircon_editor --lib contract_diagnostics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never (3 passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never (passed with unrelated graphics warnings)
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/resource_ref/*.rs zircon_runtime/src/ui/template/asset/document.rs zircon_runtime/src/ui/template/asset/mod.rs zircon_runtime/src/ui/template/mod.rs zircon_runtime/src/ui/tests/asset_resource_refs.rs zircon_runtime/src/ui/tests/mod.rs zircon_editor/src/ui/asset_editor/style/theme_authoring.rs (M15 M1 accepted: no output)
  - cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never (M15 M1 accepted: 7 passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never (M15 M1 downstream constructor check accepted with unrelated graphics warnings)
  - cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never (M15/M16 reconciliation: 9 passed)
  - cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never (M15/M16 reconciliation: 9 passed)
  - cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never (M15/M16 reconciliation: 9 passed)
  - cargo test -p zircon_runtime --lib asset_invalidation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never (M15/M16 reconciliation: 6 passed)
  - cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never (M15/M16 reconciliation: 86 passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never (M15/M16 reconciliation passed with unrelated graphics/plugin warnings)
  - cargo test -p zircon_runtime --lib asset_action_policy --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never (2026-05-02 rerun after UI DTO owner cutover: 3 passed)
  - cargo test -p zircon_runtime --lib asset_localization --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never (2026-05-02 rerun after resource/localization support fixes: 5 passed)
  - cargo test -p zircon_runtime --lib resource_collector_ignores_localized_text_fallback_strings --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never (2026-05-02 support-layer regression: 1 passed)
  - cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never (2026-05-02 rerun: 9 passed)
  - rustfmt --edition 2021 --check --config skip_children=true <DTO/resource/localization touched Rust files> (2026-05-02 rerun: no output)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never
  - rustfmt --edition 2021 --check <M18 binding runtime/editor compatibility files> (M18 accepted: no output)
  - cargo test -p zircon_runtime --test ui_asset_binding_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never (M18 accepted after review fixes: 16 passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never (M18 accepted with unrelated graphics/plugin warnings)
  - cargo test -p zircon_runtime --lib asset_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never (M18 blocked before filtered tests by unrelated lib-test UI DTO type-identity errors)
  - cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never -- --nocapture (M18 blocked before filtered tests by the same unrelated lib-test UI DTO type-identity errors)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (2026-05-02 M2 runtime DTO hard-cutover: passed with unrelated graphics/plugin warnings)
  - cargo test -p zircon_runtime --test ui_asset_binding_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (2026-05-02 M2 runtime DTO hard-cutover: 16 passed)
doc_type: module-detail
---

# UI Asset Foundation Descriptors Contracts Invalidation

## Runtime Interface DTO Ownership Status

The 2026-05-02 UI runtime-interface split makes `zircon_runtime_interface::ui` the canonical namespace for neutral descriptor, binding, component-value, component data-source, event-control/reflection, template asset/template document, layout geometry/window, surface render, tree/node, component-contract, invalidation, resource, action-policy, localization, package-report, schema-report, and related UI DTO declarations. `zircon_runtime::ui` remains the behavior owner for registries, validation, collection, compilation, cache/invalidation execution, package validation, schema migration, layout/window computation, tree access/routing/focus/interaction/scroll behavior, render extraction, and editor/runtime service APIs; behavior that needs methods on interface-owned records is exposed through runtime helpers or extension traits rather than duplicate DTO declarations.

This document still records the runtime behavior owners for M10/M12/M13/M15/M16/M18/M21/M14. Where older text says a runtime module owns a shared DTO, read that as behavior authority only: the M2 runtime hard-cutover removed the duplicate local DTO owner files for binding values, event UI control/reflection, component values, layout geometry, surface render DTOs, tree/node DTOs, schema report DTOs, action policy report DTOs, and localization report DTOs. Focused runtime checks now pass through `zircon_runtime_interface::ui` DTO identities; full workspace acceptance still requires migrating remaining editor neutral DTO imports to `zircon_runtime_interface::ui` directly.

## M13 Descriptor Authority

M13 formalizes the existing runtime component descriptor catalog as the shared widget descriptor authority. `zircon_runtime::ui::component::descriptor` owns descriptor validation and runtime descriptor behavior; neutral descriptor declarations, host/render capability declarations, palette metadata, default node templates, and fallback policy records are canonical interface DTOs after the runtime-interface split. `zircon_runtime::ui::component::catalog` owns the deterministic registry, built-in editor showcase descriptor set, revisioning, and palette view generation.

The editor consumes runtime-owned registry views. `zircon_editor::ui::asset_editor::palette` builds native palette rows from `UiComponentDescriptorRegistry::editor_showcase().palette_entries_for_host(&UiHostCapabilitySet::editor_authoring())`, then appends local component entries from `document.components.keys()` and imported references from `widget_imports.keys()`.

## Validation Coverage

Runtime tests cover duplicate schema rejection, missing default-prop schema rejection, host capability filtering, render capability declarations, palette metadata/default template generation, fallback policy, deterministic palette ordering, and registry revision bumping only on descriptor-set changes.

Editor tests cover descriptor-driven native palette rows, preservation of local component and imported reference entries, and default native node instantiation through descriptor templates.

## M10 Component Public Contract

M10 adds a folder-backed `zircon_runtime::ui::template::asset::component_contract` behavior owner for authored component contract validation. Local components now carry `UiComponentPublicContract`, including semantic `api_version`, named `public_parts`, root class policy, focus contract, and binding route declarations; those neutral declarations are canonical interface DTOs after the runtime-interface split. Component/reference nodes can also declare an optional `component_api_version` requirement.

The compiler validates contracts after document-shape validation and before expansion. The validation path rejects invalid public part exports, public part `control_id` values that belong to a different node, closed root-class policies that receive instance class appends, focus/binding contract targets that point at private internals, document or imported-style selectors that target an imported component's private `node_id`/`control_id`, unknown `:part(...)` references, and incompatible component API requirements. Semantic version compatibility is intentionally conservative for V1: matching major versions are required, and the exported minor version must satisfy the requested minor version.

`UiSelector` now parses `:part(label)` into a structured selector token so public part selectors are distinguishable from classes/states and can be validated as contract surface instead of stringly selector text. Contract validation scopes typed selectors such as `Card:part(label)` to matching imported components, while unscoped `:part(...)` and `#id` selectors stay conservative across all referenced imports. Existing flat schema migration, legacy template conversion, descriptor default-node instantiation, editor extraction/wrapping, palette insertion, and fixtures preserve behavior by seeding `UiComponentPublicContract::default()` and `component_api_version = None` unless an asset explicitly opts into M10 contracts.

The structured diagnostic productization slice keeps this runtime contract owner intact. `component_contract/diagnostic.rs` defines stable runtime codes and source paths, while `validation.rs` returns the first `UiComponentContractDiagnostic` through `component_contract_diagnostic(...)` and still converts the same diagnostic into `UiAssetError::InvalidDocument` for the compiler error path. `zircon_editor::ui::asset_editor::diagnostics` maps these runtime diagnostics into `UiAssetEditorDiagnostic` rows; `UiAssetEditorSession` stores them beside the legacy string diagnostics and `presentation_state.rs` uses `target_node_id` to select matching source-outline entries when possible.

## Follow-On Milestones

M10 foundation validation is accepted for the runtime asset compiler path after the expanded private-boundary gate covered imported style privacy, scoped multi-import public part selectors, closed root class policy enforcement, public part node/control coherence, binding/focus private target rejection, invalid semantic API strings, and API compatibility. The follow-up productization slice has accepted focused evidence for stable runtime diagnostic codes, editor projection of private selector/API mismatch/closed-root-class diagnostics, and root-class authoring through `component.root_class_policy`. Broader workspace green remains outside this M10 gate until unrelated owner blockers clear.

## M18 Binding Expression Semantics

M18 keeps `zircon_runtime::ui::template::asset::binding` beside component-contract, resource, localization, action-policy, and package behavior owners. The binding module owns compile-precondition validation and imports the serialized target assignment schema, restricted expression AST, and structured diagnostic DTOs from `zircon_runtime_interface::ui::template::asset::binding`. `UiBindingRef` remains the shared binding row for events/routes/actions, and gains `targets: Vec<UiBindingTargetAssignment>` with `serde(default)` so older assets keep loading while new assets can declare runtime-validated target semantics.

The accepted target kinds are `prop`, `class`, `visibility`, `enabled`, and `action_payload`. Prop targets are checked against the runtime component descriptor registry when a descriptor exists, with authored node props as a fallback for descriptor-less nodes. Class, visibility, and enabled targets require boolean expressions. Action-payload targets require an existing binding action payload and inherit the payload value kind when a payload entry exists.

`UiBindingExpression` intentionally supports only literals, `param.<name>`, `prop.<name>`, equality/inequality, boolean combinators, and parentheses. The module rejects unresolved refs, unsupported operators, invalid target names, and mismatched value kinds through `UiBindingDiagnosticCode::{InvalidTarget, InvalidValueKind, UnresolvedRef, UnsupportedOperator}`. `collect_asset_binding_report(...)` exposes the full report, while compiler preconditions use `validate_asset_bindings(...)` to convert the first runtime-owned binding error into `UiAssetError::InvalidDocument` before expansion.

When a runtime descriptor exists for a node's widget type, that descriptor is authoritative for `prop` targets and `prop.<name>` refs; authored node props are only a fallback for descriptor-less nodes. Action-payload targets must name an existing action payload field, and malformed single-character assignment/boolean operators are reported as `UnsupportedOperator` diagnostics.

M18 does not replace M21 action side-effect policy, does not execute arbitrary scripts, and does not move editor preview helper functions such as `concat`, `get`, `at`, or node-path mock expressions into runtime semantics. Those remain editor preview behavior until a later productization milestone defines a broader runtime evaluator.

M18 focused acceptance originally used `zircon_runtime/tests/ui_asset_binding_contract.rs` because the `zircon_runtime --lib` filtered harness was blocked by mixed UI DTO identities. The 2026-05-02 runtime-interface focused rerun superseded that blocker: `cargo test -p zircon_runtime --lib asset_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never` executed and passed. The production runtime library check and public integration contract still cover the binding report/compiler-precondition semantics with unrelated graphics/plugin warnings only.

## M12 Invalidation Graph And Compile Cache

M12 adds `zircon_runtime::ui::template::asset::invalidation` as the runtime-owned invalidation behavior layer above low-level `UiDirtyFlags`. The interface owns the neutral `UiAssetFingerprint`, `UiInvalidationSnapshot`, `UiInvalidationStage`, `UiInvalidationImpact`, `UiInvalidationReport`, and diagnostic DTO declarations; runtime owns deterministic fingerprint generation from asset documents/imports/resources, large-document diagnostics, and `UiInvalidationGraph::classify(...)` execution.

`UiInvalidationSnapshot` mirrors the inputs that can invalidate a compiled asset: root document fingerprint, widget import fingerprints, style import fingerprints, descriptor registry revision, component contract revision, and resource dependencies revision. `UiInvalidationGraph::classify(...)` compares snapshots and maps document/import/descriptor/contract/resource changes to parse, shape, import, descriptor, contract, resource dependency, selector, style, layout, render, interaction, and projection stages. Resource dependency changes are intentionally narrower than document-shape changes: they rebuild the compiled asset, dirty render output, and dirty editor projection without implying descriptor or selector drift. The stage impact then maps accepted stages back into runtime dirty consequences without replacing `UiDirtyFlags`.

`zircon_runtime::ui::template::asset::compiler::cache` adds `UiCompileCacheKey`, `UiAssetCompileCache`, and `UiCompileCacheOutcome`. `UiDocumentCompiler::compile_with_cache(...)` first runs document shape and component contract preconditions, then builds the exact cache key from the current compiler imports/descriptor registry, resource dependency collector, and the document. It returns an exact cache hit when possible, and otherwise falls back to the existing full `compile(...)` path while reporting the invalidation miss cause. The cache stores last invalidation snapshots by asset kind/id, so interleaved multi-document compiles compare a miss against the previous snapshot for the same asset instead of inventing cross-document deltas. Large-document diagnostics currently flag broad selector pressure and non-virtualized `ScrollableBox` child pressure with named thresholds.

M12 foundation validation is accepted for the runtime asset compiler path. M16 now consumes these cache-key and invalidation inputs for package validation metadata and the `UiCompiledAssetCacheRecord` cache index record; remaining product-facing work is higher-level editor visual performance surfacing and a real cross-process cache store that reads/writes those records across process runs.

## M15 Resource Reference Model

M15 adds `zircon_runtime::ui::template::asset::resource_ref` as the runtime-owned collection and validation behavior owner for UI fonts, images, media, and generic assets, while `UiResourceRef`, diagnostics, dependency rows, and related neutral contract declarations are canonical interface DTOs after the runtime-interface split. The serialized M1 contract is `UiAssetImports.resources: Vec<UiResourceRef>` with `serde(default)`, so existing asset documents keep loading while new resources can be declared explicitly. The collector's typed table classifier is intentionally narrower than generic table traversal: it recognizes resource tables by supported resource `kind`, explicit `uri`, or table-shaped `fallback`. M14 localized text refs that use string `fallback` remain localization metadata, and layout/component metadata tables with non-resource `kind` values such as `VerticalBox` remain ordinary TOML metadata instead of entering resource validation.

Typed TOML resource tables are strict contract data: they must provide `kind` and `uri`, may provide a fallback table, and `UiResourceRef::validate(path)` reports structured `UiResourceDiagnostic` errors for empty URIs, unsupported schemes, placeholder fallbacks without a URI, self-referential fallbacks, and placeholder fallback kind mismatch. The accepted URI schemes are `res://`, `asset://`, and `project://`.

`UiResourceKind::infer_from_path_and_uri(path, uri)` provides the shared inference rule for legacy-string collection: resource-like path names are considered before extension, `.font.toml` is recognized as a font, common font/image/media extensions map to their typed kinds, and otherwise supported-scheme resource URIs default to `GenericAsset`. `collect_document_resource_dependencies(...)` scans the root document plus the current compiler registered widget/style imports, explicit `imports.resources`, tokens, node props/params/layout, style overrides, child slot metadata, stylesheet declaration blocks, and component roots. This is package-input scope, matching the registered import registry already fingerprinted by M12 and reported by M16, not a reachability-only subset of imports touched during expansion. It validates typed TOML resource tables through `UiResourceRef` and stores deterministic `UiResourceDependency` rows on `UiCompiledDocument` through `resource_dependencies()`.

M12 consumes the collector through `resource_dependencies_fingerprint(...)`, `UiCompileCacheKey.resource_dependencies_revision`, and `UiInvalidationStage::ResourceDependency`. M16 back-half consumes the compiled document dependencies as deterministic `resource_dependencies` rows in `UiCompiledAssetDependencyManifest`. Resolver/file existence checks, resource diagnostics persistence beyond compile errors, editor dependency view, and runtime loader backends remain future productization.

M15/M16 resource reconciliation intentionally does not add a resource browser, resolver/file existence check, watcher/hot reload flow, image/font/media loader backend, or graphics/RHI changes. Those remain future productization milestones that must reuse the runtime typed collector and dependency manifest instead of creating parallel resource classification rules.

## M21 Action Policy And M14 Localization Surface

M21 adds `zircon_runtime::ui::template::asset::action_policy` as the runtime-owned package policy behavior surface for authored UI actions, while policy/report declarations are canonical interface DTOs after the runtime-interface split. It consumes existing `UiBindingRef` route/action data and classifies side effects as local UI, editor mutation, asset IO, scene mutation, external process, or network. Runtime package validation uses `UiActionHostPolicy::runtime_default()` and therefore only allows local UI actions; editor package validation uses `UiActionHostPolicy::editor_authoring()` and also allows editor mutation plus asset IO. The resulting `UiActionPolicyReport` is stored on `UiCompiledAssetPackageValidationReport`; this slice does not add editor policy-inspector UI and does not replace future M18 binding-expression semantics.

M14 adds `zircon_runtime::ui::template::asset::localization` as the runtime-owned collector behavior for localized text metadata, while localized text/report declarations are canonical interface DTOs after the runtime-interface split. `UiLocalizedTextRef` models authored localization tables with `text_key`, optional table/fallback data, and `UiTextDirection`; compiler preconditions validate empty localization keys before expansion. `collect_document_localization_report(...)` scans node props/layout/params and stylesheet declarations, emits deterministic `UiLocalizationDependency` rows for localized refs, and emits `UiLocalizationTextCandidate` rows for literal `text`/`label`/`title` values that should be considered for extraction. Component prop validation accepts a non-empty localized text table as a string-family prop without flattening it to fallback text, so compiled attributes and package manifests keep the structured localization metadata. Package validation stores the full `UiLocalizationReport`, and `UiCompiledAssetDependencyManifest.localization_dependencies` persists locale dependency rows beside widget/style/resource dependencies. Locale table resolver IO, external missing-key checks, editor locale preview, and extraction UI remain future productization.

## M16 Package Validation Surface

M16 adds `zircon_runtime::ui::template::asset::compiler::package` as the runtime-owned package validation surface for compiled UI asset metadata. `UiDocumentCompiler::validate_package(...)` runs the same document shape and component contract preconditions as full compile before building the M12 `UiCompileCacheKey`, then runs the existing compile path for validation and emits `UiCompiledAssetPackageValidationReport`.

The report contains a `UiCompiledAssetHeader` with source schema version, package/compiler schema versions, descriptor registry revision, component contract revision, root document fingerprint, and full cache key. It also contains a deterministic `UiCompiledAssetDependencyManifest` for registered widget/style imports plus compiled M15 package-input resource dependencies and M14 localization dependency rows, M21 action policy diagnostics, M14 localization reports, and runtime/editor profile retained and stripped sections. Runtime strips source/authoring/migration sections from the package report view; editor keeps them for authoring diagnostics.

M16 back-half adds a deterministic binary artifact envelope and a TOML package manifest writer/importer without changing compiler authority. `UiCompiledAssetArtifact` stores the package report and compiled `UiTemplateInstance` behind magic `ZRUIA016`, little-endian artifact schema version, little-endian payload length, and a UTF-8 TOML payload; the TOML payload avoids `bincode` assumptions for compiled values that contain `toml::Value`. `UiCompiledAssetCacheRecord` persists the M12 cache key, invalidation snapshot, artifact fingerprint, and byte length as the future cache index; `UiCompiledAssetPackageManifest` records the header, dependency manifest, cache record, and artifact entry. It now preserves M14 localization dependency rows and M21/M14 package reports through the same TOML surfaces. It still does not implement resource resolver IO, runtime loader backends, graphics/RHI resource consumption, editor action policy inspector UX, or editor locale preview UX.
