# 08 · ZrVM 插件完善计划（反射 / 接口注册 / GC 与生命周期管理）

> 状态：工程化细化版 v2 · 优先级：P2 · 前置：[01 插件架构核心](01-plugin-architecture-core.md) M1–M3
> 关联计划：`.codex/plans/ZrVM 语言插件与反射注册计划.md`（五里程碑维持有效） · 现状文档：`docs/zircon_plugins/zr_vm_language/runtime.md`
> 参考实现：Godot GDExtension（class registration、initialization level）、Piccolo（编译期反射注入 + TypeMeta/FieldAccessor/MethodAccessor 形态）

## 1. 目标

把 `zircon_plugins/zr_vm_language` 从 wrapper 推进到完整 VM 集成层：完备的双向反射（宿主类型导出给 VM、VM 类型注册回宿主）、接口注册（VM 实现引擎扩展点）、GC 与宿主句柄生命周期协约、热替换状态迁移生产化。`E:\Git\zr_vm` 后端经当前 manifest 声明的 `backend-zr-vm` feature 接入。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-plugins-zr-vm",
  "goal": "完成 ZrVM 插件的双向反射、接口注册、GC 生命周期、真实后端和热替换架构",
  "milestones": [
    {"id": "M1", "title": "反射注册表统一", "depends_on": []},
    {"id": "M2", "title": "接口注册四通道", "depends_on": ["M1"]},
    {"id": "M3", "title": "GC 与宿主句柄生命周期协约", "depends_on": ["M1"]},
    {"id": "M4", "title": "backend-zr-vm 真实后端全链路接通", "depends_on": ["M2", "M3"]},
    {"id": "M5", "title": "热替换状态迁移生产化", "depends_on": ["M1"]}
  ]
}
```

## 2. 实施前基线（原始实查）

计划编制时，统一反射模型的 DTO 与宿主侧注册表已在，缺的是 derive、反向通道与性能层。以下内容保留为设计来源，不代表当前实现状态；当前状态以第 5 节产出记录与 open failure 为准。

- **接口层** `zircon_runtime_interface/src/reflect/`：`ReflectTypeInfo`/`ReflectFieldInfo`/`ReflectTypePath`/`ReflectTypeKind`/`ReflectTypeRegistration`（带 `plugin_owned`/`serializable`/`editor_visible`/`remote_visible` 旗标——天生面向 inspector/replication/VM 三消费方）/`ReflectObjectAddress`/`ReflectedValue`/`ReflectFieldValue` + Read/Write/Fields 请求响应对/`ReflectSchemaRequest`/`ReflectEditorHint`（数值范围/枚举选项，inspector 用）/`ReflectError`；契约测试在 `tests/reflect_contracts.rs`。
- **runtime 侧** `zircon_runtime/src/scene/reflect/`：`type_registry.rs`、`world_reflection.rs`（World 级反射读写）、`dynamic_component.rs`、`fixed/`（内置组件的手写反射实现）。
- **zr_vm 插件** `zircon_plugins/zr_vm_language/runtime/src/`：`backend.rs`（`ZrVmBackend`/`ZrVmBackendFamily` wrapper）、`real_backend/`（instance/host_modules/values/package/lock/errors，HotReloadCoordinator 与 VmStateBlob 雏形在 `instance.rs`）、`module.rs`；`backend = "zr_vm:project"` 包协议与 scene hook（script fixed_update/update）既定。

缺口（按严重度）：

| # | 缺口 | 证据 |
|---|------|------|
| V1 | 无 derive 宏：全部反射实现手写（`scene/reflect/fixed/` 逐组件），插件组件无法低成本进反射表 | workspace 无 proc-macro crate |
| V2 | 反射单向（宿主→读写）：VM 声明类型注册回宿主无通道 | `type_registry.rs` 无外部 register 入口 |
| V3 | 无 dense call site：反射读写走 `ReflectTypePath` 字符串解析，热路径不可用 | `reflect/read_write.rs` 请求形态 |
| V4 | VM 实现宿主接口（BT 节点/RPC handler/系统）无标准通道 | `real_backend/host_modules.rs` 仅宿主→VM 方向 |
| V5 | GC 与宿主 handle 交叉引用无生命周期协约 | `real_backend/values.rs` |
| V6 | 热替换 `VmStateBlob` 无 schema 版本与字段级迁移 | `real_backend/instance.rs` |

## 3. 架构设计

核心原则维持：VM 不见 Rust 裸指针，一切经稳定 HostHandle + capability + 反射描述。**统一反射模型 = 现有 `zircon_runtime_interface::reflect` DTO 家族**，inspector（[10](10-editor-integration.md) E1）、replication（[07](07-net.md) §3.4/§3.5）、VM 三方共享，不另起类型。

### 3.1 derive 宏与注册表统一（解决 V1/V2，根工作区 [新增] proc-macro crate）

```
zircon_reflect_derive/        [新增 crate，root workspace member]
  src/lib.rs                  #[derive(ZrReflect)]：为 struct/enum 生成 ReflectTypeInfo +
                              字段 getter/setter（ReflectFieldValue 编解码）+ ReflectTypeRegistration 构造
```

```rust
#[derive(ZrReflect)]
#[zr_reflect(component, script_visibility = "public")]   // 属性进 ReflectTypeRegistration 旗标
pub struct Health { pub current: f32, pub max: f32 }
```

- `scene/reflect/type_registry.rs` [改造] 增加双向入口：
  - 宿主导出：组件（含插件静态组件）、Manager 接口、事件类型经 derive 产物自动登记（owner 插件可声明 `script_visibility`）；`fixed/` 手写实现迁移为 derive 后**删除**。
  - VM 注册：`register_vm_type(ReflectTypeRegistration, VmTypeBacking)`——VM 声明的类型进入同一注册表，宿主侧以动态组件（`dynamic_component.rs` 现路径）+ 反射访问消费。
- **全引擎一份反射模型**：07 的 replication payload schema、10 的默认 drawer 均消费 `ReflectTypeInfo`，注册表为唯一权威。

### 3.2 dense call site（解决 V3，`zr_vm_language/runtime/src/call_site.rs` [新增]）

```rust
/// 注册期把 ReflectTypePath/方法名解析为槽位；运行期调用零字符串。
pub struct CompiledCallSite { pub type_slot: u32, pub member_slot: u32, pub layout: ParamLayout }
pub struct ScriptCallTable { /* 模块加载期构建：导出表 → dense 槽位数组 */ }
```

- VM↔宿主调用（方法调用、字段读写）全部经预解析 call site（方法 id + 参数布局）；注册期后字符串查找零出现（断言进测试）。

### 3.3 接口注册：VM 实现引擎扩展点（解决 V4，`runtime/src/host_interface/` [新增]）

```rust
/// VM 函数稳定句柄：热替换后世代失配自动重解析。
#[derive(Clone, Copy)] pub struct VmCallbackHandle { pub module: u32, pub function: u32, pub generation: u32 }
```

经 host handle 暴露的扩展点注册（v1 集合，全部走 capability gate，未授权返回 `CapabilityDenied`）：

| 通道 | 映射到 | 说明 |
|------|--------|------|
| `register_system` | 01 §3.2 `SystemRegistration`（dynamic access → 保守调度） | VM 系统进调度图 |
| `register_bt_node` | [06 AI](06-ai.md) §3.1 节点目录（ScriptTask/自定义节点） | |
| `register_rpc_handler` | [07 Net](07-net.md) §3.4 `register_rpc` | payload schema 即反射描述 |
| `register_editor_operation` | EditorOperation 命名规则 `XXX.YYY.ZZZ`（承接既定设计） | |

### 3.4 GC 与生命周期协约（解决 V5，`runtime/src/gc_bridge/` [新增]）

- 宿主对象 → VM：`HostHandle`（带世代号弱引用语义，`real_backend/values.rs` [改造]）；VM GC 回收 wrapper 不影响宿主对象，宿主对象销毁后 VM 访问得到结构化错误（非 UB）。
- VM 对象 → 宿主：一律经 `VmObjectRef`（构造时向 VM 注册 GC root，`Drop` 撤销 root——宿主侧 RAII 保证不悬挂）；禁止宿主缓存裸 VM 指针（类型系统强制：raw 指针不出 `real_backend`）。
- GC 调度：`script.gc_step` ∈ Last（01 锚点表）增量步进，预算由 `VmGcBudget { max_micros_per_frame }` 资源控制；全量 GC 仅在场景切换/热替换边界允许。
- 诊断：GC 暂停时长、root 数、跨界引用计数进 runtime diagnostics（rolling store）。

### 3.5 热替换状态迁移（解决 V6，`real_backend/instance.rs` [改造] + `runtime/src/state_migration.rs` [新增]）

- `VmStateBlob` 升级为带 schema 版本的快照：`{ schema_version: u32, types: Vec<(ReflectTypePath, u32 /* type hash */)>, payload }`。
- 类型 schema 变更时走反射驱动的字段级迁移：缺省值填充 + 改名映射表（模块内声明 `renames = [["old","new"]]`）；迁移失败回滚旧模块并出诊断——与 01 §3.7 `PluginStateSnapshot` 同一回滚机制（zr_vm 插件实现该 ABI 即获得宿主级回滚）。

## 4. 模块文件树

```
zircon_reflect_derive/src/lib.rs            [新增 crate] #[derive(ZrReflect)]
zircon_runtime/src/scene/reflect/
  type_registry.rs                          [改造] 双向注册入口
  fixed/**                                  [删除] 迁移为 derive
zircon_plugins/zr_vm_language/runtime/src/
  call_site.rs                              [新增] CompiledCallSite/ScriptCallTable
  host_interface/{mod,system,bt_node,rpc,editor_op}.rs  [新增] 四通道 + capability gate
  gc_bridge/{host_handle,vm_object_ref,budget}.rs       [新增]
  state_migration.rs                        [新增] schema 版本化迁移
  real_backend/values.rs                    [改造] HostHandle 世代语义
  real_backend/instance.rs                  [改造] VmStateBlob v2 + 回滚
```

## 5. 里程碑与任务分解

### M1 反射注册表统一（跨插件收益最大单项）

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M1-T1 | zircon_reflect_derive crate（struct/enum、字段访问器、属性旗标） | 新 crate | — | `derive_round_trips_reflect_type_info` |
| M1-T2 | fixed/ 手写反射迁移为 derive 并删除 | scene/reflect/fixed | M1-T1 | 既有 ecs_reflect 测试保绿 |
| M1-T3 | register_vm_type 反向通道 + 动态组件消费 | type_registry.rs | M1-T1 | `vm_type_round_trips_as_dynamic_component` |
| M1-T4 | dense call site；字符串查找零出现断言 | call_site.rs | M1-T3 | `call_site_resolution_happens_once`（mock backend round-trip） |

### M2 接口注册四通道

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M2-T1 | VmCallbackHandle + 世代失配重解析 | host_interface/mod.rs | M1-T4 | `stale_generation_resolves_to_new_function` |
| M2-T2 | register_system/bt_node 通道 + capability gate | host_interface/{system,bt_node}.rs | 01-M2、M2-T1 | `vm_registered_system_enters_schedule_conservatively`、`vm_bt_node_executes_in_tree`（mock VM） |
| M2-T3 | register_rpc_handler/editor_operation 通道 | host_interface/{rpc,editor_op}.rs | M2-T1、07-M3 | `unauthorized_channel_returns_capability_denied` |

### M3 GC 协约

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M3-T1 | HostHandle 世代语义 | gc_bridge/host_handle.rs、values.rs | M1 | `dead_host_object_access_returns_error_not_ub` |
| M3-T2 | VmObjectRef RAII root | gc_bridge/vm_object_ref.rs | M3-T1 | `dropped_ref_unregisters_gc_root`（root 泄漏检测） |
| M3-T3 | script.gc_step 预算系统 + 诊断 | gc_bridge/budget.rs、注册路径 | 01-M1 | `gc_step_respects_frame_budget` |

### M4 backend-zr-vm 真实后端接通

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M4-T1 | `backend-zr-vm` feature 下全链路（基础库 + 回调 + GC） | real_backend/* | M2、M3 | feature 矩阵 CI（无 zr_vm 环境构建必须保持绿） |

### M5 热替换生产化

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M5-T1 | VmStateBlob v2（schema 版本 + 类型表） | instance.rs | M1 | `state_blob_round_trips_with_schema` |
| M5-T2 | 字段级迁移（缺省填充 + 改名映射）+ 回滚 | state_migration.rs | M5-T1、01-M5 | `schema_change_migrates_fields`、`migration_failure_rolls_back_old_module` |

### 当前实施状态（复核于 2026-08-02）

| 里程碑 | 状态 | 产出记录 / 边界 |
|---|---|---|
| M1 | 完成 | [`08/2026-07-14-zr-vm-m1-output-records.md`](08/2026-07-14-zr-vm-m1-output-records.md)；derive/统一注册契约、`fixed/**` 硬切及初版 dense call-site 已随共享 main 提交 `facb719f` 落库。本轮进一步完成 VM-backed 完整 JSON 闭环、clean builtin catalog、retained payload 事务、trusted manifest namespace、闭合 `List<T>`/`Map<String,T>` grammar、prepared candidate/committed epoch 身份与 catalog provenance、opaque token、自守卫 name resolution 与 unload rollback。最终 Windows 证据为插件 **21/21**、Runtime reflection **29/29**、hot reload **16/16**、`vm_type_backing` **3/3**、dynamic components **14/14**；failure validator 115/0、插件结构审计全 0，独立复核 0 Critical / 0 Important。当前 M1 output record Files 为 98/98 unique。 |
| M2 | 完成 | [`08/2026-07-13-zr-vm-m2-output-records.md`](08/2026-07-13-zr-vm-m2-output-records.md)；四通道、capability gate、世代回调与 Windows/default/真实后端验证已有记录。 |
| M3 | 完成 | [`08/2026-07-13-zr-vm-m3-output-records.md`](08/2026-07-13-zr-vm-m3-output-records.md)；runtime-neutral GC 协约与默认插件路径通过 81 + 11 项 Windows 测试。逻辑 `script.gc_step` 使用包所有权注册 ID `zr_vm_language.script.gc_step`。 |
| M4 | 完成；新增产品行为缺口保持 open | [`08/2026-07-15-zr-vm-m4-output-records.md`](08/2026-07-15-zr-vm-m4-output-records.md) 记录 `backend-zr-vm` 真实后端、GC/root、四通道与 Windows `real_backend` **15/15**。2026-08-01 复核发现 Vampire gameplay/HUD/menu/diagnostics 等价 owner 尚未迁入插件；该增量缺口由 [`08/failure-2026-08-01-zrvm-vampire-behavior-test-ownership-gap.md`](08/failure-2026-08-01-zrvm-vampire-behavior-test-ownership-gap.md) 独立保持 open，不回写成整个 M4 未实施。 |
| M5 | 完成 | [`08/2026-07-14-zr-vm-m5-output-records.md`](08/2026-07-14-zr-vm-m5-output-records.md)；v2 完整 envelope、权威类型表、统一反射 schema、缺省/改名迁移和精确回滚已通过 86 项 Script VM 回归。真实 ZrVM 已接 `saveState`/`restoreState` 完整 blob + 可选 `stateSchema` 协议；真实后端验收见 M4 产出记录。 |

- fixed 已修复：[derived-reflection-hard-cut-guard](08/fixed-2026-07-14-derived-reflection-hard-cut-guard.md)

- fixed 已修复：[derived-reflection-visibility-compilation](../zircon_runtime/render/18/fixed-2026-07-14-derived-reflection-visibility-compilation.md)

- fixed 已修复：[milestone-validation-copy-template-scope](08/fixed-2026-07-14-milestone-validation-copy-template-scope.md)

- fixed 已修复：[vm-reflection-catalog-test-support-import-drift](../zircon_editor/editor/02/fixed-2026-07-14-vm-reflection-catalog-test-support-import-drift.md)

- fixed 已修复：[zr-vm-host-modules-runtime-test-owner-drift](../zircon_runtime/runtime/04/fixed-2026-07-14-zr-vm-host-modules-runtime-test-owner-drift.md)
- fixed 已修复：[dynamic-reflection-json-projection-regression](../zircon_editor/editor/02/fixed-2026-07-14-dynamic-reflection-json-projection-regression.md)
- fixed 已修复：[vm-dynamic-property-write-structure-regression](../zircon_editor/editor/02/fixed-2026-07-14-vm-dynamic-property-write-structure-regression.md)
- 2026-07-14 回传后 owner 清单复验：插件受管测试 18/18；Runtime core-min lib-test 早期 scene filter 为 595/596，独立复核确认唯一 `JsonNumber` 失败属于本 M1 的 legacy descriptor 回归而非其他 owner。最终 VM owner 标记与声明类型双向转换修复后，受管旧 ECS 路径为 14/14、反射目录为 16/16；Runtime04 migration-journal 精确回归 1/1。Runtime13/Runtime06 定向结构审计分别为 18/14 sources，均 `missing_source_files = []`、`risks = []`。
- 2026-07-22 performance follow-up：Runtime `script/**` 96/96静态审查确认M2/M3现有功能契约仍缺steady-state artifact与真实预算闭环。PERF-MVP-444要求load/reload时发布active callback/system/package索引，445要求host实测GC deadline、next-due结构与memory policy执行，446要求prepared reflection artifact只验证一次并缓存revision snapshot，447要求bounded worker discovery与lazy bytecode。当前已直接删除callback wide slot record clone、owned systems二次clone及GC pending FIFO线性membership；其余见`08/failure-2026-07-22-runtime-script-vm-hotpath.md`，不得以既有M2/M3“完成”状态代替性能验收。
- 2026-08-11 PERF-MVP-444..447 current-source 子切片：`VmHostInterfaceRegistry` 在 manager load/reload/unload 生命周期边界原子发布 package-name/generation/capability map 与按 stage 分组的 immutable `Arc` descriptor snapshot；稳定 system/interface/package/callback 查询不再 clone wide slot records、扫描 String package name 或逐查询排序。GC 以生命周期维护的 next-due `BTreeMap` 索引提取 due bucket，并由 host `Instant` deadline 控制真实 frame/slot elapsed；backend `pause_micros` 只作遥测，低报 backend 仍被 host overrun 截止。reflection prepare 携带一次构建/验证的 `Arc<TypeRegistry>` 与 registrations artifact，generation commit 短发布 catalog state/snapshot/epoch，重复 current snapshot 和同代无变化操作复用 Arc，最终 World sync 保留一次最新 live-payload 校验。discovery 第一阶段仅解析受 root/symlink/depth/entry/path/count/bytes/time budgets 约束的 manifest；manager 通过所属 Core 的 Runtime11 bounded I/O lane 提交可取消/deadline/shutdown request，同 pool 同步等待 fail-fast。选中 load/reload 才按 package containment 与单文件/缓存总量预算读取 bytecode；path+metadata fingerprint 的 `OnceLock` 单飞缓存按变更替换，瞬时失败撤销缓存。TDD 结构门 `tools.tests.test_plugins08_vm_active_interface_snapshot` 8/8、Rust 1.94.1 scoped rustfmt 与 diff-check GREEN；Rust behavior tests 已加入但未获得 managed Cargo job。memory policy 仍缺 backend per-slot bytes（不能用 root count 或全进程 RSS 冒充），watcher path-generation、changed-type World delta/CAS、真实 backend granularity 与 1/100/10k 产品测量仍 open，因此 hotpath failure 不生成 fixed return。设计以 Bevy graph-changed build/published executable 复用为主、Godot registration-time map/read-time lookup 交叉校验；Zircon 保留 generation rollback，不引入 backend 特判。
- open / 待修复：[Vampire real-VM 行为测试 owner 缺口](08/failure-2026-08-01-zrvm-vampire-behavior-test-ownership-gap.md)；Runtime10 仍编译 10 个永久 ignored gameplay/HUD/menu/diagnostics tests 与 508 行 support owner，但 Plugins08 当前没有被声明为“已迁移”的等价 Vampire 覆盖。先在 real backend 建立可执行验收，再协同 Runtime10/Runtime15 删除旧测试与结构锚。

## 6. 验收命令

```powershell
./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -ManifestPath zircon_plugins/Cargo.toml -Package zircon_plugin_zr_vm_language_runtime -LibTests
# 真实后端（需 ZR_VM_RUST_BINDING_LIB_DIR）
./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -ManifestPath zircon_plugins/Cargo.toml -Package zircon_plugin_zr_vm_language_runtime -Features backend-zr-vm -LibTests -TestFilter real_backend
```

## 7. 风险

- derive 宏是跨插件收益最大的单项投资，也是 10-E1（反射默认 drawer）与 07-M4（replication schema）的硬前置；M1-T1 排最高优先。
- VM 系统 dynamic access 使调度器对其保守串行；文档明确 VM 系统性能定位（gameplay 逻辑而非引擎 hot path）。
- `backend-zr-vm` 的 CI 依赖外部构建产物：CI 用预编译缓存或专用 runner，default 构建绝不依赖。
- 2026-07-22 World dynamic component性能同步：retained payload的registrations×entities扫描已改为单遍type index；prepare/sync仍复制registry并全payload验证，单字段VM JSON写仍clone整component且O(F²) schema probes。Plugins08与Runtime13按PERF-MVP-446/443发布prepared immutable registry generation、World type delta和dense field validator，stable field access禁止整JSON clone；局部止损见PERF-MVP-461及performance dynamic-components证据。
- proc-macro crate 进根工作区会增加全量构建时间；保持宏实现最小化（syn features 收紧）。

## 8. 附录 · dev 参考源码对位

实现各任务前**必须先读对应参考实现**，反射模型与 GC 边界对照真实代码核对，禁止凭空实现：

| 设计点 | 参考源码（已核验存在） | 看什么 |
|--------|----------------------|--------|
| 编译期反射注入（TypeMeta/FieldAccessor/MethodAccessor） | `dev/Piccolo/engine/source/runtime/core/meta/` + 生成器 `dev/Piccolo/engine/source/meta_parser/` | 字段/方法访问器的生成形态与注册流——derive 宏（M1-T1）输出结构的判例 |
| Rust 反射 derive 工程实践 | `dev/bevy/crates/bevy_reflect/`（含 derive 子 crate） | derive 宏的属性解析、容器类型（Vec/Map/Option）反射、TypeRegistry 形态——我们复用自有 ReflectTypeInfo，但宏实现技法以此为准 |
| 跨语言注册/函数表/世代句柄 | `dev/godot/core/extension/gdextension_interface.cpp`、`gdextension.cpp` | class/method 注册流、object instance binding 生命周期、版本化函数表 |
