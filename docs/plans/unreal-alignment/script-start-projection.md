# 脚本执行：其他实体的绑定数据写入会重发既有实例的 `onStart`

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

**结论：独立 Draft 候选，源码证据高置信，未实跑。既有 Runtime216/Runtime07 P1-10 源码复核，非首次发现。** [Runtime13 旧 failure 的完成契约](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/zircon_runtime/runtime/13/failure-2026-07-22-runtime-script-binding-hotpath.md#L100-L116) 明确要求 unaffected binding 跨 generation 重建保留 activation；[Runtime216 的 P1-10](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/216-runtime-script-plugin-execution-vm-binding-hot-reload-native-isolation-debug-product-integration-current-working-tree-review.md#L151-L164) 为 Partial，未重新认定已验收。此次仅取“同一 World 中 A 写入，B 未变却重复启动”这一可独立证伪/修复的子项；不是 Fixed-before-Start、首错使后续绑定饿死、热重载策略、跨 World cache 替换或游戏框架重写。

审阅 ZE 本地 HEAD `cb495bb1c952a24c86f4513b8099bc4942c3a87d`；和 Draft 的固定 blob 基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37` 在 `zircon_runtime`、`zircon_runtime_interface`、`zircon_editor`、`zircon_plugins`、`zircon_app` 比较无差异。UE 唯一来源为本机 [Build.version:1–10](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L10>) 的 `6.0.0 / BranchName UE5`；UE 下列定位均为该检出，不推断其他版本。SHA-256 见附表。

## 1. 不一致：从真实入口到状态丢失

- **ZE 数据入口。** [scene_asset.rs:421–433](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/project_io/scene_asset.rs#L421-L433) 将场景资产 `script_bindings` 接入 World 的 `script.bindings` 动态组件。运行时的 `damage_entity` 不是测试桩：它由 [gameplay_host.rs:353–367](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/gameplay_host.rs#L353-L367) 导出；[combat.rs:54–83](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/gameplay_host/combat.rs#L54-L83) 的非致死分支写回 A 的 `script.bindings` JSON；[dynamic_components.rs:224–251](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/dynamic_components.rs#L224-L251) 只在值实际变化时推进该组件**类型级**代次。
- **ZE 执行入口。** linked ZrVM [plugin.rs:176–189](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/zr_vm_language/runtime/src/plugin.rs#L176-L189) 注册 `ScriptSceneRuntimeSystem::update/fixed_update`；动态会话在 [construction.rs:130–136](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/construction.rs#L130-L136) 装配 [script_systems.rs:10–56](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/script_systems.rs#L10-L56) 的同一回调（只补 linked 未拥有的阶段），[schedule_runner.rs:152–175](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/ecs/schedule_runner.rs#L152-L175) 调度。`run → tick_script_bindings` 在 [scene_system.rs:170–200](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/scene_system.rs#L170-L200) 请求投影；`active_script_binding_projection_for_world` 在 [scene_system.rs:330–362](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/scene_system.rs#L330-L362) 仅以 World handle 和该代次判命中。重建会从所有行重新 `into_active`（[scene_system.rs:406–451](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/scene_system.rs#L406-L451)）；每条新实例的 `started` 都为 `false`（[scene_system.rs:203–235](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/scene_system.rs#L203-L235)）。故同一 World 中持续启用、实体和 JSON 完全未变的 B 下一次 Update 又先调用 `onStart` 再 `onUpdate`，最终经 [vm_plugin_manager.rs:350–386](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs#L350-L386) 解析并调用其活动 VM 插槽；无须 reload、Prefab 或重建 World。
- **UE 对照。** `Engine` 的 [Engine.Build.cs:73–81](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Engine.Build.cs#L73-L81>) 依赖 `CoreUObject`。`AWorldSettings::NotifyBeginPlay` → `AActor::DispatchBeginPlay` → `AActor::BeginPlay` → Blueprint `ReceiveBeginPlay`，见 [WorldSettings.cpp:337–353](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WorldSettings.cpp#L337-L353>)、[Actor.cpp:4811–4855](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Actor.cpp#L4811-L4855>)、[Actor.cpp:4881–4922](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Actor.cpp#L4881-L4922>)；声明见 [Actor.h:2113–2149](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Actor.h#L2113-L2149>)。这里比较激活生命周期，不拿 Blueprint 图资产或 opcode 数量做等价标准。

## 2. 有界实际风险

同一活动 World 内，A、B 各有一个启用的脚本，首帧都成功启动；例如随后 A 的 HP 由 24 降到 18（实体仍在）。若 B 的启动例程按约只初始化一次计数、初始化位置或注册监听，下一 Update 会把它重做；在类型代次持续变化的帧可反复发生。检验窗口只要一次实际 A 写和两次 B Update 即可，不必依赖线程切换、故障回调或定时竞争。不是所有场景都会触发：不改 `script.bindings` 的 Transform 或其他类型写不失效投影；等值 JSON 不推进代次。只陈述回调可观察结果；未宣称已见崩溃、全局失效、性能数字或真实项目回归。

## 3. 根因与 UE 真正处理

ZE 用 `Rc<ActiveScriptBinding>` 同时保管一次性启动事实和可失效的查询投影；缓存 miss 把 B 当作新 activation。UE 的 `ActorHasBegunPlay` 是实例自身字段（[Actor.h:674–685](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Actor.h#L674-L685>)），完成状态由 `HasActorBegunPlay()` 查询，`DispatchBeginPlay` 不对已完成实例再次分发，并在调用之前标 `BeginningPlay`、调用完成后标 `HasBegunPlay`。组件路径在 `AActor::BeginPlay` 对每个已注册且尚未开始的组件注册 tick 再调 `BeginPlay`，[ActorComponent.cpp:1654–1668](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/ActorComponent.cpp#L1654-L1668>) 对自身 `bHasBegunPlay` 检查和置位，Blueprint 条件成立才发组件 `ReceiveBeginPlay`。其他 Actor/组件的属性变化不拥有这个状态。`UObject::ProcessEvent` 的 `UFunction::Invoke` 路径见 [ScriptCore.cpp:2243–2255](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/ScriptCore.cpp#L2243-L2255>)；不是 VM 每帧重新决定开始状态。本报告不外推 `BeginningPlay` 期间重入、UE 网络初始化顺序和 Level Streaming 的全部政策。

## 4. Rust 做法

保留同一脚本系统与调度，用当前 World 生命周期下的 binding activation 持有既有 `started`，投影只派生 phase 列表和属性索引；刷新数据时重关联既有 activation，不做第二份启动状态。优先沿用现有可标识范围：同一 `WorldHandle`/World replacement epoch、存活 `EntityId`、同一槽且包/模块/参与阶段与完整 binding 未变；任何身份或配置不确定先不要继承。保持重复定义在各自槽位互不共享；真实替换世界要重置（使用 [level_system.rs:252–271](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/level_system.rs#L252-L271) 已有 epoch，严禁只凭可复用数值实体 ID 猜测）。可以先在同一 World 的重建逻辑重用未改 binding 的既有 `Rc`，不引入新全局状态机、第二调度器或新的脚本格式；若阶段间还要求排除 remove/insert 的同一 ID 重建，应以现有寿命标识或 writer 观察为准，不从终态 JSON 猜测。

## 5. ABI / ECS 差异

UE 把实例生命周期与 `UObject`/`AActor`/`UActorComponent` 关联，Blueprint 通过生成的 reflected event 和 `UFunction` 帧；ZE runtime 使用 World/ECS 稳定实体 ID、JSON 绑定及每个 VM 包的槽位/代际。现用场景桥向回调传实体整数和时间值（[scene_system.rs:238–279](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/scene_system.rs#L238-L279)），VM manager 已在调用时刷新 generation。要对齐的是“仍活着的同一实例不会因别人改属性而重新开始”，不复制 UE `UObject` 内存布局、`FFrame`、Blueprint 资产格式，也不改 native C ABI、Prefab 或热重载策略。

## 6. ZE 改法草案（严格先等价回归，再修复）

1. **先补一个有行为的 RED 见证。** 用已注册、成功的两个 Update 绑定，记录 A/B 各自实际收到的 export；首帧 B 接收 `onStart`、`onUpdate`。通过生产 World writer 只改变 A 的非致死 HP，确认 A 存活和类型级 generation 推进；再跑同一 Update。目标 B 序列是 `onStart, onUpdate, onUpdate`，当前源码预测 `onStart, onUpdate, onStart, onUpdate`。可用正常 VM mock/记录 backend 走 `ScriptSceneRuntimeSystem::run`，或在已有脚本 Play 集成见证上测计数；不要只 `Cell::replace` 或伪造 projection。加一个真正换 World 的对照，防止跨 replacement 偷带 `started`。此回归为 UE 源码推导契约，不冒称直接移植。
2. **RED 后在 [scene_system.rs:330–362,406–451](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/scene_system.rs#L330-L451) 调整重建衔接。** 新投影仍正确重建属性索引、阶段和顺序；只为同一 world 生命周期内可确认完全未改的 activation 重用原持有者，既有 callback 代际仍交给 manager；移除、替换或不明身份不得误继承。先让上项测试转绿，再核对原有不等值更新、重复 binding/替换 World、generation 失效测试；只修改确实与新契约冲突的旧断言，不降低测试门槛。不把 Fixed-first、fail-first 或稳定绑定 ID 格式迁移夹带进来。本问题提交仅新增文档，没有实现修复或运行测试；实际 Draft 状态以 GitHub 为准。

### UE 原有测试迁移表

| UE 测试名 / 路径行 | UE 输入与断言 | ZE 既有测试位置及可迁移边界 |
| --- | --- | --- |
| `System.Promotion.Editor.Blueprint Editor` (`FBlueprintEditorPromotionTest`)；[BlueprintEditorTests.cpp:54–60](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/Kismet/Private/Tests/BlueprintEditorTests.cpp#L54-L60>)、[同文件:1166–1203](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/Kismet/Private/Tests/BlueprintEditorTests.cpp#L1166-L1203>) | 用事件图添加 `ReceiveBeginPlay`、接 `Delay`，检查节点、撤销/恢复、2 秒默认值，然后编译。**未断言**另一 Actor 属性变化后启动计数。 | [play_session_factory.rs:84–87,155–229](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/entry_runner/editor/tests/play_session_factory.rs#L155-L229) 的 `scripted_play_runs_materialized_snapshot_and_retires` 用真实 `onStart` 设置位置、`onUpdate` 位移证明启动与后续更新；它是 `ignore` 的产品测试位置，不含第二绑定的修改。相邻测试，不是本缺陷移植。 |
| **未找到直接匹配；源码推导。** `AActor::DispatchBeginPlay`、`AActor::BeginPlay` / `UActorComponent::BeginPlay`，见第 1、3 段。 | 已完成的 B 不是新激活，期间 A 改值不应给 B 再派发一次开始。固定输入两对象、一次对 A 的非生命周期写、两次可观察回调。 | [scene_system 测试:36–123](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/tests/scene_system.rs#L36-L123) 只验证 Transform/其他**类型**写不重建，并手动置 `started`；[同文件:254–300](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/tests/scene_system.rs#L254-L300) 固化同一实体禁用一个重复槽位后的整表重建，不是两实体的行为 oracle；[combat_lifecycle.rs:210–271](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/gameplay_host/tests/combat_lifecycle.rs#L210-L271) 验证合法 host hit 由 24 降至 18，但不跑场景 Update。均未执行。 |

查找范围：UE `Runtime/Engine/Private/Tests`、`Editor/Kismet/Private/Tests`、`Editor/BlueprintGraph/Private/Tests`、`Runtime/BlueprintRuntime`、`Editor/UnrealEd/Private/Tests` 及 `Plugins/Tests/{RuntimeTests,EditorTests,CQTest}/Source`；按 `DispatchBeginPlay`、`HasActorBegunPlay`、`ReceiveBeginPlay`、`BeginPlay` 调用和测试宏核对。只找到 World 启动测试辅助和上面一类图创作/Promotion 相邻项，不宣称扫遍全部 UE 插件或内容资产。本轮 UE、ZE 编译及测试计数均为 **0**；只读调查未运行 Cargo 或编辑引擎，本问题提交只含调查文档。

### 模块与待核实清单

| UE 模块 | ZE owner | 读到的覆盖 | 待查（不是候选） |
| --- | --- | --- | --- |
| `Engine`（Actor/Component 生命周期） | `zircon_runtime::script::vm::scene_system` 与 World/ECS schedule | 上表中投影测试、combat host、脚本 Play | fixed-only/阶段首调、跨 World 或生命周期替换、failure fan-out、同 ID 删除重建与重新启用；需各自收敛证据。 |
| `CoreUObject`（UFunction/VM），[CoreUObject.Build.cs:26–44](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/CoreUObject.Build.cs#L26-L44>) | `zircon_runtime::script::vm::runtime` + `zircon_plugins/zr_vm_language` | 活动槽位 callback 调用/代际逻辑；测试只源阅 | 不以 VM 语言、GC 或 Blueprint latent/stack 实现差异为缺陷；需单独核验各自产品契约。 |
| `BlueprintRuntime`（[BlueprintRuntime.Build.cs:5–12](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/BlueprintRuntime/BlueprintRuntime.Build.cs#L5-L12>)）、Editor `BlueprintGraph` / `Kismet` | 脚本包与编辑器脚本入口，不是 Prefab | 区分 graph authoring/PROMOTION 与真实运行；本条调用入口归 `Engine`/`CoreUObject`，不归 Editor 图模块 | 编译产物和调试映射未比对；Prefab 依 Unity 方向独立审查。 |

### UE 本地源文件 SHA-256

| 路径（相对本地 UE 根） | SHA-256 |
| --- | --- |
| [Engine/Build/Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version>) | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| [Engine/Source/Runtime/Engine/Engine.Build.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Engine.Build.cs>) | `1c6a698c731ca203fc9648a53c13bcc3768fae1fb91cd9ec658a3abfa962e212` |
| [Engine/Source/Runtime/CoreUObject/CoreUObject.Build.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/CoreUObject.Build.cs>) | `cb5fab6f1b508e0ed8c2d712e24d097b91d317ed18a72f8cd36074bc763b20fe` |
| [Engine/Source/Runtime/BlueprintRuntime/BlueprintRuntime.Build.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/BlueprintRuntime/BlueprintRuntime.Build.cs>) | `27476be339fbff873d53828a51a273a5731b7657750621555478f955ddf987d0` |
| [Engine/Source/Editor/BlueprintGraph/BlueprintGraph.Build.cs:6–23](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/BlueprintGraph/BlueprintGraph.Build.cs#L6-L23>) | `02810269725c5a15a17ac5266e8a0cf15997cf15be3bb7d3c6ea456f88987c34` |
| [Engine/Source/Editor/Kismet/Kismet.Build.cs:5–16](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/Kismet/Kismet.Build.cs#L5-L16>) | `cd024d684f319705f08cd6790ddc89f046b80b5276a8bf451d553525fe0851ae` |
| [Engine/Source/Runtime/Engine/Private/Actor.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Actor.cpp>) | `352df4c839c47f27b0aea1b217319a24bb857b3e5b03e18869d71fdb958cd527` |
| [Engine/Source/Runtime/Engine/Classes/GameFramework/Actor.h](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Actor.h>) | `5af0b0475274fb4b07f08701d34ff76ae61fe8a1cd6eeee534a3e627bd9380d8` |
| [Engine/Source/Runtime/Engine/Private/Components/ActorComponent.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/ActorComponent.cpp>) | `43d292aff1672ecdcff8676be13f995f9491233ad8480903f0ce2bfbc71c361f` |
| [Engine/Source/Runtime/Engine/Private/WorldSettings.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WorldSettings.cpp>) | `62c04800af3d38750a7d1fb5ba41e79796029549e924c1224de8eb9fbbbde426` |
| [Engine/Source/Runtime/CoreUObject/Private/UObject/ScriptCore.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/ScriptCore.cpp>) | `bfa201706983bd8886d42f7b5741ba00d1065cd191f8f4e98b86c3a78e064330` |
| [Engine/Source/Editor/Kismet/Private/Tests/BlueprintEditorTests.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/Kismet/Private/Tests/BlueprintEditorTests.cpp>) | `2838f0ab4ae694f4c35580d99e0d28dc47a725d99a8daf614a5e5582dbba7b72` |

**去重：** lead 本地 [PR 快照](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/github-pr-snapshot.json>) 的条目 #131–145（标题和正文）无同根因；[#131](https://github.com/He-Jiahui/ZirconEngine/pull/131) 是 ECS 阶段顺序测试，不包含既有激活复用。`git log --branches --not e893… --` 对 `scene_system` owner 和该测试文件无未合提交输出；不是对其他会话未提交实现的接管，也不是全量远端查重。以既有 P1-10 下的这个子项追踪，不新增一套 canonical 生命周期计划。

## 7. 三个正式脚本调度回归已冻结；第一次构建未获资源准入

独立测试源码 `a7b178dbadd1d60eb9434c9c4176bdbf301c7b20`，`start_projection`，Git blob `9b53dccb8fcca9006443a308201c0f820eeaefbf`。完整首方 Client 插件目录及模块注册、定向激活脚本模块的真实依赖，注册系统在普通 `LevelSystem::tick` 的 Update 经 `VmPluginManager` 派发。最末层为经正规 API 注册的诊断后端，读取真实属性并给两个不同的非空回复；不冒充原生 ZrVM、全客户端或 UE 原测。

| 计划的可判别用例 | 要验证的现象 |
|---|---|
| 普通属性写 | 第一帧 A/B 各开始并更新。只改 A 的颜色 0.25→0.75，下一帧 A 从 host 读到新值，而 B 仍只更新，不应再次启动。 |
| 等值写与两个同样的 authored 槽 | 等值写不推进代次，两个槽各自启动一次，第二帧仍各更新一次。 |
| 真正换 World | 相同数字实体号不足以冒充旧实例；新世界开始后需要自己的启动。 |

第一次尝试明确选择 `zircon_plugin_zr_vm_language_runtime --no-default-features --test start_projection --no-run --profile test --locked --offline`，但物理内存一直低于本机并发准入线，**未启动 Cargo、未运行用例**。这是本机资源前置，不是脚本错误。后续从同三例、同源码重新验证，不提前改生产实现。
