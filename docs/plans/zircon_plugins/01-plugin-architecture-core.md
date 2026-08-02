# 01 · 插件架构核心升级计划（Runtime Plugin Interface v2）

> 状态：工程化细化版 v2 · 优先级：P0（所有后续插件计划的前置）
> 关联计划：`.codex/plans/Runtime_Editor 插件注册与 EditorOperation 设计计划.md`、`.codex/plans/多插件组合可选功能规则设计.md`、`.codex/plans/ZirconEngine 周边设施与插件能力完善计划.md`
> 本文档为 02–10 各插件计划的上游：§3.1 调度锚点表、§3.2 注册 API、§3.4 生命周期 trait、§3.7 ABI v3 中的命名为**定稿名**，下游文档逐字引用。

> **M3 当前契约说明（2026-08-02）**：紧随其后的“最新进度（2026-06-13）”仅保留验证追溯；其中 `RuntimePluginFeature` 的旧 lifecycle hooks、`CapabilityView` lifecycle-hook、`RuntimePluginCatalog::from_lifecycle_plugins` 和 plugin/feature `finish` 链均已失效，不能作为实现依据。`RuntimePluginFeature` 继续仅承担 optional feature 的 manifest/registration。当前定稿以 §3.4 和 M3 表为准：`RuntimePlugin` 持有 descriptor、`ModuleLifecycle` 与 registration，`CoreRuntime` 按 `build → ready → finish → cleanup` 驱动模块；唯一当前 guard 是 `runtime_plugin_lifecycle_hard_cuts_to_kernel_module_lifecycle`。

> 当前恢复状态（2026-08-02）：`resolving_failure`。M1–M4 current-source 静态门均为 clean，M5 的 ABI 布局、V4 behavior dispatch 与热重载回滚契约均已复核；554 个 failure artifact、计划输出记录、38 个 manifest 与 40 个 distribution target 均通过静态审计。现有 validation-copy `5945e3ef29d74bd69602adca02e243b5` 仍属于其他 session，且 Cargo FIFO reservation 仍被占用；17 个 Plugins01 failure 保持 open，未声明 dynamic Cargo 或最终验收 GREEN。2026-06/07 的 warmed 二进制记录仅保留诊断追溯，不能覆盖当前源码恢复验收；旧 plugin/feature `finish` 生命周期叙述已由 Frameworks02 的 `ModuleLifecycle` 硬切替代。

> 最新进度（2026-06-13）：01-M3-T1/T2/T3 已推进。`RuntimePlugin` / `RuntimePluginFeature` 只保留 `register(...)`，注册报告直接调用新入口，第一方 `zircon_plugins/*/runtime` 与相关测试实现已同步改名；`runtime_plugin_lifecycle_hard_cuts_to_register_hook` 守护旧符号不得回流。01-M3-T2 新增 `CapabilityView::from_registration_reports(...)`，从已收集的 plugin / feature 注册报告聚合主包 capability、runtime module capability、feature capability 与 `capability_statuses`，并明确不会把仅声明但尚未注册的 `optional_features` 提前暴露给 `finish`。01-M3-T3 新增 `RuntimePluginCatalog::from_lifecycle_plugins(...)`，在 native 生命周期构造路径中固定 `plugin.register -> feature.register -> plugin.finish -> feature.finish`，并把 plugin / feature finish 阶段注册项分别写回对应 report registry。01-M4-T3 新增 `declared_system_anchors_are_registered` 静态契约守护，把 plugin.toml runtime module 的 `system_anchors` 与对应 crate 源码中的 runtime system 注册路径绑定。`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-capability-view-coremin-0613 --message-format short --color never` 已通过（仅既有 warning 噪声）；`cargo test -p zircon_runtime --lib runtime_plugin_lifecycle --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-capability-view-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` 已通过 3 项生命周期 focused 测试。`declared_system_anchors_are_registered` 已在 warmed `zircon_runtime` lib-test 二进制中直接执行通过 1 项测试，覆盖 animation/physics manifest anchor 与 runtime-system 注册源绑定。`zircon_plugins` workspace `--locked` 检查仍因 `zircon_plugins/Cargo.lock` 需要更新而未进入编译，本次未修改锁文件。无关图形导入阻断已通过 `build_mesh_draws/build.rs` 的绝对 lighting 路径最小修正解除，以便共享 `zircon_runtime` lib 编译继续进行。
> 01-M5-T1 已新增 `abi_v3_layout_is_stable` 聚合守护，直接绑定本计划要求的 ZrHostApiV3 域表尺寸、offset、pointer-dense 子表、snapshot API 与 buffer ref ABI 布局。`cargo test -p zircon_runtime_interface --lib abi_v3_layout_is_stable --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-interface-abi-v3-0613 --message-format short --color never -- --test-threads=1 --nocapture` 已通过 1 项 focused 测试；`cargo test -p zircon_runtime_interface --lib plugin_api_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-interface-contracts-0613b --message-format short --color never -- --test-threads=1 --nocapture` 已在新增别名后通过 6 项全量契约测试。
> 01-M5-T2 已把 `NativeHostApiV3RegistrationScope` 的 `register_system` 映射到 `NativeDynamicAccess`，通过 `SystemParamAccess::add_conservative_world_access()` 让 ABI native 系统在调度冲突图中以全世界写入节点参与排序。`native_system_enters_schedule_as_conservative_node` 已写入 `host_api_adapter.rs`；`rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs` 已通过。`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never` 已通过（70 个既有 warning）。旧 target-dir 曾出现 dep-info/fingerprint 缺失、lib-test 编译超时、进程 `-1` 与渲染会话 lib-test 编译错误等非断言阻断；19:40 复用 `D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b` 重跑 `cargo test -p zircon_runtime --lib native_system_enters_schedule_as_conservative_node --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never -- --test-threads=1 --nocapture` 已通过 1 项 focused 测试。
> 01-M5-T3 已补齐计划命名测试别名：`hot_reload_failure_rolls_back_to_snapshot` 覆盖替换插件恢复失败后用旧快照恢复旧句柄，`failed_registration_revoked_via_ownership` 绑定 owner-tracked 撤销路径。`rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs zircon_runtime/src/tests/plugin_extensions/extension_registry_metadata.rs` 已通过。`cargo test -p zircon_runtime --lib hot_reload_failure_rolls_back_to_snapshot --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never -- --test-threads=1 --nocapture` 曾以进程 `-1` 停止且日志只有 warning、无 Rust error；随后直接执行 warmed `zircon_runtime` lib-test 二进制，`hot_reload_failure_rolls_back_to_snapshot` 与 `failed_registration_revoked_via_ownership` 均已各通过 1 项测试。
> 01-M5-T4 已核对 `zircon_plugins/native_dynamic_fixture` 默认 ABI v3 行为夹具：真实动态库测试断言 descriptor ABI=3、runtime/editor entry 名为 v3、runtime/editor 行为健康为 `NativePluginBehaviorHealth::Clean`、editor 诊断来自 v3 host ABI table，并通过 `abi_v2_only` 覆盖无 v3 descriptor 时回退到 v2。`native_loader_calls_real_fixture_descriptor_and_entries` 已补行为健康断言；测试辅助构建现在通过临时隔离 `Cargo.toml` 离线编译真实 fixture 源文件，避免 runtime tests 修改 `zircon_plugins/Cargo.lock`。`rustfmt --edition 2021 --check zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs` 已通过；`cargo test -p zircon_runtime --lib native_loader_calls_real_fixture_descriptor_and_entries --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never -- --test-threads=1 --nocapture` 已通过 1 项 focused 测试，warmed lib-test 二进制直接执行 `native_loader_falls_back_to_v2_when_v3_descriptor_is_absent` 也已通过 1 项测试。`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-fixture-0613 --message-format short --color never` 仍因 `zircon_plugins/Cargo.lock` 需要更新而在编译前停止，本次不修改锁文件。
> Descriptor 投影/注册报告校验已补焦点证据：直接执行 warmed `zircon_runtime` lib-test 二进制通过 `plugin_extensions::runtime_plugin_descriptor` 过滤组 9 项测试，并通过 `runtime_plugin_descriptor_projects_maturity_and_statuses_to_manifest` 1 项 maturity/status 投影测试；此项从会话开放验证清单移除。
> 01-M2 typed event 派生规则已修正并验证：`register_event(...)` 现在从 runtime module owner 派生 `<package>.events` catalog namespace，并复用 `PluginEventCatalogManifest` 校验，旧式 `weather.changed` 会在注册阶段拒绝而不是延迟到 catalog merge。`cargo test -p zircon_runtime --lib typed_event_registration --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-next-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` 已通过 2 项低层回归；warmed lib-test 二进制直接执行 `plugin_extensions::extension_registry_event_catalogs` 12 项、`plugin_resource_event_and_system_registrations_apply_to_world`、`runtime_plugin_registration_collects_package_manifest_declared_runtime_contributions`、`runtime_plugin_catalog_merges_module_and_render_feature_contributions` 均已通过，catalog merge 开放验证项移除。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-plugins-architecture-core",
  "goal": "完成类型化 ECS 插件扩展、生命周期与 Editor/Native 对称边界，并以冻结读取和硬切迁移保证运行时正确性。",
  "milestones": [
    {"id": "M1", "title": "ECS 调度图深度集成", "depends_on": []},
    {"id": "M2", "title": "类型化扩展点与冻结读取边界", "depends_on": []},
    {"id": "M3", "title": "插件生命周期 v2", "depends_on": ["M2"]},
    {"id": "M4", "title": "Editor 对称注册与诊断", "depends_on": ["M3"]},
    {"id": "M5", "title": "Native ABI v3 与热重载快照", "depends_on": ["M4"]}
  ]
}
```

<!-- Workflow topology is maintained independently from milestone output records. M2 is an independently committable late-adoption slice because its implementation predates coordinator workflow evidence; the task-level dependencies in the milestone tables remain authoritative. -->

## 1. 目标

把 `zircon_runtime` 的插件接口从"描述符 + Module 间接包装"升级为一套**完备、类型化、ECS 深度集成、零运行期开销**的插件架构，使 Sound / Physics / Animation / Navigation / AI / Net / VM 等周边设施插件可以：

1. 直接向 ECS 注册组件、资源、事件、系统与调度依赖，而不必通过 scene hook 手动包装 World 访问。
2. 以 Runtime / Editor 两个部分分别注册到运行时与编辑器，能力协商对称。
3. 在注册期完成全部验证与图编译，运行期只走 dense-id 直查与已编排好的执行计划（性能目标：插件接入对帧循环的额外固定开销为零）。
4. 保持现有硬切原则：不引入兼容层、不保留旧路径 re-export。

## 2. 现状基线（按代码实查校准的缺口表）

现有四层结构保留：`package_manifest` / `runtime_plugin` / `extension_registry` / `native_plugin_loader`（均位于 `zircon_runtime/src/plugin/`）。下表为逐文件核实后的缺口（注意：访问集与冲突图基建**已存在**，比早期规划假设的更完整，v2 设计直接复用而非重建）：

| # | 缺口 | 实查现状 |
|---|------|---------|
| G1 | 系统排序只有 `(stage, order: i32)`，无 SystemSet / before-after 偏序；且 `Schedule::register_native_system` 是 World 内部 API，`RuntimeExtensionRegistry` 没有 system 注册通道，插件只能走 `register_scene_hook`（每帧 dyn 调用、无访问集参与冲突图） | `zircon_runtime/src/scene/ecs/schedule.rs:36`、`scene_system_descriptor.rs`（`order: i32`）、`plugin/extension_registry/register/scene_hook.rs` |
| G2 | 无 Resource 注册扩展点 | `scene/ecs/resource_registry.rs` 有 `resource_id::<T>()`，但 registry 不暴露 |
| G3 | 无插件事件注册 API | `scene/ecs/events.rs`/`messages.rs` 存在；插件只能登记字符串目录 `register_plugin_event_catalog`（`extension_registry/register/metadata.rs:67`） |
| G4 | 扩展点是 13 个手写 `Vec<Descriptor>` 字段，无类型化注册表/冻结表 | `extension_registry/runtime_extension_registry.rs:16-31` |
| G5 | 注册项无 owner 追踪，卸载插件无法反查清理 | 同上；`SceneRuntimeHookDescriptor.plugin_id` 是唯一带 owner 的注册项 |
| G6 | Editor 侧 capability gate 只有按注册项的 `required_capabilities` 原语，无统一验证管线；`EditorPluginDescriptor::builtin_catalog()` 手工维护造成四源同步债 | `zircon_editor/src/core/editor_extension.rs:271-316`、`editor_plugin.rs:62` |
| G7 | Native ABI 仅 entry/unload + 单一 `ZrPluginApiV1`，无分域宿主函数表，Native 插件无法注册系统/组件进调度图 | `zircon_runtime_interface/src/plugin_api.rs` |
| G8 | 热重载有 `hot_reload.rs` 生命周期但无状态快照/回滚 | `plugin/native_plugin_loader/native_plugin_live_host/{hot_reload,lifecycle}.rs` |

已存在且 v2 直接复用的基建（不重建）：

- `SystemParamAccess`（`scene/ecs/system/system_param_access.rs`）：组件/资源/事件/消息读写集 + `conflicts_with` / `conflict_kinds_with`——即规划中的 SystemAccessDecl，**定稿沿用现名**。
- `ScheduleConflictGraph` + `conservative_parallel_batches()`（`scene/ecs/schedule_conflict_graph.rs`）与 `schedule_parallel_executor.rs`：并行执行器基建已在。
- `SceneScheduleStagePlan`（`scene/ecs/schedule_stage_plan.rs`）：executor 计划缓存，仅在定义变更时重建——v2 的 `CompiledSchedulePlan` 即它的拓扑序升级版（**保留现名 `SceneScheduleStagePlan`，下游文档统一用此名**）。
- `SystemParam` / `IntoSceneSystem` / `BoxedSceneSystem`（`scene/ecs/system/`）：系统参数机制完整。
- `PluginPackageManifest`（`plugin/package_manifest/plugin_package_manifest.rs`）与 `plugin.toml`（实例：`zircon_plugins/physics/plugin.toml`）：单源 manifest 字段已基本齐备（capabilities / capability_statuses / modules / optional_features / event_catalogs）。

## 3. 架构设计

### 3.1 调度标签体系（定稿）

#### 3.1.1 主阶段

扩展现有 `SystemStage`（`scene/ecs/system_stage.rs`）为 9 阶段，固定执行顺序以代码 `ORDER` 为准（RenderExtract 维持在帧末、Last 之后，与现实一致）：

```rust
// scene/ecs/system_stage.rs [改造]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemStage {
    First,
    PreUpdate,
    FixedFirst,      // [新增] 固定步循环内首段
    FixedUpdate,     // 现有
    FixedPostUpdate, // [新增] 固定步循环内末段
    Update,
    PostUpdate,
    Last,
    RenderExtract,
}
impl SystemStage {
    pub const COUNT: usize = 9;
    pub const ORDER: [Self; Self::COUNT]; // First → PreUpdate → FixedFirst → FixedUpdate → FixedPostUpdate → Update → PostUpdate → Last → RenderExtract
    pub const FIXED_LOOP: [Self; 3];      // [新增] FixedFirst / FixedUpdate / FixedPostUpdate，由 runner 按固定步长累加器整组循环
}
```

- `schedule_runner.rs` [改造]：`FIXED_LOOP` 三段作为一组在固定步累加器内循环执行（每次固定步依序跑三段），其余阶段每帧一次。累加器参数来自现有 time 资源；上限 4 次/帧防螺旋。
- 序列化兼容不保留：场景文档中的旧 stage 名直接按新枚举解析失败即报错（开发期硬切）。

#### 3.1.2 SystemSet 与偏序

```rust
// scene/ecs/system_set.rs [新增]
/// interned 字符串 → dense index；命名规则 "<plugin>.<set>"，如 "physics.simulation"。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SystemSetId(u32);

pub struct SystemSetRegistry { /* name → SystemSetId intern 表，注册期使用 */ }

// scene/ecs/scene_system_descriptor.rs [改造]
pub enum SystemRef {
    System(String),       // 系统 id，如 "physics.step"
    Set(SystemSetId),
}
pub enum SystemOrderingConstraint {
    Before(SystemRef),
    After(SystemRef),
}
pub struct SceneSystemDescriptor {
    pub id: String,
    pub stage: SystemStage,
    pub sets: Vec<SystemSetId>,                      // [新增]
    pub constraints: Vec<SystemOrderingConstraint>,  // [新增]
    pub order: i32,        // 保留：拓扑同层内的稳定 tie-break，不再是唯一排序手段
    pub system: InternalSceneSystem,
}
```

- `SceneScheduleStagePlan::from_registry` [改造]：按 stage 分组 → 约束展开（set 约束展开为成员系统间约束）→ 拓扑排序（环检测出 `ScheduleError::OrderingCycle { stage, chain: Vec<String> }` [新增变体]）→ 同层按 `order` 再按 id 稳定排序。计划仍只在定义变更时重建（沿用 `executor_plan_dirty` 机制）。
- 跨 stage 约束（如 Before 指向另一 stage 的系统）注册期报 `ScheduleError::CrossStageConstraint` [新增变体]——阶段顺序由 `ORDER` 唯一决定，约束只在 stage 内生效。

#### 3.1.3 标准系统锚点表（下游计划逐字引用）

| 锚点 id | 阶段 | 偏序 |
|---------|------|------|
| `net.poll_ingress` | First | — |
| `net.replication_apply` | PreUpdate | after `net.poll_ingress`（跨段语义由阶段顺序保证，无需显式约束） |
| `physics.step` | FixedUpdate | — |
| `script.fixed_update` | FixedUpdate | before `physics.step` |
| `physics.sync_to_scene` | FixedPostUpdate | — |
| `ai.behavior_tick` | Update | — |
| `navigation.agent_tick` | Update | after `ai.behavior_tick` |
| `script.update` | Update | — |
| `animation.evaluate` | PostUpdate | — |
| `sound.spatial_update` | PostUpdate | after `animation.evaluate` |
| `net.replication_collect` | PostUpdate | after `animation.evaluate` |
| `net.flush_egress` | Last | — |
| `script.gc_step` | Last | — |

标准 SystemSet：每个插件至少声明一个 `<plugin>.main` set（如 `physics.main`），其全部系统 in_set 之，供其他插件以 set 粒度声明 before/after 而不耦合具体系统 id。

### 3.2 RuntimeExtensionRegistry 注册 API（定稿）

在 `plugin/extension_registry/register/` 下新增三个 owner 模块。所有新 API 记录 `owner: PluginModuleId`：

```rust
// plugin/extension_registry/owner.rs [新增]
/// 由 PluginModuleManifest.name（如 "physics.runtime"）intern 而来。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PluginModuleId(u32);

// plugin/extension_registry/register/system_registration.rs [新增]
pub struct SystemRegistration {
    pub id: String,                                  // 锚点名，如 "physics.step"
    pub stage: SystemStage,
    pub sets: Vec<SystemSetId>,
    pub constraints: Vec<SystemOrderingConstraint>,
    pub order: i32,
    /// World 在 apply 阶段才可用，故注册时持构造闭包；
    /// SystemParamAccess 由 SystemParam::init_state 在构造时自动推导。
    pub build: Box<dyn FnOnce(&mut World) -> Result<BoxedSceneSystem, ScheduleError> + Send>,
}
impl RuntimeExtensionRegistry {
    pub fn register_system(&mut self, owner: PluginModuleId, registration: SystemRegistration)
        -> Result<(), RuntimeExtensionRegistryError>;
    /// 便捷封装：从 SystemParam 函数直接构造 SystemRegistration.build。
    pub fn register_native_system<P, S>(&mut self, owner: PluginModuleId,
        id: impl Into<String>, stage: SystemStage, system: S)
        -> SystemRegistrationBuilder            // builder 上挂 .in_set()/.before()/.after()/.with_order()
    where P: SystemParam + 'static, P::State: Send, S: IntoSceneSystem<P>;
}

// plugin/extension_registry/register/resource_registration.rs [新增]
pub struct ResourceRegistration<T: Resource> {
    pub init: Box<dyn FnOnce() -> T + Send>,
}
impl RuntimeExtensionRegistry {
    pub fn register_resource<T: Resource>(&mut self, owner: PluginModuleId, init: impl FnOnce() -> T + Send + 'static)
        -> Result<(), RuntimeExtensionRegistryError>;
}

// plugin/extension_registry/register/event_registration.rs [新增]
impl RuntimeExtensionRegistry {
    /// 类型化事件接入 World EventStore，并自动生成 PluginEventCatalogManifest 条目
    /// （现有字符串目录保留为 Native/VM 侧视图，由同一注册派生，消除双录）。
    pub fn register_event<E: Event>(&mut self, owner: PluginModuleId, catalog_entry: PluginEventManifestEntry)
        -> Result<(), RuntimeExtensionRegistryError>;
}
```

应用路径：`apply_to_world.rs` [改造] 消费 pending 的 system/resource/event 注册——resource 先注册（system 的 `SystemParam::init_state` 可能解析 `Res<T>`），event 次之，system 最后构造并经 `Schedule::register_system` 进计划。

跨插件通信规则：**只允许通过事件、插件调用桥（[11 计划](11-plugin-call-bridge.md)，强/弱依赖接口直调）或引擎本体 Manager**，禁止插件间直接类型依赖；事件读写已在 `SystemParamAccess`（`add_event_read/add_event_write`）参与冲突图。

### 3.3 类型化扩展点（解决 G4/G5）

```rust
// plugin/extension_registry/typed_extension_point.rs [新增]
pub trait ExtensionKey: Clone + Eq + std::hash::Hash {}
pub struct TypedExtensionPoint<K: ExtensionKey, V> {
    entries: Vec<(PluginModuleId, K, V)>,   // dense，注册序
    index: HashMap<K, u32>,                 // 仅注册期查重；finalize 后丢弃
}
impl<K: ExtensionKey, V> TypedExtensionPoint<K, V> {
    pub fn register(&mut self, owner: PluginModuleId, key: K, value: V)
        -> Result<(), RuntimeExtensionRegistryError>;   // 重复 key → DuplicateExtension
    pub fn finalize(self) -> FrozenExtensionTable<K, V>;
}
/// 运行期唯一持有形态：dense 槽位数组 + 排序 key 索引（二分或预解析 id 直查），无哈希。
pub struct FrozenExtensionTable<K: ExtensionKey, V> { /* slots: Box<[V]>, keys: Box<[K]>, owners: Box<[PluginModuleId]> */ }
impl<K: ExtensionKey, V> FrozenExtensionTable<K, V> {
    pub fn get(&self, slot: ExtensionSlot) -> &V;            // O(1) 直查
    pub fn resolve(&self, key: &K) -> Option<ExtensionSlot>; // 仅初始化/工具路径使用
    pub fn entries_owned_by(&self, owner: PluginModuleId) -> impl Iterator<Item = ExtensionSlot>;
}
#[derive(Clone, Copy)] pub struct ExtensionSlot(u32);
```

- `RuntimeExtensionRegistry` 的 13 个 `Vec<Descriptor>` 字段全部迁移为 `TypedExtensionPoint` 实例并**删除旧字段**（硬切）；`apply_to_*` 族改为消费 frozen 表。
- `ExtensionOwnership` [新增，`plugin/extension_registry/ownership.rs`]：跨扩展点的 owner → slots 反查索引，由各 frozen 表的 owners 列聚合而成；卸载/热重载失败时整体撤销该 owner 的全部注册。

### 3.4 插件生命周期（Frameworks02 硬切后的现行契约）

本段原先的 `PluginFinishContext` 四阶段 trait 已由
[Frameworks02 模块内核生命周期统一](../zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md)
硬切替代，不得重新实现或保留兼容入口。当前契约为：

```rust
pub trait RuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor;
    fn lifecycle(&self) -> &dyn ModuleLifecycle;
    fn register(&self, registry: &mut RuntimeExtensionRegistry)
        -> Result<(), RuntimeExtensionRegistryError> { Ok(()) }
}
```

- `RuntimePlugin::register` 只向 `RuntimeExtensionRegistry` 注册，不触碰 `World`。
- 生命周期由内嵌 `ModuleDescriptor` 的 `ModuleLifecycle` 表达，并由 `CoreRuntime` 统一执行
  `build → ready → finish → cleanup`；插件 trait 不再暴露 `ready`、`finish`、`activate` 或
  `deactivate`。
- `CapabilityView` 保持为注册报告的只读投影，用于 capability 查询，不再充当插件生命周期
  hook 的上下文。`PluginFinishContext`、`PluginRuntimeContext` 及 feature 的对称 hook 已删除。
- 可选 feature 的注册、catalog 投影与生命周期时序均服从同一 ModuleLifecycle 内核；相关动态
  验证仍使用 Frameworks02 和 Plugins01 的受管编译/测试门，未在本计划中重复声明通过。

### 3.5 Capability 协商对称化（解决 G6）

- 复用现有原语：`EditorExtensionRegistration::with_required_capabilities` / `is_enabled_by`（`zircon_editor/src/core/editor_extension.rs`）保留为 gate 执行点；新增统一验证管线 `EditorPluginCatalog::validate_capabilities()` [改造 `editor_plugin.rs`]，缺失 capability 输出结构化 `RegistrationDiagnostic` 而非静默禁用。
- `RegistrationDiagnostic { severity, code, plugin_id, message }` [新增] 下沉至 `zircon_runtime_interface/src/plugin_diagnostics.rs` [新增]，runtime/editor 共用。
- `EditorPluginDescriptor::builtin_catalog()`（`editor_plugin.rs:62`）改为由 `plugin.toml` 的 `[[modules]] kind = "editor"` 条目派生（build script 或 const 生成器，落点 `zircon_editor/src/core/editor_plugin_catalog_gen.rs` [新增]），消除四源同步债；现有 static manifest contract tests 扩展守护。

### 3.6 plugin.toml 单源 schema（定稿增量）

现有字段（见 `zircon_plugins/physics/plugin.toml` 与 `PluginPackageManifest`）保持；新增两节：

```toml
[[modules]]
name = "physics.runtime"
kind = "runtime"                  # runtime | editor（现有）
crate_name = "zircon_plugin_physics_runtime"
target_modes = ["client_runtime", "server_runtime", "editor_host"]
capabilities = ["..."]
system_sets = ["physics.main", "physics.simulation"]     # [新增] 声明本模块拥有的 SystemSet
system_anchors = ["physics.step", "physics.sync_to_scene"] # [新增] 对外承诺的锚点 id（契约测试核对实际注册）

[[optional_features]]              # 既有字段，语义按《多插件组合可选功能规则设计》定稿
id = "physics.jolt"
requires_capabilities = []         # [新增] 激活所需的他插件 capability
provides_capabilities = ["runtime.capability.physics.backend.jolt"]  # [新增]
```

四源一致性（plugin.toml / runtime descriptor / 派生 builtin catalog / workspace member）由生成器单向派生 + `static_manifest_contracts` 测试族守护：plugin.toml 为唯一手写源。

### 3.7 Native ABI v3（解决 G7/G8）

维持稳定 C ABI + byte payload 原则（不暴露 World 指针）。`zircon_runtime_interface/src/plugin_api.rs` [改造] 新增分域宿主函数表：

```rust
pub const ZR_PLUGIN_ENTRY_SYMBOL_V3: &[u8] = b"zircon_plugin_entry_v3\0";

#[repr(C)]
pub struct ZrHostApiV3 {
    pub abi_version: u32,            // = 3
    pub ecs: ZrHostEcsApiV1,
    pub asset: ZrHostAssetApiV1,
    pub event: ZrHostEventApiV1,
    pub diagnostics: ZrHostDiagnosticsApiV1,
}
#[repr(C)]
pub struct ZrHostEcsApiV1 {
    /// 注册 Native 系统：host 侧包装为正规 SystemRegistration（dynamic access → 保守调度，
    /// 即冲突图中按全写处理），系统在调度图里是一等公民。
    pub register_system: unsafe extern "C" fn(ZrRuntimePluginHandle, *const ZrSystemRegistrationV1) -> ZrStatus,
    pub register_component: unsafe extern "C" fn(ZrRuntimePluginHandle, *const ZrComponentDescV1) -> ZrStatus,
    pub spawn_command: unsafe extern "C" fn(ZrRuntimePluginHandle, *const u8, usize) -> ZrStatus, // 序列化命令缓冲
}
#[repr(C)]
pub struct ZrHostEventApiV1 {
    pub emit: unsafe extern "C" fn(ZrRuntimePluginHandle, ZrEventTypeId, *const u8, usize) -> ZrStatus,
    pub drain: unsafe extern "C" fn(ZrRuntimePluginHandle, ZrEventTypeId, ZrByteBufferRef) -> ZrStatus,
}
// ZrHostAssetApiV1 / ZrHostDiagnosticsApiV1 同构：函数指针 + handle + byte payload。
```

- host-side adapter（`plugin/native_plugin_loader/host_api_adapter.rs` [改造]）把 C 注册映射为 §3.2 的正规 `register_system` / `register_event`，owner 为该 native 插件的 `PluginModuleId`。
- 热重载（解决 G8）：`native_plugin_live_host/hot_reload.rs` [改造] 增加快照协议：

```rust
#[repr(C)]
pub struct ZrPluginStateSnapshotApiV1 {
    pub save:    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrByteBufferRef) -> ZrStatus,
    pub restore: unsafe extern "C" fn(ZrRuntimePluginHandle, *const u8, usize) -> ZrStatus,
}
pub struct PluginStateSnapshot { pub plugin_id: String, pub schema_version: u32, pub blob: Vec<u8> } // host 侧 [新增]
```

重载流程：`deactivate → save → 卸载注册（ExtensionOwnership 反查整体撤销）→ 加载新库 → register/finish/activate → restore`；任一步失败回滚：撤销新注册、重新 activate 旧库并 restore 旧快照。与 VM 的 `VmStateBlob`（08 计划）同构。

### 3.8 性能设计要点

- 注册期重、运行期零成本：验证/拓扑排序/intern/哈希全部在 finalize 前完成；帧循环内扩展点访问全为 `ExtensionSlot` 索引直查，调度执行按 `SceneScheduleStagePlan` 预排序列表顺跑。
- 并行执行：复用现有 `ScheduleConflictGraph::conservative_parallel_batches` 与 `schedule_parallel_executor.rs`；v2 注册的系统因携带 `SystemParamAccess` 自动参与分批。Native/VM 系统标记 dynamic access（保守全写），文档明确两档性能预期。
- 组件注册倾向静态类型组件（`register_component` 的 native storage 路径）；动态组件仅留给 VM/Native 插件。

## 4. 模块文件树

```
zircon_runtime/src/scene/ecs/
  system_stage.rs                       [改造] 9 阶段 + FIXED_LOOP
  system_set.rs                         [新增] SystemSetId / SystemSetRegistry
  scene_system_descriptor.rs            [改造] sets/constraints 字段、SystemRef/SystemOrderingConstraint
  schedule.rs                           [改造] register_system 走拓扑计划
  schedule_stage_plan.rs                [改造] 拓扑排序 + 环检测
  schedule_runner.rs                    [改造] FIXED_LOOP 固定步循环
  schedule_error.rs                     [改造] OrderingCycle / CrossStageConstraint 变体
zircon_runtime/src/plugin/extension_registry/
  owner.rs                              [新增] PluginModuleId
  ownership.rs                          [新增] ExtensionOwnership
  typed_extension_point.rs              [新增] TypedExtensionPoint / FrozenExtensionTable
  runtime_extension_registry.rs         [改造] 13 个 Vec → TypedExtensionPoint 实例
  register/system_registration.rs       [新增]
  register/resource_registration.rs     [新增]
  register/event_registration.rs        [新增]
  apply_to_world.rs                     [改造] 消费 resource → event → system
zircon_runtime/src/plugin/runtime_plugin/
  runtime_plugin/plugin.rs              [改造] 四阶段 trait
  lifecycle_context.rs                  [新增] PluginFinishContext / PluginRuntimeContext / CapabilityView
zircon_runtime/src/plugin/native_plugin_loader/
  host_callbacks.rs                     [改造] v3 adapter
  native_plugin_live_host/hot_reload.rs [改造] 快照回滚
zircon_runtime_interface/src/
  plugin_api.rs                         [改造] ZrHostApiV3 域函数表 + 快照 API
  plugin_diagnostics.rs                 [新增] RegistrationDiagnostic
zircon_editor/src/core/
  editor_plugin.rs                      [改造] validate_capabilities 管线
  editor_plugin_catalog_gen.rs          [新增] builtin catalog 由 plugin.toml 派生
```

## 5. 里程碑与任务分解

### M1 调度标签与系统注册（最大单项）

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M1-T1 | SystemStage 9 阶段 + FIXED_LOOP；runner 固定步循环 | system_stage.rs、schedule_runner.rs | — | `fixed_loop_stages_run_per_fixed_step`、`fixed_loop_clamps_to_max_steps_per_frame` |
| M1-T2 | SystemSetId/SystemRef/SystemOrderingConstraint；descriptor 扩展 | system_set.rs、scene_system_descriptor.rs | M1-T1 | `system_set_intern_is_stable` |
| M1-T3 | 拓扑排序进 SceneScheduleStagePlan；环/跨段约束诊断 | schedule_stage_plan.rs、schedule_error.rs | M1-T2 | `stage_plan_orders_by_constraints_then_order`、`ordering_cycle_reports_chain`、`cross_stage_constraint_rejected` |
| M1-T4 | registry.register_system / builder；apply_to_world 接线 | register/system_registration.rs、apply_to_world.rs、owner.rs | M1-T3 | `plugin_system_lands_in_stage_plan`、`plugin_system_access_joins_conflict_graph` |
| M1-T5 | physics/animation scene hook 迁移为 register_system（首批验证），删除两处 scene_hook 注册 | zircon_plugins/physics/runtime、zircon_plugins/animation/runtime 接线文件 | M1-T4 | `physics_step_anchor_registered_in_fixed_update`、`animation_evaluate_runs_after_physics_sync` |

### M2 资源、事件与类型化扩展点

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M2-T1 | register_resource / register_event + apply 顺序 | register/{resource,event}_registration.rs、apply_to_world.rs | M1-T4 | `plugin_resource_available_to_systems`、`plugin_event_catalog_derived_from_typed_registration` |
| M2-T2 | TypedExtensionPoint / FrozenExtensionTable 内核 | typed_extension_point.rs | — | `frozen_table_dense_lookup_matches_registration`、`duplicate_extension_key_rejected` |
| M2-T3 | 13 个 Vec 全量迁移 + 删除旧字段；apply_to_* 改消费 frozen 表 | runtime_extension_registry.rs、apply_to_*.rs 族 | M2-T2 | 既有 registry 测试全量改写为 frozen 路径 |
| M2-T4 | ExtensionOwnership 反查 + 卸载清理 | ownership.rs | M2-T3 | `owner_unload_revokes_all_slots` |

### M3 生命周期与可选功能对接

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M3-T1 | Frameworks02 `ModuleLifecycle` 硬切：`RuntimePlugin` 保留 descriptor、lifecycle 与 registration；plugin/feature 的 ready、finish、activate、deactivate 和旧上下文类型已删除 | runtime_plugin/plugin.rs、ModuleLifecycle、runtime plugin lifecycle tests | M2 | `runtime_plugin_lifecycle_hard_cuts_to_kernel_module_lifecycle` |
| M3-T2 | `CapabilityView` 是 registration report 的只读 capability 投影，不再是 plugin lifecycle hook 上下文；未注册 optional feature 不会提前成为可用 capability | runtime plugin registration reports、capability projection | M3-T1 | `runtime_plugin_lifecycle_hard_cuts_to_kernel_module_lifecycle` |
| M3-T3 | optional feature 的注册、catalog 投影与生命周期顺序统一由 `CoreRuntime` 执行 `build → ready → finish → cleanup`；不保留 `from_lifecycle_plugins` 或 feature finish 兼容入口 | CoreRuntime、ModuleLifecycle、package manifest/catalog | M3-T1/T2 | `runtime_plugin_lifecycle_hard_cuts_to_kernel_module_lifecycle` |

### M4 Editor 对称化与 manifest 单源

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M4-T1 | RegistrationDiagnostic 下沉 interface；editor 验证管线 | zircon_runtime_interface/src/plugin_diagnostics.rs、zircon_editor/src/core/plugin/catalog.rs | — | `editor_plugin_catalog_reports_missing_capabilities_as_structured_diagnostics` |
| M4-T2 | builtin catalog 由 plugin.toml 派生；删除手写表 | zircon_editor/build.rs、zircon_editor/src/core/plugin/catalog_gen.rs、zircon_editor/src/core/plugin/descriptor.rs | M4-T1 | `builtin_editor_catalog_entries_are_derived_from_plugin_manifests` |
| M4-T3 | system_anchors 契约：manifest 声明 vs 实际注册核对；新增静态守护，要求声明 anchor 的 runtime module 对应 crate 源码保留 runtime system 注册路径与 anchor id | static_manifest_contracts/modules/system_anchors.rs | M3-T3 | `declared_system_anchors_are_registered`（2026-06-13 warmed lib-test 二进制直接执行通过 1 项） |

### M5 Native ABI v3 与热重载快照

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M5-T1 | ZrHostApiV3 域函数表 + interface 类型；新增计划聚合别名守护 | plugin_api.rs、plugin_api_contracts.rs | M1/M2 | `abi_v3_layout_is_stable`（size/offset 断言；2026-06-13 focused run passed；`plugin_api_contracts` 新别名后 6 项全量通过） |
| M5-T2 | host adapter：C 注册 → 正规 register_*（dynamic access）；ABI native 系统进入 conservative world-access 调度节点 | host_api_adapter.rs | M5-T1 | `native_system_enters_schedule_as_conservative_node`（2026-06-13 core-min `cargo check` 与 focused Cargo 均通过；旧失败均为 target/进程/render 编译阻断，无断言失败） |
| M5-T3 | PluginStateSnapshot + 回滚流程；补齐计划命名别名，覆盖失败 restore 后旧快照恢复与失败注册 owner 撤销 | hot_reload.rs、lifecycle.rs、native_plugin_live_host/tests.rs、extension_registry_metadata.rs | M5-T2 | `hot_reload_failure_rolls_back_to_snapshot`、`failed_registration_revoked_via_ownership`（2026-06-13 warmed lib-test 二进制直接执行均通过） |
| M5-T4 | native_dynamic_fixture 保持 ABI v3 descriptor/entry；行为表硬切为 `NativePluginBehaviorV4` 的 dense slot + host-owned output。`abi_v2_only` 仅允许 descriptor/entry 元数据回退，禁止调用 V2 command/state/unload callback table | zircon_plugins/native_dynamic_fixture、behavior_calls.rs、native_plugin_loader.rs | M5-T3 | `native_behavior_v2_metadata_never_invokes_the_legacy_callback_table`、`native_behavior_v4_resolves_dense_slot_without_c_string_or_plugin_owned_buffer` |

## 6. 验收命令

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -ManifestPath zircon_plugins/Cargo.toml -SkipBuild
python tools/check_conventions.py
```

## 7. 风险

- M1 触及 `Schedule` 核心，physics/animation/sound 的 scene hook 全部需要同步迁移——按硬切原则一次换血，借助现有 scene hook 契约测试兜底；迁移完成前不删 `scene_hook` 模块，M1-T5 验收后整体删除。
- `SystemParamAccess` 自动推导依赖 `SystemParam::init_state` 的元数据完整性；现有 tuple impl 已携带访问集（`system_param_access.rs` 实查），但 M1-T4 需补"事件读写进访问集"的端到端断言。
- 序列化场景文档中的 stage 字段随枚举扩容而变；开发期硬切，但需在 M1-T1 同步更新所有内置场景 fixture。

## 8. 附录 · dev 参考源码对位

实现各任务前**必须先读对应参考实现**，接口形态与边界用例对照真实代码核对，禁止凭记忆/凭空实现：

| 设计点 | 参考源码（已核验存在） | 看什么 |
|--------|----------------------|--------|
| 调度标签/SystemSet/拓扑排序/环检测 | `dev/bevy/crates/bevy_ecs/src/schedule/` | ScheduleGraph 的约束展开与环报告、SystemSet 嵌套语义、ambiguity 检测形态 |
| Plugin 生命周期（build/finish/cleanup） | `dev/bevy/crates/bevy_app/src/plugin.rs`、`app.rs` | finish 的调用时机与重入保护、PluginGroup 组合 |
| 动态插件容器与热重载 | `dev/Fyrox/fyrox-impl/src/plugin/` | Plugin 容器 Static/Dynamic 双形态、重载时实体状态保持 |
| Native ABI/初始化分级/函数表 | `dev/godot/core/extension/gdextension_interface.cpp`、`gdextension.cpp`、`gdextension_manager.cpp` | initialization level 分阶段、C 函数表版本化与兼容策略（我们硬切版本但形态可借鉴） |
| 事件双缓冲与 cursor | `dev/bevy/crates/bevy_ecs/src/event/` | Events 双缓冲 update 时序、EventReader 漏读语义 |

## 9. 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1–M5 | current-source 静态恢复审计 | `current_source_static_complete / managed_dynamic_validation_pending` | 2026-08-02 | M1 固定步、SystemSet 与锚点注册，M2 typed/frozen/ownership，M3 ModuleLifecycle 硬切，M4 manifest/catalog/diagnostic，M5 ABI layout/V4 dispatch/hot-reload rollback 均在当前源码中复核；554 个 failure artifact、计划输出记录和插件结构审计通过。受管 validation-copy 属于其他 session，17 个 failure 在动态 focused/broad、独立复核和 fixed return 前保持 open。 |
| M2 | T2-T4 类型化扩展点稳定 slot、冻结状态与 runtime finalize/apply 接线 | `plugins_01_m2_t2_t4_typed_extension_freeze_runtime_finalize` | 2026-07-10 | `TypedExtensionPoint` 使用不复用的逻辑 `ExtensionSlot`、dense 行映射和 owner 撤销 tombstone，`FrozenExtensionTable` 保留稳定 slot 查询；`RuntimeExtensionRegistry::finalize()` 覆盖 20 个类型化扩展点，runtime/project catalog 与 world/component/UI/module/asset-manager apply 入口在读取前 finalize，后续注册、排序、可变访问或 owner 撤销会使冻结状态失效。新增 module-local 与 catalog/apply/owner 回归；当前源码经 scoped rustfmt 检查，直接编译实际 `typed_extension_point.rs` 的隔离测试 5/5 通过。完整 Cargo 测试阶段仍在执行，本行不声明 crate/workspace gate 通过。 |
| M5 / 横切 | Native generation/performance failure 收敛与 broad regression 对账 | `plugins_01_native_generation_focused_scale_green_broad_repairs_pending` | 2026-07-22 | RuntimePluginId Arc owner 模块 `7/7`，native discovery unchanged + alias exact 与 1k/10k zero-work benchmark、host context `3/3` + 1/16-thread benchmark、registration replay `5/5` + 1/100/1000 规模、package validation `13+1`、catalog projection `7/7`、profile availability `8/8`、bridge stable snapshot 9 项与 16M-call performance gate、native callback `10+3` 及 1/8/32 broadcast 均获得 immutable managed test-binary 证据。`tests::plugin_extensions` core-min broad baseline 为 `392 passed / 17 failed`；7 项同归 plugin workspace lock drift，2 项为 core-min/UI 配置语义，其余暴露并已修正 discovery watcher 去重、load-report parity fixture、typed loader diagnostic guard、bridge source guard、rendering 15-feature catalog parity、AI mixed capability status、event-consumer schema 和 runtime-scene-system anchor guard。Cargo-owned plugin lock 已由完整 offline metadata 物化唯一 `zircon_runtime -> arc-swap` edge并通过 locked metadata；fresh runtime 构建已 GREEN，但 broad 重跑、Plugin08/real-fixture consumers 与 failure fixed returns 尚未完成，本行不关闭 M5 或 Plugins01。 |
| M5 / 横切 | Native system access/affinity、bounded event transport、atomic callback generation lease | `implementation_complete / managed_focused_green_broad_pending` | 2026-07-22 | registration manifest v3 已编译精确 component/resource read/write 与 main/worker affinity，生产 runner 使用冲突图、World 外 worker batch、barrier 和 panic 后 system/deferred-state 恢复；event cursor/persistent subscription queue 已有 64 events/128 KiB page 与 16K/64 MiB backlog 硬预算；PERF-MVP-541 已把 stable callback owner改为 atomic transition-bit + in-flight count，diagnostics off 不取时钟/写统计，on 写 64 个 cache-line shard。current-source reservation `443ed28a879c42228127596358928d6a` / job `a2f858fdd9894cb88df122fb92780da9` / run `255a862363d74b699f0b23cf308dfe3b` 已自然 `exit 0`，完整 lib-test 构建 19m53s，callback 过滤组 `5 passed / 0 failed / 1 ignored`；同一二进制的 ignored 1/2/16/64-thread × 1M 门 `1 passed / 0 failed`，吞吐约 10.02M/10.05M/4.83M/5.12M leases/s，四档 state mutex acquire 均为 0。直接 access-manifest 宽组暴露类型 descriptor 注册时未预分配 ECS access id（`6 passed / 1 failed`）；`World::register_component_type` 已同步建立稳定 dynamic `ComponentId`。r10 `ed339385ebc94690a60ef20d83a8be1a` 因共享 export projection 源码在排队中合法变更而按 stale snapshot 主动释放；r11 `7b0564dd8fe94805ae300bb697d78d51` 又在 Render17 编译暴露 event test 穿透私有子模块后主动释放。`dynamic_api::session` 现仅 crate-subtree 重导出预算常量，test 不再访问私有 `event_mirror`；r12 因 core-min 不编译 linked-plugin test 而主动释放。default-feature r13 `41a0329396404c4d830c6987fe663225` / job `f156748ce96a4cfd9940221340fc04e9` / run `e19d2b8efd89450888f5c03d6839eb44` 在 67 路 current hash 无漂移下已越过 linked-plugin 类型检查与主 crate codegen，E0603 未再出现；终态仅被 Performance01 exact3 owner 的 `rhi/tests/device_contract/transfer_and_fences.rs` 六处遗留 `WgpuRenderDevice` E0433 阻断，正确修复是同步硬切为 `DeterministicRhiContractDevice` 与 `deterministic_rhi_contract_*` 测试前缀。Plugins01 未修改该路径，待 active lease owner 修复后复用 warmed target 增量复核。完整 offline lock resolution `b5c83a68db5a4b21be03e1bb74fba9f1` 已物化唯一预期 edge，locked metadata job `0cd27b89d9394a45a5b2817ebae2e3ce` `exit 0`。PERF-MVP-542 已 hard cut 到 behavior ABI v4、dense slot manifest 与 host-owned bounded output sink；current-source trace 进一步确认 ABI v3 loader 解析 V4 behavior、真实 fixture 导出 V4 callback/manifest，且 real-fixture 产品测试覆盖 echo、denied、panic 与 bounded overflow。managed dynamic Cargo 与实际产品运行 trace、其余 focused/broad 及 Plugin08 consumers 尚未完成，因此相关 failures 与 M5 保持 open。 |

- 2026-07-22 `r9` 不可变二进制追加 focused 证据：event mirror `9/9`、schedule runner `4/4`、native schedule diagnostics `1/1`、scheduled native systems `7/7`；ignored callback 1/2/16/64-thread × 1M 门 `1/1`，吞吐约 10.02M/10.05M/4.83M/5.12M leases/s 且四档 state mutex acquire 均为 0。ECS access-id 与 event session 可见性修复以 default-feature current-hash `r13` reservation `41a0329396404c4d830c6987fe663225` 的 source-bound 结果为准；旧 r10-r12 均已因后续合法源码变化或 feature 覆盖不足而主动释放，不能作为验收证据。
- 2026-07-22 `r9` native lifecycle 补充：host API/context `22/0/1 ignored`、runtime behavior `11/0/1 ignored`、hot reload/rollback `12/12`；ignored context 1/16-thread × 1M 门 writer acquire 0，ignored 1/8/32-plugin broadcast 门完成 100/800/3200 callbacks。PERF-MVP-541 的 focused lifecycle/scale 已 GREEN，仍不把产品 trace 或 PERF-MVP-542 记为完成。

- 2026-07-22 fresh gate 追踪：默认 feature 的 `zircon_runtime` lib-test 编译尚未进入 Plugins01 测试执行，即命中既有 [Text04 raster target/completion API drift](../zircon_runtime/text/04/failure-2026-07-22-raster-target-completion-api-drift.md)（`TextRasterCompletionDrain` 字段、`drain_completed_for_target` 方法及 `TextRasterWorkResult::target` 未原子同步；受管 job `5833ef96101f45c9a88be871b66381e8` exit 101）。该跨计划阻断由 Text owner 修复；Plugins01 不写入其源码，也不把测试数记为本计划失败。core-min fresh discovery/broad、plugin workspace lock 物化与 source-bound default broad 预约仍按 Windows FIFO 等待，因此本状态保持 pending。
- 2026-07-22 plugin-event transport 设计根已确认：现有 `EventCursor::read` 在 iterator 返回前预推进到队尾，不能用 `.take(page_limit)` 实现有界排水；参考 Bevy 的消费时推进 cursor 仍只有两帧保留，也不能承担 Editor backlog authority。Plugins01 必须引入 subscription-owned persistent pending authority，并把 cursor advance、JSON bytes 与成功返回作为同一提交边界；冻结 `ZrRuntimeApiV3` 不增字段，无法由等价固定 page 完整表达预算时走新 table version 与 host hard cut。详见 [plugin-event drain 帧预算交接](01/failure-2026-07-22-plugin-event-drain-frame-budget.md)；当前仅关闭设计根分析，不认领实现或动态验收。

- fixed 已修复（finalize/read boundary guard 已对齐 current world plan owner）：[extension-registry-finalize-coverage-guard-drift](../zircon_editor/editor/09/fixed-2026-07-15-extension-registry-finalize-coverage-guard-drift.md)。
> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- 性能交接归档：[`01/2026-08-01-performance-review-handoffs.md`](01/2026-08-01-performance-review-handoffs.md)（16 条原始记录，迁移不改变 open 状态）

插件架构整体状态：`resolving_failure`。M1–M4 的实现契约已获当前源码静态复核，M5 的 ABI/behavior/hot-reload 静态契约也已复核；17 个 Plugins01 failure 仍须通过受管 dynamic focused/broad、独立复核与 fixed return 关闭，后续编号计划及全局验收命令仍须逐项完成和验证。

- open 待修复：[bridge-import-stable-call-double-mutex](01/failure-2026-07-17-bridge-import-stable-call-double-mutex.md)、[native-host-api-global-context-lock](01/failure-2026-07-17-native-host-api-global-context-lock.md)、[native-load-report-repeated-projection](01/failure-2026-07-17-native-load-report-repeated-projection.md)、[native-plugin-callback-global-lock](01/failure-2026-07-17-native-plugin-callback-global-lock.md)、[native-plugin-discovery-recursive-rescan](01/failure-2026-07-17-native-plugin-discovery-recursive-rescan.md)、[native-registration-replay-per-system-rebuild](01/failure-2026-07-17-native-registration-replay-per-system-rebuild.md)、[native-systems-conservative-world-writer-serialization](01/failure-2026-07-17-native-systems-conservative-world-writer-serialization.md)、[package-validation-quadratic-uniqueness-scans](01/failure-2026-07-17-package-validation-quadratic-uniqueness-scans.md)、[runtime-plugin-catalog-derived-projection-rebuild](01/failure-2026-07-17-runtime-plugin-catalog-derived-projection-rebuild.md)、[runtime-profile-availability-rebuild](01/failure-2026-07-17-runtime-profile-availability-rebuild.md)、[runtime-plugin-id-interner-ownership](01/failure-2026-07-19-runtime-plugin-id-interner-ownership.md)、[native-callback-per-call-lease-and-abi-copy](01/failure-2026-07-22-native-callback-per-call-lease-and-abi-copy.md)、[plugin-event-drain-frame-budget](01/failure-2026-07-22-plugin-event-drain-frame-budget.md)、[plugin-workspace-lockfile-drift](01/failure-2026-07-22-plugin-workspace-lockfile-drift.md)、[native-live-key-hot-reload-contract-drift](01/failure-2026-07-27-native-live-key-hot-reload-contract-drift.md)、[native-discovery-compile-boundary](01/failure-2026-07-30-native-discovery-compile-boundary.md)、[runtime-scene-system-shared-callback-state](01/failure-2026-07-30-runtime-scene-system-shared-callback-state.md)。
