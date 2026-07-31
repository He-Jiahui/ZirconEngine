---
related_code:
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_plugin_sdk
  - zircon_editor/src/core/editor_plugin_catalog_gen.rs
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
status: planned
---

# 12 编辑器插件管理

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

**编辑器插件三件套已有**（`core/editor_plugin.rs`，2026-07-05 全签名实读）：

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

**生命周期词汇已定义 10 值**（`editor_plugin_sdk/lifecycle.rs:4-14`，v2「场景/模式感知未定义」修正）：`EditorPluginLifecycleStage::{Loaded, Enabled, Disabled, Unloaded, HotReloaded, EnteredPlayMode, ExitedPlayMode, SceneChanged, AssetChanged, UiMessage}`——Fyrox `on_mode_changed/on_scene_changed` 的对应词汇**已在**，真缺口是**触发接线**：Entered/ExitedPlayMode 待 04 状态机、SceneChanged 待 01 文档消息、AssetChanged 待 09 索引事件逐一点火（本计划 M1 接线清单）。

`EditorPluginCatalog`（:187+）+ **build 期生成目录**：`editor_plugin_catalog_gen.rs` 经 `include!(OUT_DIR/...)` 提供 `builtin_editor_plugin_descriptors() -> Vec<EditorPluginDescriptor>`（静态表 `GeneratedEditorPluginCatalogEntry`）——**内建插件发现是编译期生成的**。`editor_plugin_sdk/{examples.rs, lifecycle.rs}` 为 SDK 雏形。

**runtime 侧契约成熟**：

- `RuntimePluginDescriptor` 18 字段（`plugin/runtime_plugin/descriptor.rs:18-36`）：`target_modes / capabilities / provided_interfaces / system_sets / system_anchors / capability_statuses / maturity / optional_features / default_packaging`——能力协商与打包策略字段齐。
- `EditorCoreProfile::minimal()` 六硬需求（`core_profiles.rs`）：`ui_shell / asset_core / scene_interaction / runtime_render_embed / plugin_management / capability_bridge`。
- ABI：`ZrPluginModuleDescriptorV1 { abi_version, kind, name, crate_name, target_modes, capabilities }`，`ZrPluginModuleKind::{Runtime=1, Editor=2, Native=3}`、`ZrRuntimeTargetMode::{ClientRuntime=1, ServerRuntime=2, EditorHost=3}`（`manifest.rs:5-42`）；`ZrHostApiV3` 五域指针（`plugin_api.rs:49-145`）：`ecs{register_system, register_component, spawn_command} / asset{request} / event{emit, drain} / bridge{call} / diagnostics{emit, metric}`。
- SDK：`RuntimePluginRegistrationBuilder` + `RuntimePluginModuleRegistration{ runtime_scene_system / event / plugin_option / plugin_event_catalog / export_interface<T> }`（`zircon_plugins/plugin_sdk/src/registration.rs:13+`）。

**缺口**：无加载阶段（catalog 一次性全量）；无用户启停（10 `ProjectManifest.plugins: ProjectPluginManifest` 字段已在，编辑器侧无消费者）；`ZrPluginModuleKind::Editor=2` 的 **cdylib 编辑器贡献通道未打通**（Editor 类插件经 ABI 能注册什么、怎么物化 UI——空白）；无故障隔离（`register_editor_extensions` panic 直接炸宿主）；无插件管理面板；生命周期 10 值词汇**无点火接线**（谁在何时调 `on_lifecycle_event` 需逐事件落实；细粒度感知由 01 bus 订阅补足，不复制 Fyrox 逐钩子形态）。

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
# editor_plugin.rs / editor_plugin_sdk / catalog_gen 迁入本目录，根文件薄化
zircon_runtime_interface/src/editor_contribution/   # SerializedContributionBatch DTO
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

- 切片 2.1：`SerializedContributionBatch` DTO（首批六类：view/drawer/menu/command/asset_type/settings_page）+ 物化器 + SDK builder；`editor_contribution_fixture` 夹具插件（cdylib）打通端到端。
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

- fixed 已修复：[plugin-extension-validation-regressions](08/fixed-2026-07-15-plugin-extension-validation-regressions.md)
- fixed 已修复：[native-plugin-runtime-target-mode-test-path](09/fixed-2026-07-13-native-plugin-runtime-target-mode-test-path.md)
- fixed 已修复：[plan-output-audit-counts-lifecycle-links](12/fixed-2026-07-15-plan-output-audit-counts-lifecycle-links.md)
- open 待修复：[Editor04 PlaySessionController typed mode message producer](04/failure-2026-07-29-play-mode-message-producer-missing.md)
- open 待修复：[Editor01 document authority typed message producer](01/failure-2026-07-29-document-message-producer-missing.md)

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1/M2 | 插件贡献注册、capability gate 与失败原子性 | `未通过-已归档到本功能计划` | 2026-07-12 | Editor03/08 统一受管 job `520d85713df249afae31661a7697ad07` 复现无效 operation schema、plugin registration report capability gate、duplicate extension view 三项失败；同一 binary 的 Editor08 command registry hard-cut 用例通过，因此本组由 Editor12 贡献物化/校验 owner 处理。精确用例与验收见本计划顶层 `plugin-extension-validation-regressions` Failure 链接，禁止恢复旧 operation registry。 |
| M1 | core plugin catalog owner、snapshot read model、加载阶段与状态机边界 | `进行中-静态架构已完成` | 2026-07-27 | `core/plugin/manager.rs` 单一持有 catalog store 与 manager snapshot，stable UI/commandlet 读取共享 `Arc` catalog/projection；snapshot 发布期物化 extension report，避免首次只读查询构建 extension catalog。`EditorPluginLoadingPhase` 已硬迁移至 `core/plugin/phases.rs`，`manager.rs` 无第二定义，`core::plugin` 仅保留唯一 public re-export。状态机定义 `Discovered → Validated → Loading → Active → Revoking → Disabled` 与 `Faulted → Validated` 合法边；`set_enabled` 在单次 publish 内沿合法边计算终态，不能跳过 revoke 或从 faulted 直接 active。`test_editor12_plugin_catalog_store_contract.py` 9/9、`test_editor12_plugin_manager_contract.py` 6/6、相关 `rustfmt --check` 与 scoped `git diff --check` 通过。受管 Cargo job `f8726b18912e49d7a74dcb10051f3006` 在 coordinator materialization prepare 阶段失败且未启动 Cargo；M1 仍待 source-bound 重跑、生命周期点火、三源 discovery 与独立复审。 |
| M2 | plugin registration/lifecycle panic isolation 与 Faulted 降级 | `进行中-核心边界已完成` | 2026-07-27 | 新增 `core/plugin/isolation.rs`，将 extension registration、runtime event consumer discovery 和 lifecycle callback 的 `Err`/panic 统一转为带 package/operation 的诊断。extension 先写 candidate registry，只有完整成功才 publish，避免部分贡献泄漏；snapshot 建立 faulted package index，manager publish 将对应 entry 置为 `Faulted`。新增 Rust 单元测试覆盖 error/panic 诊断与 Faulted publication；静态 `test_editor12_plugin_manager_contract.py` 6/6、catalog contract 9/9、相关 `rustfmt --check` 与 scoped `git diff --check` 通过。Rust 测试未运行，仍待 coordinator 修复后 source-bound Cargo、cdylib DTO/revoke 和独立复审。 |

- 2026-07-27 M1 三源 discovery 补充：`EditorPluginDiscovery` 明确承载 package id、`Builtin/Project/PackageManifest` source 与 loading phase；`new_with_discoveries`、`publish_catalog_with_discoveries` 在构建/发布前拒绝重复或未知 package，成功后将 source/phase 与 catalog snapshot 同代发布。无新 discovery 的 publish 保留既有 entry metadata，不建立 UI 私有副本或触发同步发现 I/O。新增 Rust 初始/发布 metadata 回归；`test_editor12_plugin_manager_contract.py` 6/6、catalog contract 9/9、全量 Editor12 changed plugin Rust `rustfmt --check` 通过。受管 Cargo 仍未运行，M1 保持进行中。
- 2026-07-27 M2 serialized contribution DTO：新增 `zircon_runtime_interface::editor_contribution`，以严格 tagged/deny-unknown-fields 格式定义 view、drawer、menu、command、asset type、settings page 六类贡献，`SerializedContributionBatch` 对 `(kind,id)` 确定排序并拒绝重复项。静态契约 `test_editor12_serialized_contribution_contract.py` 1/1 与 DTO 文件 `rustfmt --check` 通过；尚未接入 editor materializer、SDK builder 或受管 Cargo，M2 保持进行中。
- 2026-07-27 M2 asset type materialization：`SerializedEditorContribution::AssetType` 已补齐 badge、icon、color token 与 thumbnail icon，并由 `materialize_serialized_contribution_batch` 转换为现有 `AssetTypeContribution::define` 后进入候选 extension registry；非法 asset id 或 registry 冲突会使整批失败且不提交。DTO/materializer 静态契约各 1/1、局部 `rustfmt` 通过；settings page registry、SDK builder 与 Cargo 验证仍待完成。
- 2026-07-27 M2 settings page materialization：新增 `SettingsPageDescriptor`（id、display name、category path），由 `EditorExtensionRegistry` 按 id 唯一保存并以借用 row 读取；DTO 的 `SettingsPage` 现在携带显式 category path，materializer 将其加入同一候选 registry，非法 id/category 或任意同批错误不提交。serialized/materializer/settings-page 三个静态契约各 1/1、局部 `rustfmt` 通过；SDK builder 与受管 Cargo 验证仍待完成。
- 2026-07-22 plugin availability投影补充：PERF-MVP-530/531已删除底层bridge status/summary与availability descriptor无谓owned row；Editor12按534只消费Plugins01 catalog+profile+provider+manifest compact generation，列表分页借用category/index/summary，full reason/diagnostics仅selected/export懒取。stable 60Hz projection/reason clone/category scan=0，reload只刷新changed generation。
- 2026-07-22 catalog/project plan补充：Editor12的发现、启停与热重载按PERF-MVP-537一次提交完整mutation batch，不逐项调用`register`触发全投影；面板/状态机按538只持generation/fingerprint与borrowed分页视图，复用Plugins01/Runtime06发布的`CompiledProjectPluginPlan`。stable轮询不得调用`builtin().clone()`、`package_manifests()`深clone或重建runtime extension registry；toggle显示last-good并消费后台candidate进度/失败诊断。
- 2026-07-22 native discovery/load-report补充：Editor12只读状态先比较discovery/load generation，stable轮询不得调用owned `discover()`、`NativePluginLoadReport` convenience getters或加载DLL/执行entry；changed generation只消费一次共享projection与后台candidate结果。PERF-MVP-539后仍需验证1k/10k manifests stable report/projection build、candidate/manifest clone和root lock hold均为0，并显示Runtime11 queue/progress/last-good诊断。
- 2026-07-22 editor extension catalog复核：descriptor→runtime manifest匹配已由O(P×M)改为一次first-wins borrowed index；operation/menu path/unique insert也删除临时Vec、无效String clone和双查表。`EditorPluginCatalog`仍同时保留per-plugin registry与deep-cloned merged registry，mutation后全量merge；PERF-MVP-538必须让Editor12与Runtime06共享同一frozen extension generation/ordered handles，面板不得拥有第二份merged truth。
- 2026-07-23 capability hot-consumer补充：Editor manager `capability_snapshot()`当前深clone enabled/disabled/diagnostics三组Vec，runtime-event tick随后又clone enabled Vec并每帧全量reconcile subscriptions。Editor12按PERF-MVP-565把既有534/538 compact capability结果发布为单一`Arc` generation+dense membership，stable getter只clone Arc；Editor02只在generation变化时消费affected delta。不得用每tick全Vec hash/compare或为runtime consumer建立第二份capability cache。
- 2026-07-30 entry startup补充：`zircon_app/entry_runner/**` current 13/13确认产品GUI/CLI已用单个prepared pipeline消除first-party registration重复构造与project按路径二次open；剩余`EntryConfig`对project plugin manifest一次深clone，公开composition深clone整组runtime report，且entry native selection与host manifest apply各调用一次`load_discovered_editor`。Editor12不建立第二启动缓存，须把534/538的唯一native load report/plugin/extension generation以共享handle交给Editor01现有`EditorStartupPreparation`，让manifest、registrations、capability/catalog/module descriptors只封存一次且reload只失效changed generation。GUI/CLI/composition × 0/1/100/1,000 plugins要求native discovery/load/entry/build≤1/generation、deep-clone bytes=0；回链PERF-MVP-427与Editor01既有`editor-startup-single-projection` failure。
- 2026-07-30 M1 项目插件状态快照与稳定面板投影：状态 `resolving_failure`，关联 failure `12/failure-2026-07-17-editor-plugin-catalog-rebuild-and-deep-copy.md` 保持 `open`。`EditorManager` 在项目打开的单次 native load report 上发布 `Arc<ProjectPluginStatusSnapshot>`，关闭和打开回滚清除它；retained Module/Plugin pane 只读该快照，并以 `Arc::ptr_eq` 缓存派生 view data，禁止重新读取 manifest、discovery 或 DLL load。native-aware enablement、feature、packaging 与 target-mode 成功 mutation 均发布 successor snapshot，失败保留 last-good。静态证据为 `test_editor12_plugin_manager_contract.py` 24/24、全量 19 个 `test_editor12_*.py`、精确 `rustfmt --check` 与 scoped `git diff --check` 通过；Rust Cargo、独立复审、1/100/1000 scale evidence 和 `fixed-*` 回传仍待 Coordinator01 的 canonical immutable validation-copy 故障修复后执行。
- 2026-07-30 Performance01 `editor_extension`补充：PERF-MVP-538除catalog snapshot外还必须让stable contribution query消费单一capability membership与direct indexes，禁止按registration重建`BTreeSet<String>`/贡献Vec或为chrome深clone完整registry；registration/reload对command、asset type、template candidate各只build/publish一次。PERF-MVP-595另由Editor12定义generation-owned pane data source与`NotModified`/affinity/estimated-byte/last-good/unload contract；foreign `snapshot()`不得在shell/registry锁内执行，且隐藏/稳定source不得被调用。证据与规模矩阵见`../../performance/01/2026-07-30-editor-core-editor-extension-current-review.md`；current-source Cargo/F4未过，保持pending。
- 2026-07-30 Performance01根契约补充：`editor_authoring_extension.rs`的track-type与capability builder允许链式append，但每次都对完整增长Vec执行sort/dedup。当前调用位于插件构建/测试而非stable tick；Editor12不单建热点任务，而把它并入PERF-MVP-538/079：contribution batch先收集unsorted owned input，structural candidate只normalize/validate/index一次并原子发布frozen generation，稳定查询不得再次normalize。按0/1/100/10K descriptors与1/100 batches记录sort、String move/allocation、candidate build/publish，重复诊断与确定顺序必须等价。证据见`../../performance/01/2026-07-30-editor-core-root-contracts-current-review.md`；managed Cargo与规模counter未过，保持pending。

## Code Review 建议 (2026-07-31)

### 与代码现状不符，需修订

- front-matter 之外的 `status: planned` 与实现现状严重脱节。`zircon_editor/src/core/plugin/` 已是完整目录：`manager.rs` + `manager/`、`phases.rs`、`materializer.rs`、`isolation.rs`、`panel_source.rs`、`catalog_store.rs`、`catalog_snapshot.rs`、`extension_materialization.rs`、`lifecycle_message_bridge.rs`、`admission.rs`、`sdk/` 均在案，架构设计「模块布局」列出的六个核心文件全部落地。计划正文「缺口」节仍以「无加载阶段/无用户启停/cdylib 通道未打通/无故障隔离/无管理面板」描述现状，与产出记录表（M1/M2 静态架构已完成、`isolation.rs`/`panel_source.rs` 已落地）自相矛盾。建议把「现状与证据」的缺口清单据实收窄，或将 status 提升为 `in_progress` 并在缺口节标注「已落地/待接线」两栏。
- 目标 2「cdylib 贡献通道」的 DTO 归属写作 `zircon_runtime_interface/src/editor_contribution/`（目录），实际落地为单文件 `zircon_runtime_interface/src/editor_contribution.rs`。该文件 `:12 SerializedEditorContribution` 已含 `AssetType{..}`（:34）与 `SettingsPage{..}`（:43）变体、`:110 SerializedContributionBatch` 与 `:165 SerializedContributionBatchError`，与产出记录 2026-07-27 条目一致。模块布局图应同步为单文件形态。

### 设计优化建议

- 产出记录显示 materializer 目前只实装了 `AssetType` 与 `SettingsPage` 两类贡献（`materializer.rs`），而目标 2 与 M2 切片 2.1 承诺「首批六类：view/drawer/menu/command/asset_type/settings_page」。建议在 M2 里程碑内显式列出剩余四类（view/drawer/menu/command）的物化器 match 臂为独立待办项，避免「六类」在验收时被两类冒充完成——「深度测试」节的「新增一种可序列化贡献类型 = DTO 变体 + 一个 match 臂」正好可作为补齐这四类的验收口径。
