---
related_code:
  - zircon_editor/src/core/plugin/descriptor.rs
  - zircon_editor/src/core/plugin/sdk
  - zircon_editor/src/core/plugin/catalog_gen.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor.rs
  - zircon_runtime/src/plugin/core_profiles.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/manifest.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/Fyrox/editor/src/plugin.rs
  - dev/godot/editor/plugins/editor_plugin.h
plan_sources:
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_plugins/13-standalone-plugin-build.md
  - docs/plans/zircon_editor/editor_layout/08-plugin-page-interface-and-messaging.md
status: in_progress
---

# 12 编辑器插件管理

> 2026-08-01 实仓复核：`core/plugin/` 的 manager、loading phase、catalog snapshot、Faulted isolation 与 panel source，以及 serialized contribution DTO 的 AssetType/SettingsPage 物化已经落入当前源码；生命周期生产者、其余四类 cdylib 贡献、SDK builder、revoke 联动和受管 Cargo/规模证据仍未闭合。计划状态由 `planned` 修正为 `in_progress`。

## 参照证据（dev/）

**UE 加载阶段与插件类型**（`ModuleDescriptor.h:24-100`、`IPluginManager.h:18-64`）：

```cpp
// ELoadingPhase 十值（节选实测）：EarliestPossible, PostConfigInit, PostSplashScreen,
//   PreEarlyLoadingScreen, PreLoadingScreen, PreDefault, Default, PostDefault, PostEngineInit, None
// EHostType（节选实测）：Runtime, RuntimeNoCommandlet, RuntimeAndProgram, CookedOnly, UncookedOnly, ...
// EPluginType 五值：Engine, Enterprise, Project, External, Mod
// EPluginLoadedFrom 两值：Engine, Project
// IPlugin: GetName/GetFriendlyName/GetDescriptorFileName/GetBaseDir/GetContentDir/GetExtensionBaseDirs
```

要点：**加载阶段是描述符声明而非代码顺序**；插件来源（引擎级/工程级）是一等分类，决定发现路径与信任级。

**Fyrox 编辑器插件钩子全集**（`dev/Fyrox/editor/src/plugin.rs:46-132`）——13 个默认空实现钩子：

```rust
pub trait EditorPlugin {
    fn on_start(&mut self, _: &mut Editor) {}
    fn on_exit(&mut self, _: &mut Editor) {}
    fn on_sync_to_model(&mut self, _: &mut Editor) {}
    fn on_mode_changed(&mut self, _: &mut Editor) {}
    fn on_scene_changed(&mut self, _: &mut Editor) {}
    fn on_ui_message(&mut self, _: &mut UiMessage, _: &mut Editor) {}
    fn on_suspended(&mut self, _: &mut Editor) {}
    fn on_resumed(&mut self, _: &mut Editor) {}
    fn on_leave_preview_mode(&mut self, _: &mut Editor) {}
    fn on_update(&mut self, _: &mut Editor) {}
    fn on_post_update(&mut self, _: &mut Editor) {}
    fn on_message(&mut self, _: &Message, _: &mut Editor) {}
    fn is_in_preview_mode(&self) -> bool { false }
}
```

**godot 专型插件家族**（`editor_plugin.h`）：`EditorInspectorPlugin/EditorImportPlugin/EditorExportPlugin/EditorDebuggerPlugin` 各司一个扩展维度 + `_enable_plugin/_disable_plugin` 启停钩子——插件按扩展维度贡献而非泛型钩子堆叠，与 06 贡献族模型同构。

## 现状与证据（zircon）

**编辑器插件合同已有**（当前 owner 为 `core/plugin/{descriptor,registration,extension_materialization}.rs`；以下签名摘录保留原始审计上下文）：

```rust
// :15-21
pub struct EditorPluginDescriptor { package_id, display_name, crate_name, category, capabilities }
// :70-94 —— 贡献式而非 Fyrox 逐帧钩子式；四方法全带默认实现（描述符本身即最小插件，:96-100 blanket impl）
pub trait EditorPlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor;
    fn package_manifest(&self, runtime_manifest: PluginPackageManifest) -> PluginPackageManifest;
        // 默认=descriptor().attach_to_package(...)，标 RuntimeTargetMode::EditorHost
    fn editor_capabilities(&self) -> &[String];
    fn register_editor_extensions(&self, registry: &mut EditorExtensionRegistry)
        -> Result<(), EditorExtensionRegistryError>;                    // → 06 贡献批
    fn on_lifecycle_event(&self, event: &EditorPluginLifecycleEvent)
        -> Result<(), EditorPluginLifecycleError>;                      // 唯一生命周期入口
}
// :103-109 —— 注意 extensions 字段是整表批（06 批模型同构）
pub struct EditorPluginRegistrationReport {
    package_manifest, capabilities,
    extensions: EditorExtensionRegistry,     // 该插件的贡献批存档
    lifecycle: EditorPluginLifecycleReport, diagnostics: Vec<String>,
}
```

**生命周期词汇已定义 10 值**（`core/plugin/sdk/lifecycle.rs`，v2「场景/模式感知未定义」修正）：`EditorPluginLifecycleStage::{Loaded, Enabled, Disabled, Unloaded, HotReloaded, EnteredPlayMode, ExitedPlayMode, SceneChanged, AssetChanged, UiMessage}`——Fyrox `on_mode_changed/on_scene_changed` 的对应词汇**已在**，真缺口是**触发接线**：Entered/ExitedPlayMode 待 04 状态机、SceneChanged 待 01 文档消息、AssetChanged 待 09 索引事件逐一点火（本计划 M1 接线清单）。

`EditorPluginCatalog` + **build 期生成目录**：`core/plugin/catalog_gen.rs` 经 `include!(OUT_DIR/...)` 提供 `builtin_editor_plugin_descriptors() -> Vec<EditorPluginDescriptor>`（静态表 `GeneratedEditorPluginCatalogEntry`）——**内建插件发现是编译期生成的**。`core/plugin/sdk/{examples.rs,lifecycle.rs}` 为 SDK 雏形。

**runtime 侧契约成熟**：

- `RuntimePluginDescriptor` 18 字段（`plugin/runtime_plugin/descriptor.rs:18-36`）：`target_modes / capabilities / provided_interfaces / system_sets / system_anchors / capability_statuses / maturity / optional_features / default_packaging`——能力协商与打包策略字段齐。
- `EditorCoreProfile::minimal()` 六硬需求（`core_profiles.rs`）：`ui_shell / asset_core / scene_interaction / runtime_render_embed / plugin_management / capability_bridge`。
- ABI：`ZrPluginModuleDescriptorV1 { abi_version, kind, name, crate_name, target_modes, capabilities }`，`ZrPluginModuleKind::{Runtime=1, Editor=2, Native=3}`、`ZrRuntimeTargetMode::{ClientRuntime=1, ServerRuntime=2, EditorHost=3}`（`manifest.rs:5-42`）；`ZrHostApiV3` 五域指针（`plugin_api.rs:49-145`）：`ecs{register_system, register_component, spawn_command} / asset{request} / event{emit, drain} / bridge{call} / diagnostics{emit, metric}`。
- SDK：`RuntimePluginRegistrationBuilder` + `RuntimePluginModuleRegistration{ runtime_scene_system / event / plugin_option / plugin_event_catalog / export_interface<T> }`（`zircon_plugins/plugin_sdk/src/registration.rs:13+`）。

**当前缺口**：manager/loading phase、项目插件状态快照、Faulted isolation 与 panel source 已存在，不再重复实现。仍未闭合的是生命周期 10 值的真实生产者接线；cdylib 六类贡献中 View/Drawer/Menu/Command 四类的 materializer 与 SDK builder；revoke/diagnostics 的完整端到端；以及 source-bound 受管 Cargo、双轨 fixture 和 1/100/1000 规模证据。

## 目标

1. **`EditorPluginManager`**：发现三源——内建（`builtin_editor_plugin_descriptors()` 既有）/ 工程 `plugins/`（10 布局）/ zircon_plugins 产物清单（`PluginPackageManifest`）；按 10 `ProjectManifest.plugins`（`ProjectPluginManifest` 既有字段）过滤；加载阶段裁剪 UE 十值为三值 `LoadingPhase::{PreWorkbench, Default, PostWorkbench}`（编辑器无 splash/config 细分需求，留扩展位）；语义=贡献物化时机（PreWorkbench 可注册 WorkbenchSlot 结构性贡献，PostWorkbench 只能挂内容）。
2. **贡献通道双轨**：
   - 进程内（内建/rlib feature 链接）：`register_editor_extensions` 直投 06 `ContributionStore`（现机制保留，来源标 `ContributionSource::Plugin(id)`）。
   - **cdylib（`ZrPluginModuleKind::Editor`）**：序列化贡献描述 `SerializedContributionBatch`（view/drawer/menu/命令/资产类型/设置页，各为 id+schema 声明；DTO 入 `zircon_runtime_interface`）经 ABI 投递 → 宿主物化器转 `ContributionBatch`；回调经 `ZrHostApiV3.event{emit,drain}` + 01 `Custom{topic,payload}` 消息（该变体的合法生产者即本物化器）；自定义绘制面板走 `editor_layout/08` 页面协议。
   - SDK：`zircon_plugins/plugin_sdk` 增 `EditorContributionBuilder`（与 `RuntimePluginRegistrationBuilder` 同风格链式）。
3. **故障隔离**：注册与回调包 `catch_unwind` 边界；panic → 该插件贡献 revoke（06 ticket）+ 状态降级 Disabled(Faulted) + `diagnostics` 域上报；宿主不倒。
4. **管理面板**：数据源 `{ descriptor, source, phase, state, capabilities, load_duration, report: EditorPluginRegistrationReport, failure }`；启停写 `ProjectPluginManifest`（重启生效为底线，PostWorkbench 阶段插件支持热启停=contribute/revoke）；插件设置页=贡献一种（17 settings 框架承载）。
5. **版本与依赖纪律**：加载前校验 `abi_version`、`engine_version_req`（`PluginPackageManifest`）、能力需求 vs `EditorCoreProfile`/`RuntimeCapabilities`；插件间声明依赖仅做循环检测拒载（完整解析器不做）。

## 非目标

- 插件市场/在线分发；`ZrHostApiV3` 扩域（需求走 runtime 插件计划提案）；运行时（非编辑器）插件加载路径改动（`RuntimePluginCatalog` 域）；cdylib 热卸载（见风险）。

## 架构设计

### 模块布局

```
zircon_editor/src/core/plugin/
  mod.rs
  manager.rs           # EditorPluginManager：三源发现/阶段/启停/状态机
  phases.rs            # LoadingPhase 三值
  materializer.rs      # SerializedContributionBatch → ContributionBatch 物化
  isolation.rs         # catch_unwind 边界 + Faulted 降级
  panel_source.rs      # 管理面板数据源
# retired root paths were hard-cut; descriptor, SDK, and catalog generation are folder-owned
zircon_runtime_interface/src/editor_contribution.rs # SerializedContributionBatch DTO（已落地单文件）
zircon_plugins/plugin_sdk/src/editor_contribution.rs # EditorContributionBuilder
```

### 插件状态机

```
Discovered → (enable表过滤) Disabled | Validated → (phase 到达) Loading
  → Active | Faulted(报告)
Active → (用户停用) Revoking → Disabled     # PostWorkbench 插件免重启
Faulted → (用户重试/升级) Validated
```

状态迁移事件入 bus；`EditorPluginRegistrationReport`（既有五字段）为 Loading→Active 的产物存档。

### 双轨等价契约

同一夹具插件（`native_dynamic_fixture` 惯例延伸：`editor_contribution_fixture`）编译两态——rlib 直注册与 cdylib 序列化投递——对 `ContributionStore` 的最终物化内容做**逐字段等价断言**：双轨语义一致性的机器化保证。

### 深度测试

新增一种可序列化贡献类型（如设置页）= DTO 变体 + 物化器一个 match 臂 + SDK builder 一方法；manager/isolation/状态机零改动。

## 里程碑

### M1 管理器与启停

- 切片 1.1：`core/plugin/` 目录化迁移（editor_plugin.rs 拆入，catalog_gen 保留 build 机制）；`EditorPluginManager` 三源发现 + 状态机 + `LoadingPhase`；现 catalog 一次性加载改为按阶段（内建插件默认 Default）。
- 切片 1.2：`ProjectPluginManifest`（10 manifest 既有字段）启停消费 + abi/engine 版本与能力校验 + 循环依赖拒载；生命周期 10 值点火接线清单（Loaded/Enabled/Disabled 由 manager 直发；EnteredPlayMode/ExitedPlayMode 订阅 04 `ModeMessage`；SceneChanged 订阅 01 `DocumentMessage`；AssetChanged 订阅 09 索引事件——各依赖计划未落地者记接线债）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（catalog 既有测试迁移后须过 + 状态机全迁移矩阵 + 阶段顺序断言 + 校验失败形状）。更新 `docs/zircon_editor/core/plugin.md`。

### M2 cdylib 贡献通道与隔离

- 切片 2.1：`SerializedContributionBatch` DTO（首批六类：view/drawer/menu/command/asset_type/settings_page）+ 物化器 + SDK builder；当前 DTO 六类已定义，materializer 仅完成 asset_type/settings_page，view/drawer/menu/command 四类与 `editor_contribution_fixture` 端到端仍是独立待办。
- 切片 2.2：`isolation.rs`：注册/回调 catch_unwind + Faulted 降级 + revoke 联动（06 ticket）+ diagnostics 上报。
- 测试阶段：`cargo test -p zircon_runtime_interface --locked`（DTO 往返）+ `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked` + `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked`；验收：双轨等价断言、夹具插件 panic 注入→宿主存活且贡献回收（隔离测试）。

### M3 管理面板与热启停

- 切片 3.1：`panel_source.rs` + 启停写回 `ProjectPluginManifest` + 设置页贡献；PostWorkbench 插件热启停（contribute/revoke 往返）。
- 测试阶段：启停→(重启 | 热切)→生效矩阵；面板数据源与 manager 状态一致性；证据记状态节。

## 风险与开放问题

- **cdylib 热卸载不做**：热停用=revoke 贡献 + 停投事件，**库本体不卸**（悬挂函数指针风险；UE 同样不卸编辑器模块）。真卸载留待「零活跃回调证明」机制，立场写入模块文档。
- `Custom{topic,payload}` 消息的 topic 命名空间须以插件 id 前缀强制（物化器注入），防插件互踩——与 06 命名空间规则同处定稿。
- 序列化贡献表达力上限：完全自定义绘制走 `editor_layout/08` 页面协议；仍不足者要求插件提供进程内伴生 rlib（文档明示此阶梯）。

## 产出记录与时间

请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前状态：core plugin catalog、loading phase、Faulted isolation、项目插件快照、panel source 与 AssetType/SettingsPage 物化已有实现；生命周期生产者、其余四类贡献、SDK builder、revoke 联动和 source-bound 受管验证仍未闭合，父计划保持 `in_progress`。

- 具体记录已迁入：[性能评审交接归档](12/2026-08-01-performance-review-handoffs.md)
- fixed 已修复：[plugin-extension-validation-regressions](08/fixed-2026-07-15-plugin-extension-validation-regressions.md)
- fixed 已修复：[native-plugin-runtime-target-mode-test-path](09/fixed-2026-07-13-native-plugin-runtime-target-mode-test-path.md)
- fixed 已修复：[plan-output-audit-counts-lifecycle-links](12/fixed-2026-07-15-plan-output-audit-counts-lifecycle-links.md)
- open 待修复：[Editor04 PlaySessionController typed mode message producer](04/failure-2026-07-29-play-mode-message-producer-missing.md)
- open 待修复：[Editor01 document authority typed message producer](01/failure-2026-07-29-document-message-producer-missing.md)
- open 待修复：[editor plugin catalog rebuild and deep copy](12/failure-2026-07-17-editor-plugin-catalog-rebuild-and-deep-copy.md) · [plugin list canonical catalog projection owner boundary](12/failure-2026-07-27-plugin-list-canonical-catalog-projection-owner-boundary.md) · [plugin UI template V2 runtime host wiring](12/failure-2026-07-27-plugin-ui-template-v2-runtime-host-wiring.md) · [template V2 pane dynamic control state projection](12/failure-2026-07-27-template-v2-pane-dynamic-control-state-projection.md) · [plugin manager inspector customization guard drift](12/failure-2026-08-01-plugin-manager-inspector-customization-guard-drift.md)

## Code Review 处理结果 (2026-08-01)

### 已处理

- front matter 已提升为 `in_progress`；缺口段已从“全部缺失”改为当前剩余接线与验收边界。
- serialized contribution DTO 的落点已校准为单文件 `zircon_runtime_interface/src/editor_contribution.rs`。

### 仍开放

- M2 仍需补齐 View/Drawer/Menu/Command 四类 materializer、SDK builder、cdylib fixture 双轨等价与 panic/revoke 产品链；AssetType/SettingsPage 的存在不得作为六类完成证据。
