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
  - docs/plans/optimize/zircon_editor/126-editor-builtin-view-window-descriptor-catalog-content-provider-capability-template-localization-current-source-review.md
doc_type: review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 173 · Editor Builtin View、Window Descriptor Catalog、Content Provider、Capability、Template、Localization 当前源码复核

## 1. 结论

Editor52/126 的主裁决仍成立：当前系统有可保留的 descriptor、layout、template metadata、capability filter 和若干真实 domain session，但 `ViewRegistry` 本质上仍是 metadata/instance registry，不是工程级 content-provider catalog。`open_descriptor()` 在没有创建 pane body、document/toolkit session、native content 或 first-present receipt 的情况下即可返回 `Ok(ViewInstance)`；snapshot 再按 raw descriptor ID 推断 `ViewContentKind`。因此“描述符存在”“命令能打开”“窗口进入 layout”“ZUI 文件存在”都不能单独证明产品功能可用。

本轮逐文件复核 40 个 builtin descriptor：22 个 ActivityView、18 个 ActivityWindow。`descriptor_content_kind()` 只覆盖其中 21 个，余下 **19 个已注册条目落入 `ViewContentKind::Placeholder`**，但 `resolve_view_tab()` 仍写出 `placeholder: false`。10 个 functional panel 只有 `design_stack`/测试侧实例事实，7 个 functional window 主要只有 command/menu/topology 事实；其中 Prefab 的 presentation 仍明确写着 asset-specific tooling 是 placeholder。当前最危险问题不是缺少更多 descriptor，而是 catalog 对 Availability、Opened 和 Presented 的语义失真。

本轮不新增 canonical finding，继续由 Editor52 作为 owner，并刷新 Editor126。当前状态为：

| 等级 | Open | Partial | Closed | 合计 |
|---|---:|---:|---:|---:|
| P0 | 1 | 0 | 0 | 1 |
| P1 | 33 | 7 | 0 | 40 |
| P2 | 12 | 0 | 0 | 12 |
| 资格门 | 22 Fail | 9 Partial | 1 Pass | 32 |

Tooling 按用户要求排除，后续另行迁移到 Rust。本轮只做 review 和文档，没有修改实现，也没有查询、轮询、等待或实时跟踪协调器。

## 2. 当前语料冻结与方法

冻结点为 HEAD `ea35974cdf64068f6789010451d20bbf69e0a29d`、2026-08-27T16:42:53+08:00。共享工作树冻结时有 8,206 个 status 条目；下表和结论均以当前磁盘为准，不假设脏改动已经提交，也不覆盖其他会话的修改。

| 类别 | 文件 | 总行 | 非空行 | bytes | `#[test]` | `#[ignore]` | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Zircon source/test/assets | 339 | 58,476 | 53,302 | 4,327,035 | 51 | 4 | `1543dc929cfd2a4720ec0c08c13ed0e71e294c3dc60e364aaeb60e72ee7c7706` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | 14 | 11,421 | 9,693 | 415,202 | 1 | 0 | `241e317875ff522ce4b188634a4fd25a220b156c24ad8b8b363a66deab1d7e05` |
| plan/docs | 19 | 7,270 | 5,634 | 791,344 | 0 | 0 | `d1519f4f721902efb00abc1474423a46bd2591ca71fbf1a51ccedcccb01c84ec` |
| 去重 union | 372 | 77,167 | 68,629 | 5,533,581 | 52 | 4 | `7b9ac47c7395bce1edab521bc3661ed42828e623bd43f3cca08e255dd51ef85e` |

指纹算法仍为 normalized relative path + NUL + raw bytes + NUL。复核方法包括：展开所有 frontmatter owner roots；逐文件阅读 descriptor/registry/open/restore/snapshot/projection/layout/extension/template/session/test 路径；抽取所有 `res://ui/editor/*.zui`；对 19 个 unmapped ID 做全生产 Rust caller 扫描；重读 14 个本地参考文件的 provider、spawn、dock、plugin lifecycle 和 owner boundary。没有运行 Cargo、Editor、ZUI compiler、native window、reload、fault、scale、soak 或跨引擎 benchmark，因此静态存在和结构测试不会被写成动态通过。

## 3. 40 项目录闭包

### 3.1 数量与内容映射

| 目录组 | 数量 | 当前事实 | 工程判定 |
|---|---:|---|---|
| Direct ActivityView | 12 | Project、Hierarchy、Inspector、Scene、Game、Assets、Plugin、Build、Generated、Console、Diagnostics、Timeline 有 descriptor；部分有 pane template/native projection。 | 可保留壳层，不等于统一 provider。 |
| Functional ActivityView | 10 | Prefab/Material/UI/Animation 的双 panel 加 Asset Preview/Metadata；只设置 ID、title、slot、constraints、icon。 | 10/10 无 descriptor-bound provider/template/session。 |
| Direct ActivityWindow | 9 | Workbench、Prefab、Asset Browser、showcase、Material demo/lab、Debug Observatory、Animation Sequence/Graph。 | 部分有模板或真实 domain presentation，仍由 raw ID 映射。 |
| Functional ActivityWindow | 7 | Scene/Game、Prefab、Material、UI Asset、Animation、Asset Browser、Diagnostics window。 | command/menu/topology 可达，但没有窗口级 content provider/session contract。 |
| 外加 builtin | 2 | `editor.ui_asset` 与 `editor.welcome` 在 catalog 汇总末尾加入。 | 有实际产品路径样板，但仍未成为 provider-bound definition。 |
| 合计 | 40 | 22 ActivityView + 18 ActivityWindow。 | 目录数量完整性不能替代产品完整性。 |

`descriptor_content_kind()` 当前显式覆盖 21 个 builtin ID。下列 19 个 catalog ID 未映射并默认 Placeholder：

| 组 | ID |
|---|---|
| Functional panel | `editor.prefab.viewport`、`editor.prefab.inspector`、`editor.material.graph`、`editor.material.preview`、`editor.ui.designer`、`editor.ui.source`、`editor.animation.timeline`、`editor.animation.graph`、`editor.asset_preview`、`editor.asset_metadata` |
| Functional window | `editor.scene_game_window`、`editor.prefab_editor_window`、`editor.material_editor_window`、`editor.ui_asset_editor_window`、`editor.animation_editor_window`、`editor.asset_browser_window`、`editor.diagnostics_window` |
| Direct window | `editor.debug_observatory`、`editor.workbench_window` |

### 3.2 Caller 与实例事实

| 路径 | 当前证据 | 差距 |
|---|---|---|
| `design_stack` | 10 个 functional panel 都在 preset 中出现；Material 等窗口布局会生成 `editor.material.graph#material_editor` 一类 instance ID。 | production shell 并不调用这套 `default_view_instances()`；它主要被 registry helper 和测试消费。 |
| builtin shell | `ensure_builtin_shell_instances()` 只恢复 11 个固定实例：Assets、Plugins、Hierarchy、Inspector、Console、Runtime Diagnostics、Performance、Build、Generated、Game、Scene。 | functional window 内部 layout 的实例没有由同一 production catalog materialize。 |
| command/menu | Prefab、Material、UI Asset、Animation、Asset Browser、Diagnostics、Debug Observatory 有默认 command/menu ID。 | caller 只追到 `manager.open_view()`，没有 provider spawn/present receipt。 |
| Welcome | 打开 `editor.asset_browser_window`、`editor.ui_asset_editor_window`。 | Welcome action 可达不证明 Asset Browser/UI Editor window body 已绑定其 domain session。 |
| topology test | Material window 可生成 floating native host；Asset Browser/Diagnostics 可生成 exclusive page。 | 只验证 placement，不验证内容、交互、session 或 first-present。 |
| reflection/layout | Debug Observatory 仅有 name mapping；Workbench window 用于 root layout。 | 都仍可能被 snapshot 投影成 `Placeholder` kind 且 `placeholder: false`。 |

`default_view_instances()` 能制造与 design stack 对应的 metadata instance，但生产 workspace/reset 路径走的是 `ensure_builtin_shell_instances()`。这两套表形成第二 authority：默认 layout 可以引用 catalog 中存在、但 session 未 materialize 且没有 provider 的 ID，最终表现为 Missing View 或固定 placeholder 文案。

## 4. 当前已实现的真实进展

以下内容值得保留，但只能把对应 finding 判为 Partial：

1. `PaneTemplateSpec` 已把 shell 与 body 拆开，并携带 document、payload kind、route namespace、interaction mode；16 个从当前 descriptor 源码抽取的 ZUI URI 全部存在，共 2,771 行、137,247 bytes。该结果只证明物理依赖存在，不证明 parse/compile/schema/binding/route/a11y/localization 成功。
2. Extension template contribution 会检查非空、trimmed、`.zui` 后缀；pane data source 必须引用已登记 template；完整 replacement 在 candidate map 中构造后原子替换，并校验 existing view 的 template link。
3. Extension view 会把 matching template 绑定为 `TemplateV2` pane，并携 required capabilities；但 `validate_extension_view_descriptors()` 本身仍只检查重复 ID，未检查 provider、owner lease、schema、resource compile 或 callback policy。
4. ContributionStore 已有 ticket/owner/generation；但是 `ViewInstance` 没有这些字段，ViewRegistry 也没有 unregister/revoke consumer，因此 generation 还不能 fence 已打开 instance 或 route。
5. Template pane data source 在调用 `source.snapshot()` 前会释放 shell mutex。这是 G-13 的局部正确实现；它没有 `catch_unwind`、deadline、cancel 或 quarantine，也不能代表尚不存在的通用 content provider callback 已安全。
6. `close_view()` 会执行 document close begin/commit，并清理 animation/UI Asset session 与 dependency generation；但它返回 bool，没有 provider-wide quiesce/drain/dispose receipt，layout 失败和跨集合清理仍不是共同事务。
7. `pane_projection` 当前只构造 active content kind 的 native body，capability set 也采用 HashSet union + sorted snapshot，capability error 使用单输出 buffer。这些是局部性能改进，不改变 catalog/provider authority 缺失。

## 5. 关键断路

### 5.1 Register/Open/Restore 仍是 metadata transaction

`ViewDescriptor` 没有 provider/factory、owner、generation、namespace、schema version、availability 或 admission result；`ViewInstance` 也只有 instance/descriptor/title/JSON/dirty/host。`register_view()` 只验证 duplicate ID；`open_descriptor()` 只校验 raw string capability、single-instance index，然后插入 metadata instance；`restore_instance()` 同样只验证 descriptor/capability 后插入。没有任何一步创建 pane controller、document toolkit、domain session 或 native body。

`layout_commands::attach_instance()` 先把 instance 放进 session，再执行 layout/native window 变化；后续失败没有 rollback receipt。`close_view()` 则分散删除 session、animation、UI Asset、dependency generation 和 registry。工程级 open/close 需要一个跨 provider/session/layout/window 的可补偿事务，而不是多个容器的顺序修改。

### 5.2 Snapshot/Projection 会制造错误可用性

只有 instance 或 descriptor 缺失时，`resolve_view_tab()` 才调用 `placeholder_view()` 并写 `placeholder: true`。只要 descriptor 已注册，即使 `descriptor_content_kind()` 返回 `Placeholder`，snapshot 仍写 `placeholder: false`。presentation 的 Placeholder 文案又声称 descriptor unavailable，状态与原因互相冲突。Prefab 更明确地显示“host slot ready，asset-specific tooling still placeholder”，但 descriptor 仍可进入 Available catalog。

### 5.3 Capability 不是产品闭包

`required_capabilities: Vec<String>` 由 `builtin_view_descriptors()` 的 ID side-match 后置附加；列表没有 typed ID、provider generation、BuildSet、project session、trust/platform policy 或 stable denial code。代码还保留 `editor.animation_timeline` 这个与 catalog 当前 `editor.animation.timeline` 不同的 alternate ID。`EditorSubsystemReport` 对 unknown requested string 直接视作 custom enabled，不能证明对应 provider 已安装或兼容。

### 5.4 Template 与 provider 仍分离

`ActivityWindowTemplateSpec` 只有一个 document string。Builtin 资源未在 catalog admission 阶段执行真实 ZUI compile/schema/link；extension 只验证字符串形状和贡献表引用。`pane_projection` 必须继续以巨大 `ViewContentKind` match 汇总所有 domain data，说明模板不是独立拥有 lifecycle/session 的 provider，目录也无法回答“谁创建、谁销毁、谁恢复、谁隔离故障”。

### 5.5 测试证明的是结构，不是产品完成

五个 canonical test 文件主要验证 descriptor 存在、kind/host/constraints/template metadata、single/multi-instance policy、capability filtering 和窗口拓扑。它们没有覆盖 40 个定义的 provider admission、真实 body spawn、document identity、first-present、restore failure、reload generation、revoke teardown、callback panic/timeout 或 all-ID availability truth。当前 51 个 test attribute 与 4 个 ignored performance test 不能关闭这些缺口。

## 6. 本地参考实现约束

| 参考 | 当前源码证据 | 对 Zircon 的约束 |
|---|---|---|
| Unreal `FWorkflowTabFactory` / `FTabManager` | `SpawnTab()` 先调用 `CreateTabBody()`；spawner 同时提供 `OnSpawnTab`/`CanSpawnTab`；TabManager 可 register/unregister，并在 invoke 时复用 live tab 或真实 spawn。 | Zircon 的 open success 必须晚于真实 content spawn，provider 必须可撤销并决定可用性。 |
| Godot `EditorDockManager` / `EditorPlugin` | manager 持有真实 `EditorDock`，明确 add/remove/open/close/floating；plugin 暴露 make_visible/edit/handles/clear/enable/disable。 | descriptor、UI control 与 plugin owner 生命周期不能分裂。 |
| Fyrox EditorPlugin / Material / Animation | plugin 有 start/exit/sync/UI message；Material/Animation plugin 持有 `Option<...Editor>`，按需创建、同步、关闭并 destroy。 | domain editor session 应由 provider owner 持有和终结，而不是只由 layout ID 暗示。 |
| Bevy Plugin | build/ready/finish/cleanup 是显式 phase。 | provider registration、ready、cleanup 和 owner generation 需要 phase contract。 |
| Unity Graphics `VolumeComponentProvider` | provider 同时绑定 target 与 target editor，构造 filtered type tree，并在选择后真正 AddComponent；asmdef 明确 assembly boundary。 | catalog 项必须把查询、能力、owner 边界和实际创建动作闭合。 |

这些对照只用于抽取可观察的结构约束，不外推闭源行为，也不宣称 Zircon 必须复制其 API。

## 7. P0 状态

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P0-01 | **Open** | metadata-only descriptor 可进入 Available/open success；19 个已注册 ID 映射为 Placeholder，而 snapshot 仍可标 `placeholder: false`。 | 建立 compiled `ViewDefinition`；缺 provider/template/schema/owner 的定义 admission fail-close 或明确 Unavailable，不得进入 shipping catalog、默认 layout、command palette 或功能统计。 |

## 8. P1 状态（01-20）

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P1-01 | **Open** | `ViewDescriptor` 无 content factory/provider。 | 绑定 typed provider key、owner lease、spawn contract。 |
| P1-02 | **Open** | `open_descriptor()` 在 body/session 创建前返回 Ok。 | 返回 Resolving/Spawned/Presented/Failed/Recovered receipt。 |
| P1-03 | **Open** | content kind 仍由目录外 raw ID 推断。 | binding 编译进 definition。 |
| P1-04 | **Open** | unknown registered ID 默认 Placeholder。 | admission 拒绝缺 binding；只允许 restore-only unavailable。 |
| P1-05 | **Open** | Placeholder kind 可与 `placeholder: false` 同时出现。 | 使用 typed availability/reason/generation。 |
| P1-06 | **Open** | constraints 继续代替 content 语义。 | provider 明确 body、payload、route、interaction。 |
| P1-07 | **Open** | 10 个 functional panel 无 provider/session。 | 逐项绑定 toolkit/controller/document session 或降级。 |
| P1-08 | **Open** | 7 个 functional window 只有入口/placement。 | command 追到真实 spawn 与 presented frame。 |
| P1-09 | **Open** | Scene/Game window 无目录外 production caller。 | 合并真实 provider 或删除 shipping definition。 |
| P1-10 | **Open** | UI Asset/Asset Browser 存在并行 ID。 | canonical ID + alias migration。 |
| P1-11 | **Open** | Prefab presentation 明示 tooling placeholder。 | Editor03/44 完成前标 Unavailable。 |
| P1-12 | **Open** | Debug Observatory 复用 Diagnostics payload 但分裂 ID。 | typed perspective variant 或合并。 |
| P1-13 | **Open** | Material demo/lab 使用 showcase content family。 | 样例与 Material authoring provider 分离。 |
| P1-14 | **Open** | core extension 与 workbench 各有一套 `ViewDescriptor`。 | 合并为同一 versioned schema/compiled type。 |
| P1-15 | **Open** | 无 schema/version/namespace/alias retirement。 | 加 owner namespace、version 和 migration。 |
| P1-16 | **Open** | capability 是 ID side-match 后附加的裸字符串。 | typed capability + provider/build/session generation。 |
| P1-17 | **Open** | snapshot 不绑定 BuildSet/project/trust/platform policy。 | 编译 composite availability closure。 |
| P1-18 | **Partial** | 16 个 builtin ZUI 物理存在；extension 有 document 形状与 link validation。 | 增加真实 resource parse/compile、schema、payload、route、interaction link。 |
| P1-19 | **Partial** | extension replacement 会拒绝贡献表内 missing template；builtin/物理文件/compile 仍在呈现期。 | catalog admission 预解析并返回 typed dependency error。 |
| P1-20 | **Open** | label/icon/a11y 没有 provider 级完整性闭包。 | 编译 locale/icon/accessibility coverage。 |

## 9. P1 状态（21-40）

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P1-21 | **Open** | `ViewInstance` 无 provider/session owner。 | 绑定 instance/scope/provider lease。 |
| P1-22 | **Open** | multi-instance 只按 bool，不含 asset/document identity。 | factory 按 qualified session key 去重/并存。 |
| P1-23 | **Open** | restore 只插入 metadata。 | 恢复真实 provider 或 typed unavailable placeholder。 |
| P1-24 | **Open** | design stack、builtin shell、catalog 是多份 ID authority。 | 默认 layout 从 compiled catalog projection 生成。 |
| P1-25 | **Partial** | document close 与两类 domain session cleanup 已存在。 | 统一 quiesce/drain/dispose/terminal receipt 和失败补偿。 |
| P1-26 | **Partial** | contribution store 有 generation 和 atomic template replacement。 | generation 绑定 existing instance/session/route 并迁移或撤销。 |
| P1-27 | **Partial** | extension 有 template/source/capability 贡献校验；view admission 仍只查 duplicate。 | 同时校验 provider、owner、resource compile、callback policy。 |
| P1-28 | **Partial** | contribution ticket/owner 已存在。 | ticket 贯穿 view registry unregister、instance teardown、layout repair。 |
| P1-29 | **Open** | capability failure 仍是拼接 String。 | stable public denial code + bounded diagnostics。 |
| P1-30 | **Open** | placeholder/不可用项没有统一 surface filtering。 | catalog 生成 command/menu/layout/metrics projection。 |
| P1-31 | **Open** | open/menu success 不证明 present。 | 分离 Registered/Spawned/Presented/Ready。 |
| P1-32 | **Open** | callback 无 panic/timeout/FFI fault domain。 | bounded invocation + quarantine。 |
| P1-33 | **Partial** | template data source snapshot 已在 shell mutex 外调用。 | 推广到全部 provider，并补 panic/deadline/cancel fence。 |
| P1-34 | **Open** | route 不携 instance/provider/document generation。 | generation-qualified route envelope。 |
| P1-35 | **Open** | instance payload 无 versioned state schema/currentness。 | 独立 state document、migration、unknown-field policy。 |
| P1-36 | **Open** | unavailable view 无 retry/remove/repair-copy contract。 | typed reason 与 bounded repair action。 |
| P1-37 | **Open** | layout/command/menu/Welcome 各维护 raw alias。 | 全部由 canonical catalog 投影。 |
| P1-38 | **Open** | window template 与 pane capability family 不统一。 | 统一 definition capability closure。 |
| P1-39 | **Open** | 无 40 项 provider/resource/owner/locale/icon CI 矩阵。 | 生成并强制 catalog qualification manifest。 |
| P1-40 | **Open** | 测试仍以 metadata/placement/instance policy 为主。 | 40 项真实 spawn/session/reload/restore/first-present E2E。 |

## 10. P2 状态

| ID | 状态 | 能力 | 目标 |
|---|---|---|---|
| P2-01 | **Open** | Catalog inspector | 展示 definition/provider/capability/owner generation。 |
| P2-02 | **Open** | Provider health | spawn/present/fault/quiesce/reload 指标。 |
| P2-03 | **Open** | View provenance | 记录 open source/operation/document/plugin/layout。 |
| P2-04 | **Open** | Alias migration | 旧 ID、retired ID、owner move 可审计迁移。 |
| P2-05 | **Open** | Missing dependency repair | template/icon/locale/provider 缺失可控修复。 |
| P2-06 | **Open** | Layout simulation | 写入前模拟 capability/monitor/provider 结果。 |
| P2-07 | **Open** | Availability policy | BuildSet/project/trust/platform 动态裁剪。 |
| P2-08 | **Open** | Accessibility metadata | label/role/shortcut/live region 与 provider 同代。 |
| P2-09 | **Open** | Localization coverage | culture matrix 与 pseudo preview。 |
| P2-10 | **Open** | Support bundle | 导出 instance/route/provider/template/failure receipt。 |
| P2-11 | **Open** | Multi-window topology | window/viewport/document routing 正式策略。 |
| P2-12 | **Open** | Performance budget | catalog query/restore/snapshot p95/p99 预算。 |

## 11. 资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| G-01 | **Fail** | Available definition 不要求 typed provider。 |
| G-02 | **Fail** | 缺 provider 的 builtin definition 仍可 admission。 |
| G-03 | **Fail** | open receipt 不区分 spawn/present/failure。 |
| G-04 | **Fail** | content kind 仍按 raw ID match。 |
| G-05 | **Fail** | Placeholder 与 snapshot flag 可矛盾。 |
| G-06 | **Fail** | instance 无 provider/session/owner generation。 |
| G-07 | **Fail** | capability 无 BuildSet/session/policy digest。 |
| G-08 | **Partial** | template spec 和 16 个物理资源存在，未全链 compile/link。 |
| G-09 | **Fail** | locale/icon/a11y 无 catalog coverage gate。 |
| G-10 | **Fail** | 默认 layout 仍有第二份 ID 表。 |
| G-11 | **Fail** | restore capability/descriptor 失败不会形成 typed unavailable product。 |
| G-12 | **Partial** | 有局部 document/session close，无统一 lifecycle receipt。 |
| G-13 | **Partial** | template data source 在锁外 callback，通用 provider 尚不存在。 |
| G-14 | **Fail** | callback panic/timeout/FFI fault 不会 quarantine。 |
| G-15 | **Fail** | route payload 无 generation fence。 |
| G-16 | **Partial** | extension 有部分 template/capability 校验，无 provider/owner compile admission。 |
| G-17 | **Partial** | contribution ticket 存在，无 view/instance production revoke。 |
| G-18 | **Fail** | multi-instance 测试不覆盖 document/asset/session identity。 |
| G-19 | **Partial** | command/menu/Welcome 能到 open；不能证明真实 provider/first-present。 |
| G-20 | **Fail** | Scene/Game、Prefab、Material、Animation、Diagnostics alias 未收敛。 |
| G-21 | **Fail** | UI Asset/Asset Browser 并行 ID 无迁移去重。 |
| G-22 | **Fail** | duplicate 有 deterministic error；unknown/retired 无 catalog compile policy。 |
| G-23 | **Fail** | unavailable 尚未统一过滤 command palette/功能统计。 |
| G-24 | **Fail** | first-present failure 无投影回滚。 |
| G-25 | **Partial** | contribution generation/replacement 原子，但旧 instance/route 不受 fence。 |
| G-26 | **Fail** | descriptor/template snapshot 无统一 bounded parser/quarantine。 |
| G-27 | **Fail** | open/attach/close/restore 无可重放事务 receipt。 |
| G-28 | **Partial** | 资源物理存在和 metadata 测试存在，真实 ZUI compile/schema 未进入本轮证据。 |
| G-29 | **Fail** | provider callback 的线程、取消、deadline 无测试。 |
| G-30 | **Fail** | 多窗口/viewport/降级矩阵没有产品 E2E。 |
| G-31 | **Pass** | 参考对照仅抽取本地源码可观察结构，不外推闭源实现。 |
| G-32 | **Partial** | 372-file union 指纹已重算；本轮没有独立 reviewer。 |

## 12. 目标架构

必须建立一个不可变、可代际替换的 `CompiledViewCatalog`，而不是继续给 `ViewDescriptor` 增加零散字段：

```text
ViewContribution
  -> schema/resource/capability/owner validation
  -> CompiledViewDefinition
       identity + aliases + version
       provider lease + provider generation
       template/payload/route/localization/icon/a11y closure
       document/session/multi-instance policy
       availability + stable denial reason
  -> immutable CatalogGeneration
       command/menu/layout/Welcome/metrics projections
  -> OpenViewTransaction
       resolve -> spawn -> mount -> first-present -> commit
       failure -> compensate -> unavailable/diagnostic receipt
  -> ViewInstanceLease
       instance + provider + document + route generations
  -> quiesce -> drain -> dispose -> terminal receipt
```

关键不变量：

1. `Available` 意味着 provider、资源、capability、owner 和 schema 已全部 admission；不是 descriptor 已插入 HashMap。
2. `Opened` 意味着 body/session 已创建并挂载；`Presented` 意味着目标窗口至少完成一次可观察呈现。
3. 所有 command/menu/layout/Welcome projection 来自同一 catalog generation；禁止 raw ID side table 成为第二 authority。
4. restore 可以显示 Unavailable，但必须携 typed reason、旧 generation、repair/remove action，不能伪造正常 content。
5. reload/revoke 先 fence 新调用，再 quiesce/drain，最后销毁 instance/session/template/route；旧 generation 不能路由到新 session。

## 13. 重构顺序

### Phase A：先封住假 Available

1. 合并 core extension/workbench 两套 descriptor schema，定义 typed `ViewDefinitionId`、owner、version、alias、provider key、availability reason。
2. 把 `descriptor_content_kind()` binding 移入 compiled definition；对 19 个缺 provider 的条目 fail-close 或显式 Unavailable。
3. 修复 snapshot 的 Placeholder/availability 矛盾，过滤 command/menu/layout/metrics projection。

### Phase B：建立 provider-bound open transaction

1. 以 UI Asset、Animation、Diagnostics 的现有 session 为样板实现 `ViewContentProvider`。
2. 让 open 返回阶段化 receipt；attach/layout/native window/first-present 失败必须补偿 registry/session/window。
3. `ViewInstance` 加 qualified document/session/provider generation，close 统一 quiesce/drain/dispose。

### Phase C：收敛目录和默认布局

1. 从 catalog 生成 builtin shell、design stack、commands、menus、Welcome action 和 reflection 名称。
2. 选择 UI Asset/Asset Browser、Scene/Game、Prefab、Material、Animation、Diagnostics canonical ID，提供 versioned alias migration。
3. 删除 `editor.animation_timeline` 等无 owner alternate string 和手写 second authority。

### Phase D：Extension、resource 与故障域

1. 把 contribution ticket/generation 贯穿 view admission、instance lease、route 和 revoke。
2. catalog compile 真实解析 ZUI，验证 component/schema/payload/route/localization/icon/a11y。
3. provider/data-source callback 一律锁外调用，并加 panic、deadline、cancel、quarantine、bounded diagnostics。

### Phase E：产品资格矩阵

1. 对全部 40 项生成 provider/resource/owner/capability/locale/icon/a11y manifest。
2. E2E 覆盖 open -> spawn -> first-present -> interact -> close、restore、reload、revoke、capability downgrade 和 multi-window。
3. 最后再做 catalog query、large restore、snapshot clone 的 p95/p99 benchmark；局部 HashSet/buffer 优化不能替代架构完成度。

## 14. 复核结论

当前 Editor 视图系统的壳层已经足以承载真正的工程化重构，但目录仍把 metadata existence 当成产品 availability。最优先工作不是增加更多窗口，也不是给 Placeholder 补静态文案，而是把 provider、template/resource、capability、owner generation、session lifecycle 和 present receipt 编译成单一 definition，并让所有入口只投影这一权威。P0-01 在这条闭包建立前必须保持 Open。

本轮仅完成静态源码/资源/测试/本地参考审查与计划记录；未运行动态验证。实施前必须在独占或明确冻结的工作树重新计算选择集和 fingerprint，并重查 40 项资格矩阵。
