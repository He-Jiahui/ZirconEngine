# 组件/ECS 生命周期 · EC-01：despawn 回调新增的 SparseSet 组件不在旧清理列表内

> 取证说明：UE 本机路径与 SHA256 固定参考树；本地记录不随 PR 发布，不冒充 GitHub 可读证据。§1–6 保留首次源码调查；§7 记录后续 ZE 原生回归与修复阶段。UE 测试仍未执行。

**原问题定义在 §1–6；独立 Draft #167 已发布，同一 PR 的回归推进见 §7。先真实 RED，再评估生产修复。**

- 本地审计 ZE `cb495bb1c952a24c86f4513b8099bc4942c3a87d`；ZE 链接使用 lead 指定、已核对引擎目录零差异的发布基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`，避免未发布提交的失效链接。
- 本地 UE [Build.version:2–9](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L2-L9>) 是 **6.0.0，BranchName UE5**；SHA256 `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6`。不是猜测的 UE5.7。
- **首次调查阶段的结论是源码级 storage/lifetime 缺陷**：回调前冻结的 component IDs 被当成回调后的完整 sparse 清理列表。此时尚未运行 Cargo 或 ZE/UE 测试，下面终态为当时源码推导；后续 ZE 实测与保留的验收边界见 §7。
- 不把 UE UObject 与 Rust ECS 布局不同、UE 全局禁止 processing 中结构修改而 ZE 允许回调写 World，本身当成缺陷。真正缺陷是：**ZE 已返回 Ok 的组件插入，其值在实体注销后仍留在 owned sparse storage，公开组件计数与活实体数脱节。**

## 1. 和 Unreal 哪里不一致：精确位置与可达调用链

### ZE 的公开写入与销毁路径

| 证据 | 符号与事实 |
|---|---|
| [Z1 · hierarchy:10–63](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/hierarchy.rs#L10-L63>) | **缺陷 owner `World::remove_entity`**：第34行先取 `component_ids`，第36–47行同步派发 Remove/Despawn；第61–62行仍用这一旧列表删除 sparse rows。 |
| [Z2 · typed API:135–143,186–265](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/typed_api.rs#L135-L265>) | `World::insert<T>` 对存在实体接纳普通自定义组件。SparseSet 分支先 `component_storage.insert_at_tick`，再更新当前 archetype signature。没有“该实体正在 remove_entity”准入检查。 |
| [Z3 · validators:55–101](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/typed_api/fixed_components.rs#L55-L101>)；[同文件:358–373](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/typed_api/fixed_components.rs#L358-L373>) | 泛型禁写仅针对特定 authored/derived 类型；一个普通 `LateSparse(u32)` 不命中，固定组件验证也返回 Ok。 |
| [Z4 · lifecycle registration/dispatch:9–20,78–106](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/observers.rs#L9-L106>)；[bucket:66–77](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/ecs/observer/entry.rs#L66-L77>) | `observe_component_lifecycle` → `trigger_component_lifecycle` → bucket dispatch，同步调用业务 observer。 |
| [Z5 · safe producer:454–505](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/retained_memory.rs#L454-L505>) | `RetainedLifecycleObserverV1::new` 是安全构造器；callback 参数确实是 `&mut World`。本例可用显式 `()` state、纯 effects 和不捕获闭包实现，不需要滥用 unsafe preserving adapter。 |
| [Z6 · target guard:31–39](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/ecs/schedule_execution.rs#L31-L39>) | `ScheduleExecution::target_change` 只验证 World owner 和 schedule plan 身份。同一个 World 中新增普通 component 不替换它们，所以 Z1 的回调后检查不会取消本例。 |
| [Z7 · sparse cleanup:512–539](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/ecs/storage/component_storage/store.rs#L512-L539>) | `remove_entity_components` 只遍历传入 IDs，不扫描/补删新类型；`len_for_component` 直接返回 storage 的长度。 |
| [Z8 · sparse ownership:60–78,252–279](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/ecs/storage/component_storage/sparse.rs#L60-L279>) | `SparseComponentStorage` 将新值追加到 `entries`；`len` 为 `entries.len()`。generation 验证能拒绝 stale lookup，不会替代删除拥有值。 |
| [Z9 · identity:77–79,114–126](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/identity.rs#L77-L126>)；[Z1:64–87](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/hierarchy.rs#L64-L87>) | archetype 行和 stable/internal identity 随后被移除；这一步只销毁 table row，不会清空未列入 Z7 的 sparse row。 |
| [Z10 · public count:65–74](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/typed_api.rs#L65-L74>) | `component_count_for_id` 对 SparseSet 直接使用 Z7 的数量，故幽灵 row 不仅是不可观察的内部容量。 |

真实可达入口不止测试：脚本 host [导出 `despawn`:368–373](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/gameplay_host.rs#L368-L373>) → [`gameplay_host::lifecycle::despawn`:178–188](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/gameplay_host/lifecycle.rs#L178-L188>) → `LevelSystem::with_world_mut` → Z1 `World::remove_entity` → Z4 callback → Z2 `insert<LateSparse>` → 返回 Z1 → Z7 旧 IDs 清理 → Z9 注销身份。自定义 observer 的安装是本例显式前置条件，不声称当前每个产品项目默认装了它。

这里没有把 Editor 的 subtree detach/undo 路径混同为该入口：[detached subtree:344–398](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs#L344-L398>) 先提取数据/注销，再派发 staged callbacks，是不同路径。

### UE 对应销毁/observer 链

下列 U 编号的本地整文件 SHA256 见文末。

1. [U1 · `FMassEntityManager::DestroyEntity`:907–925](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassEntityManager.cpp#L907-L925>)：先 `OnPreEntityDestroyed`，再 `FMassArchetypeData::RemoveEntity`，最后释放身份。
2. [U2 · `OnPreEntityDestroyed` / `OnCompositionChanged`:316–319,396–417](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassObserverManager.cpp#L316-L417>) → [U2 · `HandleElementsImpl`:467–475](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassObserverManager.cpp#L467-L475>)；observer 执行时持有 `FScopedProcessing`。
3. [U1 · `AddFragmentToEntity`:1088–1096](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassEntityManager.cpp#L1088-L1096>) 和 [U1 · `AddSparseElementToEntity`:1453–1463](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassEntityManager.cpp#L1453-L1463>) 都经 [U1 的 `CHECK_SYNC_API`:22–23](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassEntityManager.cpp#L22-L23>)，不允许在该 processing 范围同步改 composition；需要后续结构操作的 observer 使用 deferred command。
4. 独立于 guard，[U3 · `FMassArchetypeData::RemoveEntity`:553–578](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassArchetypeData.cpp#L553-L578>) 在实际删除时取当前 chunk 的 per-entity sparse element bitset，再调用 [U4 · `FSparseElementsStorage::RemoveEntity`:364–372](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassSparseElementsStorage.cpp#L364-L372>)；[U4 · `FTypePool::Remove`:88–100](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassSparseElementsStorage.cpp#L88-L100>) 销毁实例并减少计数。不是 ZE 的“回调前列表沿用至回调后”。
5. batch 入口另有 [U1 · `BatchDestroyEntityChunks`:970–981](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassEntityManager.cpp#L970-L981>)：pre-destroy observers 后验证 collection 未因 composition mutation 失效，再删除和释放身份。

**UE guard 的边界也要如实保留：** [U8 · `testableCheckf*`:60–97](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Mass/MassCore/Public/Mass/TestableEnsures.h#L60-L97>) 在 Mass 测试范围输出错误并提前返回，非测试范围走 `checkf`，Shipping/Test 宏也不同。不能将这段源码说成所有配置均提供 Rust 式可恢复错误。对本问题最可直接借鉴的是“实际销毁按当前拥有的 sparse 集合释放”和有效的结构变更边界，而非复制宏。

## 2. 实际影响：安全 API 即可构造的最小反例

准备 `Health` 普通 Table component 和仅含内联数值的 `LateSparse`，后者声明 `Component::STORAGE_TYPE = SparseSet`，两者按现有 retained-owner 契约实现计量。

1. 在同一个 World 先注册 `LateSparse` 类型，得到 component ID `late_id`，但不把它插入目标实体。
2. spawn 目标实体，具有 `Health(1)`。记录当前 internal identity；让所有其他设置保持默认。
3. 为 `Health` 的 **Remove** 注册 safe `RetainedLifecycleObserverV1::new`：state 为 `()`；effects 为纯 `Ok(())`；callback 只做 `world.insert(event.entity(), LateSparse(7))`，该组件不属于固定 NodeRecord，且该 observer 在 Create/Add 时不执行。另以 Despawn 回调重复此单独用例。
4. 调用公开 `world.remove_entity(entity)`。

逐源码推导的终态：

| 可观察量 | 应满足的销毁守恒 | 当前路径推导 |
|---|---|---|
| callback 内插入 | 要么在写入前有明确拒绝，要么被成功接纳后参与最终清理 | `Ok(None)`，新 row 和 signature 均已安装 |
| `contains_entity(entity)` / `get::<LateSparse>(entity)` | false / None | false / None，身份确实已注销 |
| `component_count_for_id(late_id)`（没有其他该类型实体） | **0** | **1**，row 不在旧 `component_ids` 中 |
| late component 的拥有值 | 本次实体删除完成后不再被该实体保留 | 仍在 `SparseComponentStorage.entries`；直到 storage 清空/重建或 World 释放等后续操作才可能析构 |

这并非只凭回调会递归就猜测风险：新增普通 sparse 组件不会再次触发 `Health` 的 Remove；不替换 World/plan；不使用并发、不溢出句柄，也不触及 unsafe 数据访问。现有 generation 检查会挡住旧实体的后续 typed lookup，故**不声称 use-after-free、错误命中新 generation、崩溃或已测得性能退化**。可以证实的影响是残留 owned payload、延迟 Drop 与公开数量不守恒；Drop 次数仍应由后续实际测试观测。

## 3. 根因及 UE 实际解决

ZE 用同一 `component_ids` 承担两件不同的事：

- **通知快照**：本轮为哪些原有组件发 Remove/Despawn；
- **最终拥有集合**：当前实体到底还有哪些 sparse rows 要释放。

在 callback 可以合法修改 World 的模型下，两者不再等价。Z2 会同步更新当前 signature，但 Z1/Z7 没有再读取它。Z6 的 guard 专门防止 owner/schedule 替换，不能证明 component 集合未变。

UE 的实际路径同时有 processing 期间的同步结构 mutation 检查、batch collection currentness 检查和销毁时取当前 per-entity sparse bitset 的释放逻辑。这里不声称“UE 只靠 Arc/GC 自动解决”，也不声称当前 UE 所有配置的 assert 都是硬安全边界。

与历史 [ECS review:285–296](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event.md#L285-L296>) 的泛 observer recursion/budget/panic gap 有关联，但本条不以“应新建 observer 架构”为结论；它独立指向已有 `remove_entity` 中的旧清理集合。

## 4. 不发明新架构的 Rust 做法

优先利用 ZE 已有的 **当前 archetype signature + 明确 owning storage**：

- 保留原通知顺序以及现有 owner/schedule guards；在真正释放 sparse rows 前，重新从仍有效的同一实体取得当前 signature/IDs，而不是复用回调前的通知快照。
- 若同一实体已经被 callback 注销或 generation 改变，应走明确的已有实体错误/取消逻辑，不能拿旧 internal handle 继续清理。该 identity 再验证是修复评估时必须核对的边界，不在本报告另计一条缺陷。
- 只删除该当前 signature 对应的 rows，保留现有“不扫描所有已注册 storage”的设计，不引入新的 ECS、GC、全局生命周期服务或后台清理任务。
- 是否要进一步禁止“正在销毁目标”的直接结构变更，可在后续 Draft 评审中决定；若选择该政策，应使用一个 owner-scoped 私有准入/RAII guard 和现成 deferred commands，而不是把所有 ZE observer 统一改成 UE processing API。它不是本轮已实施方案。

## 5. ABI / ECS 差异

- ZE 是 Rust Table/SparseSet，`Component` 默认 Table，自定义类型可选 SparseSet；证据见 [Component trait:3–8](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/ecs/component/marker.rs#L3-L8>)。UE Mass sparse element 是 chunk/per-entity bitset 加 type pool，不应照搬数据布局。
- ZE `EntityId` 映射到内部 generation；UE `FMassEntityHandle` 用 Index/SerialNumber。这些不同不是缺陷。残留 row 的拥有关系不能靠“旧 handle 已失效”替代清理。
- UE UObject ActorComponent 另有 registered/initialized/destroying 状态；不将 `UActorComponent::DestroyComponent` 等同为 Mass/ZE ECS component remove。此次实质对照是 Mass 销毁与 sparse storage。
- 保留现有 Rust `SceneResult`、callback producer 和 DTO 即可修改清理 owner，不需要改变动态 C ABI、interface table、component ID 编码、retained-memory 版本或序列化格式。若以后选择新增 typed admission error，应先审核错误映射范围，而非在本草案顺带改 ABI。
- ZE 的 observer 可同步写入 World 是既有设计。因此不能把 UE `ChangingCompositionSync` 的全局禁止政策不加区分地搬过来；测试应先锁定**实体删除后 row 计数/拥有值守恒**这一共同契约。

## 6. ZE 具体改法：仅草案与等价测试表

**不得先改引擎验证猜测。** 原问题定义阶段只建议独立 Draft，未改测试/生产；后续已建立 #167，§7 如实区分准备、实际执行与修复阶段。

候选最小修复落在 Z1：把回调前 IDs 明确当 notification snapshot；完成最后一个 callback 的 owner/plan 检查后，重新验证目标 identity，从**当前 signature**得到 cleanup IDs，再传给现成 Z7。现有 table row、observer、dynamic component、camera 和 identity 清理顺序先保持不变。late component 的 Remove/Despawn 通知是否也应发出是待评审契约，不能通过不受控“循环发回调直到没有组件”掩盖；若要求完整通知，应先决定准入/延期政策再修。

现有 [despawn 结构守卫:20–34](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/tests/ecs_identity_storage/despawn_contract.rs#L20-L34>) 只查字符串，未在回调期间改 composition；它不是这条行为回归的替代品。仍应保留其成本意图：依据当前 signature，非全 storage 扫描。

### 等价测试表（计划，尚未移植/运行）

| 类别 / 本地 UE 测试名与位置 | 输入 | UE 原期望 / ZE 对应期望 | ZE 既有测试落点 |
|---|---|---|---|
| **现成销毁通知用例迁移**：U5 `System.Mass.Observer.Fragment.SingleEntitySingleArchetypeDestroy`，[474–488](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MassEntityTestSuite/Private/MassObserverTest.cpp#L474-L488>)；base fixture [380–438](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MassEntityTestSuite/Private/MassObserverTest.cpp#L380-L438>) | 两类 archetype 各3实体；对 int 实体 `[1]` 加 Float fragment，flush；观察 Remove 后 deferred destroy | Add 时无 Remove；destroy 只通知目标一次，observer 仍可读 fragment，其派生 deferred command 最终被 flush。ZE 对应保持既有 Remove→Despawn 顺序，不捏造 UE 的两个相同事件 | [现有 lifecycle order:79–132](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/tests/ecs_observers_messages.rs#L79-L132>)；追加行为用例复用此模块的 fixture |
| **现成 sparse 清理用例迁移**：U6 `System.Mass.Stress.Sparse.Storage.OnEntitiesDestroyed`，[61–102](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MassEntityTestSuite/Private/StressTests/MassStressSparseElementsTest.cpp#L61-L102>) | 10实体全加 SparseInt；删除 `[2,5,7]` | 被删实体的 sparse view 失效，其余7实体的值仍存在；ZE 内部 storage/公开 count 均为7，不能只断言 stable handle 失效 | [identity/despawn 既有行为:98–159](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/tests/ecs_identity_storage.rs#L98-L159>)；以及 [component storage 行测试](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/tests/ecs_identity_storage/component_storage.rs#L1-L56>) |
| **现成计数守恒用例迁移**：U7 `System.Mass.Stress.Commands.BatchDestroy.WithSparseElements`，[570–616](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MassEntityTestSuite/Private/StressTests/MassStressCommandsTest.cpp#L570-L616>) | 20实体，各有 SparseInt(value=i×10) 和 sparse tag；全部销毁 | 所有 handle 无效，SparseInt storage count=0。ZE 不需要复制 UE chunk 容量，只保留相同实体/值/计数契约 | 上述 identity 模块；同时复用 [lifecycle order](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/tests/ecs_observers_messages.rs#L79-L132>) 的 observer 验证方式 |
| **源码推导、待复核，非 UE 原测迁移**：本报告第2段 witness | 原来缺少 LateSparse 的实体；Health Remove observer 通过 safe producer 插入 LateSparse；独立 Despawn 变体；均不替换 World/plan | `remove_entity` 完成后 count=0、实体不存在、late payload 不再由该实体保留；若准入改为拒绝，必须在插入前可观察到 typed rejection，且同样 count=0。现状推导 count=1 | [observer 模块:1275–1331](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/tests/ecs_observers_messages.rs#L1275-L1331>) 附近已有“同 owner/plan 内回调”控制；新用例不得替换成 World replacement |
| **源码推导、待复核，非 UE 原测迁移**：Drop/控制 | 用按现有 retained 契约计量的 Drop probe 替换 inline LateSparse；另放一个 survivor 同类型实例；删除目标后再释放 World | 删除目标仅释放它所拥有的实例；survivor 值/计数保留；World 最后释放不二次 Drop 目标。先定义 producer/计量，不能为复现而违反 unsafe retention 契约 | 同 observer 模块和现有 storage 测试模块；新行为测试，不用 source-string assertion 冒充 |

### 不冒充等价迁移的 UE 测试反证

- U5 [`FForbidModifyOnDestroy`:242–294](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MassEntityTestSuite/Private/MassObserverTest.cpp#L242-L294>) 虽然名字最像本问题，**真正两条 composition mutation 在260–261行被注释**，`bIsProcessing` 也未形成该预期断言；剩余为观察和 dummy deferred command。不能将其说成“UE 已有销毁回调新增 sparse 的回归”。
- U5 [`ChangingCompositionSync`:632–678](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MassEntityTestSuite/Private/MassObserverTest.cpp#L632-L678>) 真正检查 Add observer 同步加 fragment 被拒绝；[Deferred:680–708](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MassEntityTestSuite/Private/MassObserverTest.cpp#L680-L708>) 检查延期加 fragment 成功。这是 processing policy 的有效控制，但 UE/ZE callback 准入政策不同，**不能不加说明地作为本条 sparse 析构测试的等价移植**。
- U6 [名为 `DestructorValidation`:472–521](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MassEntityTestSuite/Private/StressTests/MassStressSparseElementsTest.cpp#L472-L521>) 实际只断言 view 不存在，没有 Drop/析构次数计数；故本报告的 Drop probe 是新设计的源码推导测试，不冒称迁入 UE 已有 destructor counter。
- 搜索范围：本地 `MassEntityTestSuite/Private`（包含 Coverage 和 StressTests）、MassEntity manager/observer/archetype/sparse storage，以及 ZE scene/world/ecs 和 scene tests。未找到与“despawn observer 同步新增另一种 sparse component 后必须释放”完全相同的有效 UE 单测。其最终 storage 契约来自 U3/U4 和上述清理测试；须在 Draft 后复核执行。

### 本地 UE 源码 SHA256

| ID / 本地文件 | SHA256 |
|---|---|
| [U1 · MassEntityManager](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassEntityManager.cpp#L907-L925>) | `3aba75170678e56797f9a76b1d3c0e3d80e0d1d48ce9a9f9719ec408372b76a0` |
| [U2 · MassObserverManager](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassObserverManager.cpp#L467-L475>) | `ae01abc38baa844916ec2293e440e7e7b9089ef55199ee4b9949906a22db9312` |
| [U3 · MassArchetypeData](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassArchetypeData.cpp#L553-L578>) | `5ddb9823002f2a89556c1a5c47ffb266e99dc3caae9d564c1af215bf8de8e0fd` |
| [U4 · MassSparseElementsStorage](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassSparseElementsStorage.cpp#L364-L372>) | `50fc0de45ad476c3d7e03585180891d409c3ae1c7ea86743a0e724140527450f` |
| [U5 · MassObserverTest](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MassEntityTestSuite/Private/MassObserverTest.cpp#L474-L488>) | `72bb43bdf97e897e050b9c4795f4ae2e48c849f6c2b965a9f3b458b7841a4f93` |
| [U6 · MassStressSparseElementsTest](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MassEntityTestSuite/Private/StressTests/MassStressSparseElementsTest.cpp#L61-L102>) | `cd3dd522c478c2ade273dbad7119608e57043c755755309ef07d897d2135e0a9` |
| [U7 · MassStressCommandsTest](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MassEntityTestSuite/Private/StressTests/MassStressCommandsTest.cpp#L570-L616>) | `592ee44969c485b9f1fedc856bb8309cf24ff38c819d485e92b6c57a72951661` |
| [U8 · TestableEnsures](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Mass/MassCore/Public/Mass/TestableEnsures.h#L60-L97>) | `38e6fba2906b7e8c7594020c64cd4c0ca35e211ba124d179de7603a899e0d0de` |

### 去重

只计 **EC-01 一条**。本次已读的 [PR snapshot](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/github-pr-snapshot.json>) 中 [PR131](<https://github.com/He-Jiahui/ZirconEngine/pull/131>) 是 ECS 调度阶段等价测试；当前 [unreal stage contracts](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/tests/ecs_schedule/unreal_stage_contract.rs#L1-L23>) 也明确在测 stage admission/order，不是销毁 sparse cleanup。既有 owner/plan replacement 测试防护有效，本例保持同 owner/plan，所以不重报这批已处理问题。与 PR145/135/144 均无共同故障 owner。领域覆盖另见 [组件覆盖记录](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/research/components-coverage.md>)。


## 7. 同一 Draft 的回归推进

**本问题实际闭环：原5案 2 PASS / 3 FAIL → 单 owner 当前集合/原代际绑定窄修 → 原5同字节 + 新3代际控制共 8 PASS / 0 FAIL / 0 ignored。扩展 observer 31/2、identity 37/1，3条未修改旧源码守卫仍失败，不能称整组 GREEN。保持 Draft，不合并。**

在 [原 observer harness](../../../zircon_runtime/src/scene/tests/ecs_observers_messages.rs#L1519-L1520) 仅加薄挂载，原1,517行测试和 imports 不变；[190行子模块](../../../zircon_runtime/src/scene/tests/ecs_late_sparse.rs#L1-L190) 放全部5案，不建立新 runner 或 ECS 架构。各 observer 使用安全 retained constructor、显式 `()` state 与非捕获回调；payload 走现有 retained-owner 计量，Drop 计数器在真实 component 内。

| case | 必须观察的行为 oracle |
|---|---|
| `cpu167_despawn_cleans_up_sparse_added_by_remove_observer` | 目标原无 LateSparse；Health Remove callback 的插入确为 `Ok(None)`，立即同实体读回后记录实际 kind/target；删除返回后实体已不存在且 typed get 为 None，**公开 sparse count 必须0**。 |
| `cpu167_despawn_cleans_up_sparse_added_by_despawn_observer` | 独立 World，仅 Despawn observer，重复相同插入/回执/计数 oracle，不与 Remove 两次写混在一案。 |
| `cpu167_despawn_preserves_survivor_with_preexisting_sparse` | 原有同类型两行，删除目标后仍保留 survivor 的身份和值；count **2→1→0**，最后才删除 survivor。 |
| `cpu167_despawn_cleans_up_sparse_replaced_by_remove_observer` | 原有行被 callback 替换，必须 `Ok(Some(old))` 且读回新值；删除只清目标，不清 survivor，count1。 |
| `cpu167_despawn_drops_callback_added_sparse_once` | 接纳真实 owning payload；删除返回时 target/survivor Drop **1/0**，World 最终释放后 **1/1**；不能仅让 stale get 返回 None 来伪造清理。 |

外部同步回执只在 callback 内插入成功且读回之后记录；漏调、错目标或提前拒绝都不能通过。没有手动清理、storage loop 复制、私有谓词替代或放宽为“插入成功/失败都可”。这些新测试来自 ZE accepted-write 契约和 UE Mass 当前拥有集合释放语义，**不是 §6 邻近 UE 原测试的逐字移植**；UE 原测试未执行。

首次实际执行采用现成 `zircon_runtime --lib` CPU harness，关闭 defaults，明确 `core-min,native-host`，过滤 `scene::tests::ecs_observers_messages::late_sparse_despawn::cpu167_`。它不启用 Graphics/UI/Script，也没有隐藏其他已准入 test modules；编译/依赖前置错误不算 RED。本次在原实现真实失败之后才评估 cleanup 修改，实际后续结果与失败守卫保留如下。


### 原实现实际 RED 与单 owner 候选（2026-11-18）

原 `zircon_runtime` CPU libtest 实际编译并执行所选5案：**2 PASS / 3 FAIL / 0 ignored / 7775 filtered**。Remove、Despawn 两案均已成功接纳 `Ok(None)` 插入、同实体读回且记录准确回执；实体消失且 stale get 为 None 后，公开 count 仍为 **1（期望0）**，在测试63行失败。Drop 案也接纳拥有值，但在177行观察 target Drop **0（期望1）**；World 析构之后的终态未在这次失败处到达，不能据源码补写为实测。预先有行与 callback 替换的 survivor 两案通过。命令回执 SHA256：`29090e6d632a67ae7478dd2887ea66ec805fbe93b0406d433704441743d85cf3`；相关18份输入运行前后一致，回执不代表封闭进程树或产品验收。

独立评估要求候选在 [同一清理 owner](../../../zircon_runtime/src/scene/world/hierarchy.rs#L10-L101) 保留通知快照，读取**回调后的当前 signature**作为拥有值清理列表，同时在 child orphan 之前绑定原 `InternalEntity`。每次原 World/plan guard 通过后立即核对同 stable ID 仍是原 index/generation；若消失或被替代，使用已有 `MissingEntity`，不得把新一代当原删除目标。原通知、World/plan 错误优先级、table/archetype/hierarchy/observer/camera/identity 后续顺序保留，不扫描全部 storage，不循环通知 late component，不新增准入架构、公开 API/错误变体、ABI 或 schema。

另在 [既有 identity harness](../../../zircon_runtime/src/scene/tests/ecs_identity_storage.rs#L854-L855) 薄挂载 [3条代际控制](../../../zircon_runtime/src/scene/tests/ecs_despawn_identity.rs#L183-L230)：通过安全显式 retained state/真实恢复预算、one-shot observer 与公开 `remove_entity` / `insert_node_record`，分别在 Remove、child Hierarchy Replace、Despawn 回调恢复**同 stable ID / 不同 generation**；要求旧目标 typed `MissingEntity`，替代实体 record/table/sparse/count 完整，且不继续下一个 outer bucket。这3案已在候选上实际全部通过，但未在原实现上执行，不能冒称它们也取得过 baseline RED。它们没有改动原5条 oracle 或原旧测试正文。

后续确实执行了同 `core-min,native-host` CPU profile 的 observer、identity 两个既有前缀，并用 `cpu167_` 聚焦复核8案；结果不伪装全绿，见下表。未执行的其他 libtests 只是编译准入，不声称所有 callback、UE/其他 OS、产品或性能通过。


### 实际 scoped GREEN 与保留的旧守卫失败

| 真正执行的过滤器（同一 CPU feature/profile） | 实际结果 | command receipt SHA256 |
|---|---|---|
| `scene::tests::ecs_observers_messages::late_sparse_despawn::cpu167_`（修复前） | **2 PASS / 3 FAIL / 0 ignored / 7775 filtered** | `29090e6d632a67ae7478dd2887ea66ec805fbe93b0406d433704441743d85cf3` |
| `scene::tests::ecs_observers_messages::`（修复后扩展） | **31 PASS / 2 FAIL / 0 ignored / 7750 filtered**，原5全部通过；不能称整组 GREEN | `8fea9033968efebace77d797181df42ad4319702a95d2544094ed4cba381b7e2` |
| `scene::tests::ecs_identity_storage::`（修复后扩展） | **37 PASS / 1 FAIL / 0 ignored / 7745 filtered**，3新代际控制及两旧 despawn 成本守卫通过；不能称整组 GREEN | `ecb8287a3f55a3f79db6343ff5bf7da6e92cad0b72412e7dd7cc5a134b629a26` |
| `cpu167_`（修复后聚焦确认，无 skip） | **8 PASS / 0 FAIL / 0 ignored / 7775 filtered**；原5 + 新3，无其他匹配 | `f64580f9ade25843df92dd35af03a21619f800e4218f5956be92f860c9c4932f` |

四次 payload 都是既有 `zircon_runtime --no-default-features --features core-min,native-host --lib`，`--locked --offline --jobs 1 -- --test-threads 1`；只换表中执行 filter。Windows x64/MSVC、Rust1.94.1，普通 test profile debug=0/incremental=false。使用规范本地 wrapper 显式绑定 owned工作树、Python3.12.14 与 F盘既有 cache/target，所有编译/cache/temp 在 drive-root cargo-targets 下；不启用 Graphics/UI/Script，不隐藏其他准入 libtests，不跑 UE/产品。前18/后20个选定源/夹具/manifest/lock 输入运行前后分别一致；不是整个进程/源码闭包封印。两组重复包含聚焦案，不能相加成总通过数。

扩展组的3项失败完整保留，未改 guard 或 `--skip`：

1. [batch writer 守卫](../../../zircon_runtime/src/scene/tests/ecs_observers_messages.rs#L471-L485) 要求一个 LF-only 多行字符串；实际 Windows CRLF 的 [queue 输入](../../../zircon_runtime/src/scene/ecs/events/queue.rs) 不匹配。该输入与 `e893` 规范化字节相同，基线 LF Git blob 命中字符串，CRLF 输入不命中。
2. [immutable bucket 守卫](../../../zircon_runtime/src/scene/tests/ecs_observers_messages.rs#L434-L463) 仍要求旧 `Fn(&mut World, &ComponentLifecycleEvent) + Send + Sync` 文本；它消费的6份 observer 源与 `e893` 规范化一致，基线和当前 retained callback 源都无该字面量。
3. [locator 守卫](../../../zircon_runtime/src/scene/tests/ecs_identity_storage/component_storage.rs#L137-L155) 先在已委托 locator 叶模块的入口文件要求过时字段字面量；该旧字面量在当前实现中并不存在。原测试139行已有 `CR-R02-runtime_ecs_identity_storage-0001` 注释；测试及 sparse 输入均与 `e893` 规范化一致。**此旧案后续 slot-generation 行为断言未到达，不能补写成 PASS。** 新3案的同 ID/新代际行为是真正独立执行通过的。

这是直接输入相同及纯字符串谓词的源码反证，不是实际重跑过宽基线的声明。旧 parent 正文、3项 guard 和它们消费的源码未因本修复改动；不在 #167 顺带修复测试基础设施，不把窄 GREEN 升格为整个 ECS/Runtime 通过。

实际保持了 accepted late-write 的释放守恒和原目标绑定；Drop 案在删除返回时 **target/survivor 1/0**，World 释放后 **1/1** 真正通过；3代际案验证替代实体的 record/table/sparse/count 保留及不继续后续 outer bucket。现有 World/plan cancellation 与通知顺序控制也通过。其他 OS、所有 callback 组合/异常、UE原测试、App/editor产品、性能以及上述3个旧守卫的完整验收仍不在本 scoped GREEN 内。所有PR保持Draft，未合并。
