---
related_code:
  - zircon_editor/src/ui/host/builtin_views
  - zircon_editor/src/ui/host/view_registry.rs
  - zircon_editor/src/ui/host/editor_capabilities.rs
  - zircon_editor/src/ui/host/editor_subsystems.rs
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/startup/welcome_view.rs
  - zircon_editor/src/ui/workbench/view
  - zircon_editor/src/ui/workbench/snapshot/workbench
  - zircon_editor/src/ui/workbench/reflection/activity_descriptors.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/core/editor_extension/view_descriptor.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/ui/workbench/event/menu_item_binding.rs
  - zircon_editor/assets/ui/editor
tests:
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - zircon_editor/src/tests/host/pane_template_descriptor.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup/window_topology.rs
  - zircon_editor/src/tests/workbench/registry/instance_policy.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/performance/01/2026-08-19-editor-ui-workbench-view-descriptor-instance-generation-plugin-lifecycle-architecture-review.md
  - docs/plans/performance/01/2026-08-19-editor-ui-binding-compiled-intent-generation-architecture-review.md
  - docs/plans/performance/01/2026-08-19-editor-ui-binding-dispatch-single-domain-request-architecture-review.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/WorkflowOrientedApp/WorkflowTabFactory.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/WorkflowOrientedApp/WorkflowTabFactory.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/TabManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
  - dev/godot/editor/docks/editor_dock_manager.h
  - dev/godot/editor/docks/editor_dock_manager.cpp
  - dev/godot/editor/plugins/editor_plugin.h
  - dev/godot/editor/plugins/editor_plugin.cpp
  - dev/Fyrox/editor/src/plugin.rs
  - dev/Fyrox/editor/src/plugins/material/mod.rs
  - dev/Fyrox/editor/src/plugins/animation/mod.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentProvider.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Unity.RenderPipelines.Core.Editor.asmdef
refreshes:
  - docs/plans/optimize/zircon_editor/52-editor-builtin-view-window-descriptor-catalog-content-provider-capability-template-localization-product-integration-review.md
doc_type: review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 126 · Editor Builtin View、Window Descriptor Catalog、Content Provider、Capability、Template、Localization 当前源码刷新审查

## 1. 结论

当前 Editor 仍有一套可保留的描述符基础：`builtin_view_descriptors` 集中注册 Activity View/Window，描述符可以表达 instance policy、dock/slot、document kind、constraints、pane template 与 required capabilities；UI Asset、Animation 和 Diagnostics 的部分 session 证明真实 provider 可以建立在这一层之上。当前源码不是“完全没有视图系统”。

但 descriptor catalog 仍然只是 metadata registry，不是可证明的产品能力目录。`core/editor_extension::ViewDescriptor` 只有 id/title/category/document/template 字段；`ui/control::EditorUiControlService` 只保存 descriptor 与 route；`ViewRegistry` 注册/打开流程没有 typed content provider、spawn factory、session owner、close/reload receipt。functional panel/window 的 content kind 又在另一处按 raw ID 推断，未映射 ID 会落入 Placeholder，而 snapshot 仍可能把它标为非 placeholder。默认 layout、command、menu 和 open success 因而能证明“有入口”，不能证明“有可交互内容”。

本轮重新扫描旧编号 52 的完整当前范围，登记 **1 项 P0、40 项 P1、12 项 P2 与 32 个资格门**。Editor50 继续拥有 extension contribution/revoke，Editor13 拥有 layout restore/migration，Editor03/14/15/23/25 拥有各自 domain toolkit；本文只负责 catalog 的 content/provider/capability 闭包和 truthful availability。

## 2. 当前语料冻结

旧编号 52 的 54 个 evidence roots 展开为 372 个去重物理文件：

| 类别 | 文件 | 总行 | 非空行 | bytes | `#[test]` | `#[ignore]` | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Zircon source/test | 339 | 58,113 | 52,944 | 4,301,374 | 48 | 3 | `f537d1e934c613816d74608aeefcb33f0756eab50b98ff07c58740e740cc147b` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | 14 | 11,421 | 9,693 | 415,202 | 1 | 0 | `241e317875ff522ce4b188634a4fd25a220b156c24ad8b8b363a66deab1d7e05` |
| plan/docs | 19 | 7,270 | 5,634 | 791,344 | 0 | 0 | `d1519f4f721902efb00abc1474423a46bd2591ca71fbf1a51ccedcccb01c84ec` |
| 去重 union | 372 | 76,804 | 68,271 | 5,507,920 | 49 | 3 | `5a3ca28313205212c71c401008248bb61f636b842b134f09e3e0fe53b22206aa` |

指纹采用 normalized relative path + NUL + raw bytes + NUL；实施前必须重算。目录内的 48 个测试属性主要检查 metadata、placement 和 instance policy，不能替代 provider spawn、ZUI compile、domain session、reload 或 first-present E2E。

## 3. 参考约束

| 参考 | 可迁移约束 |
|---|---|
| Unreal WorkflowTabFactory/TabManager | tab factory 负责真实内容创建、owner、layout restore、close/reinvoke；metadata-only tab 不能算 opened。 |
| Godot EditorDockManager/EditorPlugin | dock/plugin 的 add/remove、输入、绘制、visibility、clear 和 owner 生命周期可观察。 |
| Fyrox plugin | 插件 panel 通过真实 plugin/editor container 接入，不是只写一条 descriptor。 |
| Bevy App/Plugin | plugin build/finish/cleanup phase 要与 resource ownership 绑定。 |
| Unity Graphics VolumeComponentProvider | provider registry 以类型、能力和创建器为闭包；缺 provider 的 component 不应出现在可用目录。 |

## 4. P0：目录可用性失真

### **P0-01** metadata-only descriptor 可作为 Available/open 成功发布

`ViewRegistry` 能注册 descriptor 并创建 `ViewInstance`，但 descriptor 没有 typed content provider 或 spawn factory；content kind 在目录外由 raw ID `match` 推断，未知项默认 Placeholder。当前 functional panel/window 中多项只有 `design_stack`、command 或 menu caller，真实 pane/window body、document session、toolkit 和 provider 并不存在。snapshot 还可能把 Placeholder 内容写成 `placeholder: false`，最终 UI 显示 Missing View 或固定 placeholder 文案。

重构为 compiled `ViewDefinition`：定义必须同时绑定 `ViewContentProvider`、template/payload/route contract、required capability、owner generation、document/session policy 和 close/reload compensation。缺 provider 的定义在 catalog compile/admission 阶段失败或明确标记 Unavailable，不能进入 shipping catalog、默认 layout、command palette 或功能统计。

## 5. P1：Content、Provider、Capability、Template（01-20）

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-01 | `ViewDescriptor` 没有 content factory/provider。 | 绑定 typed provider key、owner lease 与 spawn contract。 |
| P1-02 | `open_descriptor()` 的 Ok 早于 body/session 创建。 | 返回 Resolving/Opened/Presented/Failed/Recovered receipt。 |
| P1-03 | Content kind 由目录外 raw ID 二次推断。 | 将 content binding 固化进 compiled definition。 |
| P1-04 | 未知已注册 ID 默认降为 Placeholder。 | 编译期拒绝缺 binding，restore 才允许 unavailable placeholder。 |
| P1-05 | Placeholder 与 `placeholder: false` 互相矛盾。 | snapshot 使用 typed availability、reason、owner generation。 |
| P1-06 | pane constraints 被误当作内容语义。 | provider 明确 pane body、payload、route、interaction。 |
| P1-07 | functional panel 只有 layout consumer。 | 每项绑定 controller/toolkit/document session 或降级。 |
| P1-08 | functional window 只有 command/menu consumer。 | open command 必须追到 provider spawn 和 presented frame。 |
| P1-09 | Scene/Game window 存在 catalog 外零 caller。 | 合并到真实 provider 或从 shipping catalog 删除。 |
| P1-10 | UI Asset/Asset Browser 有并行 ID。 | 选择 canonical ID，以 alias migration 处理旧 layout。 |
| P1-11 | Prefab descriptor 正文仍声明资产编辑能力为 placeholder。 | 在 Editor03/44 完成前标 Unavailable，不得伪装 Available。 |
| P1-12 | Debug Observatory 与 Diagnostics 共享 payload 却分裂 ID。 | 定义 typed perspective variant 或合并 descriptor。 |
| P1-13 | Material Demo/Component Lab 复用 UI showcase payload。 | 样例与 Material authoring provider 分离。 |
| P1-14 | core 与 UI 存在两套 `ViewDescriptor`。 | 统一 schema，extension/builtin 使用同一 compiled type。 |
| P1-15 | descriptor 没有 schema/version/namespace。 | 加 owner namespace、version、alias、retirement、migration。 |
| P1-16 | required capability 是裸字符串列表。 | 使用 capability ID + provider generation + denial reason。 |
| P1-17 | capability snapshot 未绑定 BuildSet/session。 | availability 必须校验 BuildSet、project session、policy digest。 |
| P1-18 | template ID 存在但没有资源/schema/link 检查。 | catalog compile 验证 ZUI/template/payload/route 全链。 |
| P1-19 | template missing 只在呈现阶段暴露。 | admission 时预解析资源并返回 typed missing dependency。 |
| P1-20 | localization/icon key 没有 provider 级闭包。 | 编译并校验 localized label、icon、accessibility metadata。 |

## 6. P1：Instance、Layout、Lifecycle、Extension（21-40）

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-21 | `ViewInstance` 没有 provider/session owner。 | instance 绑定 `ViewInstanceId`、scope、provider lease。 |
| P1-22 | multi-instance policy 未验证 document/asset identity。 | factory 按 document/session key 去重或允许并存。 |
| P1-23 | layout restore 只恢复 descriptor/placement metadata。 | restore 生成 unavailable placeholder 或真实 provider，不静默成功。 |
| P1-24 | default design stack 是独立 ID/尺寸事实源。 | 从 compiled catalog 生成默认 layout。 |
| P1-25 | pane/window close 没有 provider shutdown receipt。 | close 走 quiesce、drain、dispose、terminal receipt。 |
| P1-26 | plugin reload 未和现有 view instance generation 绑定。 | owner generation 变化时撤销或迁移 instance。 |
| P1-27 | extension view 注册只验证重复 ID。 | 注册时校验 provider、template、capability、owner。 |
| P1-28 | extension view 没有 unregister/revoke 合同。 | Editor50 的 contribution ticket 贯穿 view mount/revoke。 |
| P1-29 | required capability 失败没有 stable public reason。 | 返回 capability missing/owner retired/build mismatch code。 |
| P1-30 | placeholder 可能进入命令 palette/功能计数。 | availability 过滤所有 command/menu/layout/metrics projection。 |
| P1-31 | menu/open success 不证明 presented frame。 | UI receipt 分离 Registered、Spawned、Presented。 |
| P1-32 | content provider callback 无 fault domain。 | panic/timeout/FFI fault 进入 provider quarantine。 |
| P1-33 | provider 回调可能持有 registry mutex。 | snapshot 外调用，避免锁内 ZUI/session/extension callback。 |
| P1-34 | view route 与 document route 没有 generation fence。 | route payload 携 instance/provider/document generation。 |
| P1-35 | view state 没有持久化 schema/currentness。 | versioned state document、migration、unknown field policy。 |
| P1-36 | unavailable view 没有 repair/retry/diagnostic action。 | 显示 bounded reason、retry、remove、restore-copy 操作。 |
| P1-37 | layout/command/menu/Welcome 各自维护 feature alias。 | canonical catalog 生成所有 surface projection。 |
| P1-38 | activity window 与 pane template 的 capability 语义不一致。 | 统一 capability closure 和 content family。 |
| P1-39 | catalog 没有全量 definition/provider 资格矩阵。 | CI 检查每个 ID 的 provider、resource、test、owner、locale、icon。 |
| P1-40 | 测试只验证 descriptor 数量和 placement。 | 增加真实 spawn/session/reload/restore/first-present 测试。 |

## 7. P2：长期能力

| ID | 能力 | 目标 |
|---|---|---|
| P2-01 | Catalog inspector | 展示 definition/provider/capability/owner generation。 |
| P2-02 | Provider health | 记录 spawn、present、fault、quiesce、reload 指标。 |
| P2-03 | View provenance | 追踪打开来源、operation、document、plugin 与 layout。 |
| P2-04 | Alias migration | 旧 ID、retired ID、owner move 可审计迁移。 |
| P2-05 | Missing dependency repair | template、icon、locale、provider 缺失可受控修复。 |
| P2-06 | Layout simulation | 在写入前模拟 capability/monitor/provider 结果。 |
| P2-07 | Availability policy | 按 BuildSet、project profile、trust、platform 动态裁剪。 |
| P2-08 | Accessibility metadata | label、role、shortcut、live region 与 provider 同代。 |
| P2-09 | Localization coverage | catalog label/icon/tooltip 的 culture matrix 与 pseudo preview。 |
| P2-10 | View session support bundle | 导出 instance、route、provider、template、failure receipt。 |
| P2-11 | Multi-window topology | window/viewport/document instance routing 有正式策略。 |
| P2-12 | Catalog performance budget | 大目录查询、layout restore、snapshot clone 的 p95/p99 预算。 |

## 8. 资格门

| Gate | 通过条件 |
|---|---|
| G-01 | 每个 Available definition 都有 typed content provider/factory。 |
| G-02 | 缺 provider 的定义 compile/admission fail-close。 |
| G-03 | open receipt 区分 Registered、Spawned、Presented、Failed。 |
| G-04 | content kind 不再由 raw ID 二次推断。 |
| G-05 | Placeholder availability 与 snapshot 标志一致。 |
| G-06 | provider/session/document/owner generation 绑定同一 instance。 |
| G-07 | required capabilities 有 BuildSet/session/policy digest 校验。 |
| G-08 | template、payload、route、interaction 资源全部可解析。 |
| G-09 | localization、icon、a11y metadata 有完整 key coverage。 |
| G-10 | default layout 从 catalog 生成，不维护第二份 ID 表。 |
| G-11 | restore 失败显示 typed unavailable，不伪造 opened。 |
| G-12 | close/reload/revoke 有 quiesce、drain、dispose receipt。 |
| G-13 | callback 不在 registry/template mutex 内执行。 |
| G-14 | provider panic/timeout/FFI fault 会 quarantine 并阻止继续调用。 |
| G-15 | route payload 带 instance/provider/document generation。 |
| G-16 | extension view 注册包含 provider/owner/capability validation。 |
| G-17 | extension revoke 能撤销 view、menu、layout、route 和 instance。 |
| G-18 | multi-instance policy 有 document/asset/session identity 测试。 |
| G-19 | command/menu/Welcome/open 端到端到真实 provider。 |
| G-20 | Scene/Game、Prefab、Material、Animation、Diagnostics aliases 有明确 canonical ID。 |
| G-21 | UI Asset/Asset Browser 并行 ID 有迁移和去重策略。 |
| G-22 | catalog unknown/retired/duplicate IDs 有 deterministic compile error。 |
| G-23 | capability unavailable 不进入功能统计与命令 palette。 |
| G-24 | first-present 失败能撤销 Presented/Ready projection。 |
| G-25 | provider replacement/reload 不会把旧 generation route 到新 session。 |
| G-26 | corrupt/oversized descriptor/template snapshot 有 bounded parser/quarantine。 |
| G-27 | layout restore、open、close 的 transaction receipt 可重放。 |
| G-28 | 真实 ZUI/resource compile 与 snapshot schema 进入 CI。 |
| G-29 | provider callback 的跨线程边界、取消和 deadline 有测试。 |
| G-30 | 多窗口、多 viewport、无 provider、能力降级矩阵有产品测试。 |
| G-31 | Unreal/Godot/Fyrox/Bevy/Unity 对照只验证结构语义，不外推闭源实现。 |
| G-32 | 重新计算 372-file union fingerprint，并由独立 review 检查目录闭包。 |

## 9. 建议实施顺序

1. 先把 `ViewDescriptor`、content kind、template、capability 和 provider 合并成一个 compiled definition，修复 P0-01 的 Available/open 失真。
2. 接入真实 provider/session owner，先以 UI Asset、Animation、Diagnostics 作为样板，再处理 Prefab、Material、Scene/Game alias。
3. 将 extension registration、layout restore、command/menu/Welcome、reload/revoke 全部改为 catalog projection，删除 raw-ID 第二 authority。
4. 最后补 localization/icon/a11y、fault domain、multi-window 和性能基准；metadata 数量测试不能作为产品完成条件。

## 10. 复核结论

当前 View/Window 目录有较好的描述符和布局骨架，但“注册成功”仍然大于“内容可用”。工程级 Editor 必须让一个定义同时拥有内容 provider、能力闭包、模板/路由资源、session owner、生命周期 receipt 和真实呈现证据；否则默认 layout、菜单和测试只是在放大假功能。Editor126 的 1 项 P0、40 项 P1、12 项 P2 与 32 个资格门应在后续实现中作为目录准入门槛，而不是继续增加更多 descriptor 数量。
