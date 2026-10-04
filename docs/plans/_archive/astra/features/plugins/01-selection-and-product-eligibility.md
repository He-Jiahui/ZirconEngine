---
status: in_progress
review_date: 2026-09-26
source_recheck_required: true
plan_sources:
  - docs/plans/astra/optimize/01-review-and-repair.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/20-plugin-sdk-example-native-editor-fixture-test-carrier-artifact-isolation-product-truth-review.md
---

# Plugin Selection 与产品资格收敛

## 目标与边界

本计划修复首方 Runtime/Editor provider catalog 静默丢弃 selection 的问题，并用强类型 package role 阻止 sample 与 test fixture 进入 Editor 产品 catalog 及 linked feature export 的产品注册链。范围限于共享 selection outcome、两个首方 provider catalog、`zircon_app` 当前消费链、plugin package declaration/manifest role、Editor build catalog 的 role admission，以及 linked feature source admission。Native artifact trust、安装、Marketplace、Hub、render internals 与 Tooling 不在本计划内。

共享合同归 `zircon_runtime::core::framework::project`；首方 catalog 负责为每个目标内且启用的 selection 生成唯一 outcome；`zircon_app` 在使用 registration 前执行 required fail-closed。Optional 的 unsupported provider 保留为可观察 outcome，但不会阻止组合。Package role 与 package kind、maturity、category 分离；Editor build catalog 只接纳产品资格允许的 role，不使用 package ID 或目录名黑名单。

## M1 · Typed Selection Resolution

实施切片：

1. 在 Runtime framework 增加 selection resolution outcome/report，保留 selection identity、required 标志与 `Resolved`、`InvalidId`、`Duplicate`、`Unsupported` 终态。
2. Runtime 与 Editor 首方 catalog 返回 typed report，每个目标内启用 selection 恰有一个 outcome；重复项不得静默覆盖或重复注册。
3. `zircon_app` 适配当前 Runtime/Editor 消费者；required Runtime provider 非 `Resolved` 阻止组合，显式配置 `editor_crate` 的 required Editor provider 同样阻止 Editor startup；optional `Unsupported` 仍可观测且允许降级。
4. 增加 catalog 与 App 回归测试，覆盖 invalid、duplicate、required missing、optional unsupported 和 resolved registration 投影。

Testing Stage：由父 Astra 批次执行 Runtime framework、两个首方 catalog 与 App 的 package check/focused tests。失败先从共享 outcome 或 catalog resolution 修复，再向 App 组合测试复验；未取得受管终态证据前保持 `implemented_pending_validation`。

## M2 · Product Package Eligibility

实施切片：

1. 增加与结构 kind、maturity 独立的强类型 package role，缺失字段按现有 production package 兼容为 Production。
2. Plugin SDK declaration 将 role 投影到 package manifest；SDK example 标记 Sample，native/editor contribution carrier 标记 TestFixture。
3. Editor build catalog 解析 role 并只接纳 Production/允许的 DeveloperTool，Sample/TestFixture 必须产生确定性排除。
4. 增加 manifest roundtrip、declaration projection 与 build catalog admission 测试，证明普通生产插件保留、carrier 被排除且无名称黑名单。

Testing Stage：由父 Astra 批次执行 Runtime package manifest、Plugin SDK、Editor build catalog 的 focused tests与生成 manifest 一致性检查。任何生成文件漂移先修 declaration 单一来源；不得通过手改名单绕过 role admission。

M2 的 `implemented_pending_validation` 只覆盖上述 declaration/manifest 与 Editor build catalog 的准入；它不证明 linked feature export 的 provider role 已从独立 package manifest 传到 App。

## M3 · Linked Feature Source Role Admission

当前源码缺口：

- `zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/feature.rs` 的 `from_feature` 与 `zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/native.rs` 的 `from_native_feature_manifest` 都把 provider role 默认为 Production。`zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report/registrations.rs` 会从已发现的 package manifest 覆盖 native report 的 role。
- `zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs` 的 `from_project_manifest` 只接收 `ProjectManifest` 和 profile 名称；`zircon_runtime/src/plugin/export_build_plan/export_build_plan.rs` 的 `ExportLinkedRuntimeCrate` 只保存 crate/path/kind/provider ID；`zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs` 生成的 feature provider 只附加 provider ID；`zircon_app/src/entry/export_bootstrap.rs` 的 provider 执行后也只覆盖 ID。
- `zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/provider.rs` 与 `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/merge.rs` 已按 role 过滤。缺口在 linked producer 的来源绑定，而非 catalog 过滤谓词。

实施切片：

1. 为 linked export 规划增加由宿主提供的源 package admission 输入。按实际 linked crate 路径在受信任的 `project_root/zircon_plugins` 来源内定位独立 `plugin.toml`，校验普通文件、根内路径、唯一 package 身份、feature owner/provider 关系及对应 runtime module crate；嵌套首方 feature 只有在其 owner manifest 明确声明该 feature 且 crate 位于该 owner package 内时才可继承 owner role。缺失、歧义、路径不符或不合资格 role 必须在生成代码前形成确定性阻断；不能从项目 selection 的 ID、crate 名、provider 函数返回值推断 role。
2. 将已准入 manifest 的 role、来源路径和内容摘要作为 linked source receipt 传入 `ExportLinkedRuntimeCrate` 与 generated bootstrap。生成的 `ExportRuntimePluginFeatureRegistrationProvider` 必须带该 role；App 的 `into_report` 必须以 receipt role 无条件覆盖 `from_feature`/`from_native_feature_manifest` 的默认 Production。Sample/TestFixture 不得满足产品 feature selection 或注册 runtime module；required selection 产生 fatal，允许的 Production/DeveloperTool 保留现有行为。
3. 迁移 `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs` 的普通与 native-aware export 规划/构建调用、`zircon_runtime/src/bin/zircon_export_validate/run.rs` 入口及 `zircon_runtime/src/plugin/export_build_plan/materialize/mod.rs` 中无 source root 的 materialize/preview/write-generated 路径，使它们在 linked feature receipt 缺失或来源改变时 fail closed。Editor native-aware 物化目前使用 native staging root，不能把它当作 linked 源 manifest；`zircon_runtime/src/plugin/export_build_plan/materialize/package_lookup.rs` 的 inventory 也仅服务 native package，不保存供 linked bootstrap 使用的 manifest role。
4. 增加 source-root fixture 回归：同一 required external feature 分别由 Production、Sample、TestFixture manifest 提供，检查规划诊断、生成 provider role stamp、App catalog/module 准入与 required fatal；同时覆盖缺失/歧义 manifest、crate 路径越界、owner/feature/module 不匹配以及来源变更。聚焦位置为 `zircon_runtime/src/tests/plugin_extensions/export_build_plan_feature_provider.rs`、`zircon_app/src/entry/tests/export_bootstrap.rs` 和 `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/derived_projection/tests.rs`。

现有 `zircon_plugins/plugin_sdk_examples/plugin.toml`、`zircon_plugins/editor_contribution_fixture/plugin.toml` 与 `zircon_plugins/native_dynamic_fixture/plugin.toml` 标记了 Sample/TestFixture，但没有当前随产品交付的 linked feature registration carrier。已知首方外部 feature 的 provider ID 还可能只是 owner package 内的虚拟身份：`zircon_plugins/sound/plugin.toml` 声明 `sound_timeline_animation_track`，实际 crate 位于 `zircon_plugins/sound/features/timeline_animation_track/runtime`；不能只按 provider ID 拼接目录并视为 manifest 证据。M3 source and focused regressions are implemented pending managed validation. The declared external sound_timeline_animation_track provider has no independent plugin.toml or source package at zircon_plugins/sound_timeline_animation_track; its nested crate at zircon_plugins/sound/features/timeline_animation_track/runtime cannot establish that external role. A synchronized owner-embedded declaration/manifest migration or a real independent carrier is still required before that opt-in can export.

The ordinary Editor `generate_export_plan`/`execute_export_build` API has no project-root argument; linked features receive an explicit `MissingSourceRoot` fatal result. The current product UI uses the project-root-aware native-aware route. A root-bearing ordinary API migration remains open for linked feature export.

Generated Cargo feature dependencies use the absolute admitted source path, so source-template builds depend on the original project tree. Canonical planning binds the complete export payload to an in-memory admission proof; the proof is deliberately omitted from serialization, so deserialized or edited plans must be replanned before materialization. Materialization also checks generated feature provider calls and Cargo dependency paths against admitted receipts. The receipt rechecks `plugin.toml` and the crate `Cargo.toml` before materialization; it does not snapshot source files or seal changes between materialization and the later Cargo build. Portable source packaging and build-time drift closure remain open.

## Combined Acceptance

1. 目标内每个启用 selection 都有一个终态 outcome；required Runtime selection 与显式 required Editor provider 的 invalid/duplicate/unsupported 阻止对应 App composition，optional unsupported 保持 typed degraded。
2. Resolved outcome 与 registration package identity 一致；Runtime 与 Editor catalog 使用同一 framework contract。
3. M2 的 Production package 仍进入 Editor build catalog；Sample/TestFixture 从该 catalog 与默认加载行确定性排除。
4. 当前 carrier 的 role 由 declaration 和 manifest 同源表达，测试不依赖 package 名称、目录名或 `category = sdk`。
5. M3 的 linked feature role 来自已准入的独立 source manifest/receipt，并在 generated bootstrap 与 App report 中保持一致；Sample/TestFixture 不满足产品 feature selection，required 失败，Production 保留。缺失或变化的来源不能由默认 Production 放行。
6. 格式、diff、源码 guard、自审通过；Cargo、真实产品目录与测试结果只由后续受管 combined validation 接受。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | Typed selection resolution | `implemented_pending_validation` | 2026-09-05 | Framework report 单元覆盖 resolved/invalid/duplicate/unsupported、optional degraded 与 scoped required fail-closed；Runtime/Editor catalog 已迁移到同一 resolver；App Runtime composition 与 Editor startup 已执行 checked conversion。 |
| M2 | Product package eligibility | `implemented_pending_validation` | 2026-09-05 | `PluginPackageRole` 完成 manifest/descriptor/SDK projection；三个 carrier manifest 已标记；Editor generated catalog 以 role admission 排除 carrier，生产 Navigation 保留。`rustfmt` 与 `git diff --check` 已通过；Cargo 留给父 Astra combined validation。 |
| M3 | Linked feature source role admission | `in_progress` | — | Source-root admission, typed diagnostics, manifest/feature/crate identity checks, role and digest receipts, generated/App role stamping, and pre-materialization drift checks are implemented with focused regressions. Managed Cargo, App product composition, real carrier migration, and product acceptance remain open. |
