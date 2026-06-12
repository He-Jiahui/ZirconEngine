# 11 · 插件调用桥计划（强/弱依赖 · 接口直调 · 事件机制优化）

> 状态：工程化细化版 v2 · 优先级：P1（横切基建，02/05/06/07 的可选依赖调用均建立在其上）
> 前置：[01 插件架构核心](01-plugin-architecture-core.md) M2（TypedExtensionPoint/owner）、M3（finish/CapabilityView）；M3 里程碑另依赖 01-M5（热重载快照）
> 参考实现：Godot GDExtension 的 method bind 表（预解析槽位直调）、Bevy Events（双缓冲 + cursor）、OSGi 服务注册表（强/弱服务绑定语义，仅取形态）

## 1. 目标

为插件之间提供统一的**调用桥（Plugin Call Bridge）**：

1. **强依赖**：依赖方在注册期声明，目标插件必须启用——闭包验证在计划期完成，运行期调用**零检查零查找**（直接引用 + 虚表调用）。
2. **弱依赖**：目标插件可缺席/可禁用——调用方拿到的桥永远有效，目标未启用时调用返回结构化状态 `BridgeError::NotEnabled`，启用后自动接通；热路径成本恒为 **1 次原子读 + 1 次虚表调用**。
3. **事件机制优化**：事件通道 dense id 化、无订阅者通道零成本、弱依赖事件订阅休眠/自动接通，与调用桥共用同一启用/世代模型。

调用桥与既有机制的关系裁决：**Manager 保留为引擎本体的全局服务单例；插件之间的一切同步调用一律经调用桥**（替代 01 §3.2 原"只允许事件或 Manager"中对插件间 Manager 直查的容忍）；capability 仍是声明/探测层（finish 期决策"要不要接"），bridge 是调用层（运行期"怎么调"）。

## 2. 现状基线（实查）

- 依赖声明：`PluginPackageManifest.dependencies`（`plugin/package_manifest/plugin_dependency_manifest.rs`）已有 `{ id, required: bool, capability: Option<String> }`——**`required` 字段即强/弱语义的现成单源**，缺接口粒度声明。
- 跨插件调用现状：无标准通道——sound occlusion、ai sight 等计划目前只到"CapabilityView 探测"一层，实际调用要么经 Manager 全局查找（字符串/TypeId），要么无法表达"对方未启用"的状态。
- 事件：`scene/ecs/events.rs` 的 `Events<T>`（send/send_batch/update 双缓冲 + `EventCursor` 读游标）形态良好；但 `EventStore` 以 `TypeId` 哈希查通道（`events<T>()`），每次收发都付哈希成本，且无"无订阅者早退"。
- 01-M2 交付的 `TypedExtensionPoint`/`FrozenExtensionTable`/`PluginModuleId` 是本计划的直接地基。

缺口：

| # | 缺口 |
|---|------|
| C1 | 无接口级导出/导入声明与注册 API；插件间调用无类型安全通道 |
| C2 | 无强依赖闭包验证（缺目标时应拒绝加载并指明依赖链）；无弱依赖"未启用"结构化状态 |
| C3 | 无启用/禁用/热重载下的桥一致性模型（世代号） |
| C4 | EventStore 哈希查通道；无 dormant 通道；事件类型无 dense id |
| C5 | Native/VM 插件无桥 ABI 通道 |

## 3. 架构设计

### 3.1 接口声明（plugin.toml 单源 + 中立契约 trait）

接口 trait 定义在 `zircon_runtime::core::framework::<domain>`（中立契约层），实现在提供方插件；版本进 id，破坏性变更开新 id（同插件可同时导出 v1/v2 平滑过渡）：

```toml
# 提供方 plugin.toml
[[provides_interfaces]]
id = "physics.query.v1"          # [新增节] 导出接口目录，契约测试核对实际 export

# 依赖方 plugin.toml —— 复用现有 dependencies 节，required 即强/弱
[[dependencies]]
id = "physics"
required = false                  # true = 强依赖；false = 弱依赖（现有字段）
interfaces = ["physics.query.v1"] # [新增字段] 本插件实际导入的接口
```

```rust
// core/framework/bridge.rs [新增] 中立契约
pub trait PluginInterface: Send + Sync + 'static {
    /// 全局唯一接口 id，如 "physics.query.v1"；注册期 intern 为 InterfaceSlot。
    const INTERFACE_ID: &'static str;
}
// 示例（framework::physics [改造] 增加）：
pub trait PhysicsQueryInterface: PluginInterface {
    fn ray_cast(&self, query: &PhysicsRayCastQuery, filter: &PhysicsQueryFilter,
                out: &mut Vec<PhysicsRayCastHit>) -> Result<(), PhysicsBackendError>;
}
```

接口实现的并发契约：实现体 `Send + Sync`，只触碰提供方插件内部状态，**不得访问 World**（World 访问一律走系统 + `SystemParamAccess`，保证调用桥不绕过调度器的冲突图）。

### 3.2 注册与解析（`plugin/bridge/` [新增]，对接 01 生命周期）

```rust
// 注册期（RuntimePlugin::register）—— 提供方导出：
impl RuntimeExtensionRegistry {
    pub fn export_interface<T: PluginInterface + ?Sized>(
        &mut self, owner: PluginModuleId, implementation: Arc<T>,
    ) -> Result<(), RuntimeExtensionRegistryError>;       // 重复导出同 id → DuplicateExtension
}

// finish 期（RuntimePlugin::finish）—— 依赖方解析：
impl PluginFinishContext<'_> {
    /// 强依赖：目标缺席/未启用 → Err（含依赖链诊断），本插件激活失败。
    pub fn resolve_strong<T: PluginInterface + ?Sized>(&self)
        -> Result<StrongBridge<T>, RuntimeExtensionRegistryError>;
    /// 弱依赖：永远成功；目标缺席时返回 dormant 桥。
    pub fn resolve_weak<T: PluginInterface + ?Sized>(&self) -> WeakBridge<T>;
}
```

- 解析必须与 plugin.toml 的 `dependencies.interfaces` 声明一致（多解析/漏声明 → 契约测试失败），保持四源一致性纪律。
- 强依赖闭包验证（解决 C2）：finalize 前对全部 `required = true` 依赖做拓扑验证，缺失输出 `RegistrationDiagnostic { code: "bridge.strong_dependency_missing", chain: ["ai" → "navigation" → "physics"] }`；强依赖目标在运行期**拒绝单独禁用**（请求被拒并列出依赖者，禁用必须从依赖链叶子开始）。

### 3.3 桥的运行期形态与性能模型（解决 C1/C3）

```rust
// plugin/bridge/table.rs [新增]
/// finalize 产物：dense 槽位数组，运行期唯一权威。
pub struct FrozenBridgeTable { entries: Box<[BridgeEntry]> }
struct BridgeEntry {
    /// 类型擦除的提供者；启停/热重载经 ArcSwapOption 原子替换。
    provider: arc_swap::ArcSwapOption<dyn Any + Send + Sync>,
    /// 偶数 = 启用，奇数 = 禁用/缺席；每次状态翻转 +1。单次 Acquire 读完成"是否启用 + 是否变代"双判定。
    generation: AtomicU32,
    owner: PluginModuleId,
}

// plugin/bridge/strong.rs [新增]
/// 强依赖桥：finish 解析后即为直接引用，调用 = 一次虚表调用，无任何检查。
pub struct StrongBridge<T: ?Sized> { target: Arc<T> }
impl<T: ?Sized> std::ops::Deref for StrongBridge<T> { type Target = T; /* … */ }

// plugin/bridge/weak.rs [新增]
pub struct WeakBridge<T: ?Sized> {
    slot: InterfaceSlot,                       // dense 索引
    cached: UnsafeCell<Option<(u32, Arc<T>)>>, // (已验证世代, 已 downcast 引用) —— 慢路径填充
}
impl<T: PluginInterface + ?Sized> WeakBridge<T> {
    /// 热路径：1 次原子读判世代——命中缓存世代 → 直接虚调用；
    /// 世代变更 → 慢路径重新 downcast（每次启停后仅一次）；奇数世代 → NotEnabled。
    pub fn call<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, BridgeError>;
    /// guard 形态：系统体起始处 pin 一次，整个系统执行期内免重复检查
    ///（安全性：启停/热重载仅发生在帧边界，由 01 生命周期保证——帧内 pin 不会悬挂）。
    pub fn pin(&self) -> Result<BridgeGuard<'_, T>, BridgeError>;
    pub fn is_enabled(&self) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeError { NotEnabled /* 目标插件未启用/已禁用 */, Absent /* 目标未安装 */ }
```

性能预算（写进基准测试）：StrongBridge 调用 ≈ 裸 `dyn` 调用；WeakBridge 命中路径 ≤ 裸调用 + 1 次 Acquire load + 1 次分支；`pin` 后批量调用与 StrongBridge 同价。

启停/热重载一致性（解决 C3）：`activate` → 写 provider + 世代置偶；`deactivate` → 世代置奇 + 清 provider（帧边界执行，01 §3.4 时序保证）；热重载 = deactivate → 替换 → activate，世代跨两次递增，所有 WeakBridge 自动慢路径重连——**调用方零感知、零接线**。

### 3.4 事件机制优化（解决 C4，`scene/ecs/events.rs` [改造] + `event_channel.rs` [新增]）

1. **dense 通道**：`register_event::<E>`（01-M2）注册期 intern `EventTypeId(u32)`；`EventStore` 从 `TypeId → Box<dyn Any>` 哈希图改为 `Box<[ErasedEventChannel]>` 槽位数组 + 注册期一次性的 TypeId→id 映射（仅慢路径/动态侧使用）。`EventReader<E>/EventWriter<E>` SystemParam 在 `init_state` 解析一次槽位，**运行期收发零哈希**。
2. **统一翻转**：现 `Events<T>::update`（双缓冲交换）收口为 `events.update_all` 内置系统 ∈ First（在 `net.poll_ingress` 之前，order 前置），删除散落的手动 update 调用点。
3. **dormant 通道（无订阅者零成本）**：finalize 后无任何 reader 声明的通道标记 inactive——`EventWriter::send` 编译为一次分支早退，不写缓冲；弱依赖插件 activate 注入新 reader 时通道激活（与 §3.3 同一世代模型）。发布者对"订阅者可能不存在"完全无感。
4. **弱依赖事件订阅**：订阅目标插件（weak）声明的事件类型时，目标缺席 → cursor 进入 dormant 列表；目标 activate 时按事件类型 id 自动接通并从接通点开始读（不回放历史，语义与 `EventCursor` 现行为一致）。
5. **容量与分配**：通道按高水位自适应预分配（帧末收缩滞后 N 帧防抖）；大 payload 准则：> 128 字节的事件类型 payload 走 `Arc`，契约测试静态断言尺寸。
6. **选择准则（写进规范）**：需要同帧返回结果的查询/命令 → 调用桥；单向通知/一对多广播/帧间解耦 → 事件。禁止用事件模拟同步调用（回包事件反模式），禁止用桥做广播。

### 3.5 Native / VM 插件接入（解决 C5）

- ABI v3（01 §3.7）新增域表：

```rust
#[repr(C)]
pub struct ZrHostBridgeApiV1 {
    /// 预解析槽位直调：interface_slot/method_slot 在加载期经反射描述解析一次（与 08 CompiledCallSite 同机制）。
    pub call: unsafe extern "C" fn(ZrRuntimePluginHandle, u32 /* interface_slot */, u32 /* method_slot */,
                                   *const u8, usize, ZrByteBufferRef) -> ZrStatus,
}
// ZrStatus 新增码：ZR_STATUS_BRIDGE_NOT_ENABLED（弱依赖未启用，对应 BridgeError::NotEnabled）
```

- Native/VM 插件之间不直接互调，一律经 host 桥表（owner 追踪、世代一致性、诊断免费获得）；VM 侧经 [08](08-zr-vm.md) §3.2 的 `ScriptCallTable` 走同一槽位。

### 3.6 管理与诊断

- `FrozenBridgeTable` 随 01 的 `finalize()` 一并冻结；`ExtensionOwnership` 反查覆盖 export 条目（卸载插件 → 其导出接口全部置奇世代）。
- 诊断（[10 规范](10-editor-integration.md) §5）：每槽位调用计数与最近 `NotEnabled` 次数（debug 构建原子计数，release 编译为空操作）进 rolling diagnostics；editor 增加桥矩阵视图（提供方 × 依赖方 × 状态），挂 `view.core.plugin_bridges`。

## 4. 模块文件树

```
zircon_runtime/src/core/framework/bridge.rs        [新增] PluginInterface trait / BridgeError
zircon_runtime/src/plugin/bridge/
  interface_id.rs                                  [新增] InterfaceSlot intern
  table.rs                                         [新增] FrozenBridgeTable / BridgeEntry / 世代协议
  strong.rs                                        [新增] StrongBridge
  weak.rs                                          [新增] WeakBridge / BridgeGuard
  diagnostics.rs                                   [新增] 调用计数（debug）
zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs  [新增] export_interface
zircon_runtime/src/plugin/runtime_plugin/lifecycle_context.rs  [改造] resolve_strong/resolve_weak
zircon_runtime/src/plugin/package_manifest/plugin_dependency_manifest.rs  [改造] interfaces 字段
zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs     [改造] provides_interfaces 节
zircon_runtime/src/scene/ecs/events.rs             [改造] dense 通道 + dormant
zircon_runtime/src/scene/ecs/event_channel.rs      [新增] ErasedEventChannel / 槽位表
zircon_runtime_interface/src/plugin_api.rs         [改造] ZrHostBridgeApiV1 + 新状态码
```

## 5. 里程碑与任务分解

### M1 桥核心（强/弱语义闭环）

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M1-T1 | PluginInterface/InterfaceSlot/export_interface | framework/bridge.rs、interface_id.rs、bridge_registration.rs | 01-M2 | `duplicate_interface_export_rejected` |
| M1-T2 | FrozenBridgeTable + 世代协议 | table.rs | M1-T1 | `generation_parity_encodes_enabled_state` |
| M1-T3 | StrongBridge：finish 解析 + 闭包验证 + 依赖链诊断 | strong.rs、lifecycle_context.rs | 01-M3、M1-T2 | `missing_strong_dependency_fails_with_chain`、`strong_call_has_no_runtime_check`（基准断言） |
| M1-T4 | WeakBridge：call/pin/NotEnabled + 缓存世代慢路径 | weak.rs | M1-T2 | `weak_call_returns_not_enabled_when_target_absent`、`weak_call_hot_path_single_atomic_load`、`pin_guard_amortizes_checks` |
| M1-T5 | plugin.toml 双节解析 + 声明/解析一致性契约 | package_manifest/* | M1-T3/T4 | `resolved_interfaces_match_manifest_declaration` |

### M2 事件机制优化

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M2-T1 | EventTypeId dense 通道 + Reader/Writer 槽位解析 | events.rs、event_channel.rs | 01-M2 | `event_send_receive_has_no_hash_lookup`、既有 Events 测试保绿 |
| M2-T2 | events.update_all ∈ First 收口 | events.rs、注册路径 | 01-M1、M2-T1 | `double_buffer_swaps_once_per_frame` |
| M2-T3 | dormant 通道 + 无订阅者早退 | event_channel.rs | M2-T1 | `unsubscribed_channel_emit_is_branch_only`、`channel_activates_on_late_subscriber` |
| M2-T4 | 弱依赖事件订阅休眠/接通 | event_channel.rs、weak.rs | M1-T4、M2-T3 | `dormant_subscription_connects_on_plugin_activate` |

### M3 动态启停与热重载一致性

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M3-T1 | activate/deactivate 帧边界世代翻转；强依赖禁用拒绝 | table.rs、生命周期接线 | M1、01-M3 | `disable_strong_target_rejected_with_dependents`、`weak_bridge_reconnects_after_reenable` |
| M3-T2 | 热重载经世代跨两次递增自动重连（含 Native） | table.rs、native live host 接线 | M3-T1、01-M5 | `hot_reload_swaps_provider_without_caller_rewiring` |

### M4 Native/VM 接入与诊断

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M4-T1 | ZrHostBridgeApiV1 + NOT_ENABLED 状态码 + host adapter | plugin_api.rs、host_callbacks.rs | 01-M5、M1 | `native_bridge_call_round_trips`、`native_weak_call_maps_not_enabled_status` |
| M4-T2 | VM 经 ScriptCallTable 接桥 | 08 host_interface 接线 | 08-M2、M4-T1 | `vm_bridge_call_uses_preresolved_slots` |
| M4-T3 | 调用计数诊断 + editor 桥矩阵视图 | diagnostics.rs、editor | [10 规范](10-editor-integration.md)、M1 | `bridge_diagnostics_paths_registered`、editor 契约测试 |

## 6. 对既有计划的接线变更（落地时同步修订）

- [01](01-plugin-architecture-core.md) §3.2 跨插件通信规则更新为："只允许通过**事件、调用桥（本计划）**或引擎本体 Manager；禁止插件间直接类型依赖"。
- [02 Sound](02-sound.md) occlusion、[05 Navigation](05-navigation.md) 几何收集、[06 AI](06-ai.md) sight raycast：finish 期 `CapabilityView` 探测语义不变，运行期调用从"Manager 查找"升级为 `WeakBridge<PhysicsQueryInterface>`（physics 侧在 [03](03-physics.md) M2 后导出 `physics.query.v1`）。
- [06 AI](06-ai.md) MoveTo 写组件字段、[03↔04](04-animation.md) ragdoll 双资源通道**维持现状**——组件/资源数据流不属于调用桥范畴（经 ECS 与调度器管理）。

## 7. 验收命令

```bash
cargo test -p zircon_runtime --lib --locked
cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked
```

## 8. 风险

- `WeakBridge` 的缓存重验依赖"启停只在帧边界"这一生命周期不变量；若未来引入帧内禁用，需把 `pin` 升级为读写世代锁——在 table.rs 注释中显式记录该不变量，并以 `debug_assert` 在非帧边界翻转时炸断。
- `ArcSwapOption<dyn Any>` 的 downcast 仅在慢路径发生，但接口 trait 必须 `'static`；带借用参数的接口经引用参数传递（如 `&mut Vec<…>` out 参数），禁止返回内部引用——接口设计准则写入 framework/bridge.rs 文档注释。
- 事件 dense 化触及 `EventStore` 全部调用点（runtime 内部 + 插件），与 01-M2 的 register_event 同窗口落地避免两次迁移。
- dormant 通道的"激活时不回放历史"语义要在文档与测试中钉死，避免插件作者误以为可收到激活前事件。
