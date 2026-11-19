# 物理/碰撞 · PH-01：旋转 Box 的 AABB fallback 丢失朝向，shape cast 可漏报

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

**处置建议：只作为一条独立、问题定义先行的 Draft 候选；不是修复提交。**

- ZE 审计基线：本地 `cb495bb1c952a24c86f4513b8099bc4942c3a87d`；按 lead 核对后的可访问发布基线，本文 ZE 引用固定到 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。原生 Git 比较确认两者在 `zircon_runtime`、`zircon_runtime_interface`、`zircon_plugins` 无差异。没有采用他人脏技能内容。
- 本地 Unreal 是 [Build.version:2–9](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L2-L9>) 所示 **6.0.0 / BranchName UE5**，文件 SHA256 `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6`。不称为 UE5.7，也不由弃用宏推断版本。
- **证据等级：源码可达性、输入接纳和几何反例高置信；未运行 Cargo、ZE/UE 测试或产品。** 文中的返回值均为逐分支源码推导，不是实测 RED。
- **契约反证不能省略：** [PhysicsShapeCastQuery:20–23,40–42](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/physics/shape_cast_query.rs#L20-L42>) 已明确当前实现为 AABB 近似、不承诺完整旋转几何；[PhysicsQueryInterface:99–103](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/physics/query_interface.rs#L99-L103>) 也不把结果当精确接触点。另一方面，[当前插件文档:302](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/crates/zircon_plugins/physics/runtime.md#L302>) 称其为 conservative sweep / oriented AABB。本文只提出**支持的有效 Box 不能因忽略旋转而使包围体缩小**这一窄 alignment gap，不虚构已经承诺 exact narrow phase。

## 1. 和 Unreal 哪里不一致：精确位置与真实调用链

### ZE：朝向被保留到快照，但构造 sweep 包围体时不再参与计算

| 证据 | 符号、位置与作用 |
|---|---|
| [Z1 · world sync](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/manager/world_sync.rs#L37-L100>)；[同文件:311–316](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/manager/world_sync.rs#L311-L316>) | `build_world_sync_state` 读取实体 world/local transform；`combine_transforms` 将实体与 collider local rotation 相乘，存入 `PhysicsColliderSyncState.transform`。单位缩放、无父节点即可触发，不涉及 affine 分解。 |
| [Z2 · runtime system:94–109](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/runtime_system.rs#L94-L109>)；[service:121–141](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/manager/service.rs#L121-L141>) | 已安装 `physics.step` 调用 `tick_scene_world`；builtin 分支会同步真实 World 的 collider 快照，QueryOnly 不需要动态刚体。 |
| [Z3 · service:88–98](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/manager/service.rs#L88-L98>)；[同文件:183–194](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/manager/service.rs#L183-L194>) | `PhysicsManager::shape_cast` 和实际 `PhysicsQueryInterface` 实现均调用共享 `query::shape_cast`，不是未调用的 backend trait 占位。 |
| [Z4 · query:69–83](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/manager/query.rs#L69-L83>) | 接纳有限距离、方向、姿态后，从同步快照进入 `shape_cast_query`。这里没有按 backend 分流到 native sweep。 |
| [Z5 · sweep:18–45,47–87](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/backend/builtin/query_contact/sweep.rs#L18-L87>) | 请求完整 `origin_transform` 被放进临时 collider；先做初始重叠，再分别调用 `collider_aabb` 取得 query 和 target bounds，膨胀后射线测试。 |
| [Z6 · geometry:50–91](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/backend/builtin/query_contact/geometry.rs#L50-L91>) | **缺陷 owner `collider_aabb`**：Box 只计算 `half_extents × abs(scale)`，最后 `translation ± half_extents`；函数没有使用 `transform.rotation`。旋转既不作用于目标 Box，也不作用于请求 Box。 |
| [Z7 · overlap proxy:45–55](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/backend/builtin/overlap/proxies.rs#L45-L55>)；[slab:20–27](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/backend/builtin/query_contact/raycast/aabb.rs#L20-L27>) | 初始 overlap 也不会替本例纠错；随后平行于 Y slab 的射线一旦位于缩小的 Y 区间外，确定返回 None。 |

可达链：**安装 Physics runtime systems → `physics.step` → `tick_scene_world` → `build_world_sync_state` / `sync_builtin_world` → 公共 manager 或 query bridge 的 `shape_cast` → `query::shape_cast` → `shape_cast_query` → `collider_aabb` → `ray_cast_collider` / slab**。现有 [集成 fixture:52–85](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/mod.rs#L52-L85>) 的源码已安装并调用此链；这不是本轮执行回执，也不证明当前 fixture 已可编译。当前 [spawn_node:95–99](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/bootstrap.rs#L95-L99>) 返回 `SceneResult<EntityId>`，而 [原 query fixture:664–674](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/query.rs#L664-L674>) 未解包便传给 `set_collider`，静态可见 API drift 必须先核对处理。这里只证明插件装配路径，不声称普通默认 App profile 已启用 Physics。

### UE：查询姿态进入 transformed bounds 和窄相，而非只保留平移/缩放

以下 U 编号的文件 SHA256 见文末本地证据表。

1. [U1 · `UWorld::SweepSingleByChannel`:163–171](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/WorldCollision.cpp#L163-L171>) 将 `Rot` 传给 `GeomSweepSingle`。
2. [U2 · `FGenericPhysicsInterface::GeomSweepSingle`:1045–1053](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/SceneQuery.cpp#L1045-L1053>) 传 `FGeomSQAdditionalInputs(CollisionShape, Rot)`；[本地查询分支:883–903](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/SceneQuery.cpp#L883-L903>) 在没有 query handler/显式忽略时调用普通 scene cast。
3. [U2 · `TSceneCastCommonImpWithRetryRequest`:533–556](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/SceneQuery.cpp#L533-L556>) 构造带 rotation 的 `StartTM`；[U2 · `TSQTraits::SceneTrace`:299–302](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/SceneQuery.cpp#L299-L302>) 转入 `LowLevelSweep`。
4. [U3 · `LowLevelSweepImpl`:108–120](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/SceneQueryLowLevel.cpp#L108-L120>) → [同文件 `FGenericChaosSQAccelerator::Sweep`:39–45](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/SceneQueryLowLevel.cpp#L39-L45>) → [U4 · `SweepHelper`:680–698](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Public/SQVisitor.h#L680-L698>) 使用 `CalculateTransformedBounds(StartTM)` 做 broad phase。
5. [U4 · `TSQVisitor`:334–336](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Public/SQVisitor.h#L334-L336>) 将 target `ActorTM` 与 query `StartTM` 一并传给 [U5 · `Chaos::SweepQuery`:252–324](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Experimental/Chaos/Public/Chaos/GeometryQueries.h#L252-L324>)；该实现构造相对刚体变换和 local direction，选择专用 raycast 或 `GJKRaycast2`，没有将 rotation 丢弃。

测试真实入口是 [U7 · `TEST(GJK, GJKRaycastTests)`:239–246](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosApp.cpp#L239-L246>) → [U6 · `GJKSphereBoxSweep`:817–900](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosTestGJK.cpp#L817-L900>)，含明确的 rotated box/needle 子用例。不是从未接线的 helper 猜测 UE 行为。

## 2. 实际影响：一个有效、非退化反例

以下均为单位缩放、无父子层级、有限标准单位 quaternion；所有 filter 使用默认可命中设置，快照中只有一个目标。

- 目标：Box，local half extents `(2, 0.25, 0.25)`，world center `(0,0,0)`，绕 Z 轴旋转 90°。
- 请求：Sphere，radius `0.1`，起点 `(-3,1.5,0)`，方向 `+X`，最大距离 `6`。
- 真正的 world Box 范围：X `[-0.25,0.25]`、Y `[-2,2]`、Z `[-0.25,0.25]`。球心经过 box 的长边内部，首次接触距离为 `3 - 0.25 - 0.1 = 2.65`。
- 当前 `collider_aabb` 给出 Y `[-0.25,0.25]`；膨胀 radius 后仍只有 `[-0.35,0.35]`。球心路径恒有 `Y=1.5`、`direction.y=0`，所以 slab 分支必拒绝。起点 X=-3 距离初始近似 Box 也大于 radius，不会被 initial-overlap 路径提前命中。

**源码推导结果：返回空向量，而真实受支持的 Box 被路径穿过。** 这是漏报，不只是 conservative 方法允许的提前命中/假阳性。调用者得到的可观察事实仅是“没有 hit”；本文不推断某个未找到的角色控制器已穿墙，不声称 crash、帧率或性能损失。

输入接纳有依据：[scene shape validation:42–51](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/manager/validation.rs#L42-L51>)、[finite transform:87–95](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/manager/validation.rs#L87-L95>) 与 Z4/Z5；这些入口不拒绝标准旋转或此 Box。该 witness **尚须通过现有真实 manager fixture 执行复核**。

## 3. 根因及 UE 实际解决

根因是把**局部轴半尺寸乘缩放**误用作**世界轴包围半尺寸**。一般有效 Box 的世界 AABB 半尺寸应由旋转后的三条缩放基向量的绝对分量累加，即 `abs(R) × (abs(S) × h)`。这里用数学表达说明几何关系，不转载 Epic 实现。

UE 的对应做法分两层：U4 先按完整 `StartTM` 求 transformed bounds，随后 U4/U5 在 target/query 的真实刚体相对姿态下求窄相。ZE 不必在此 PR 一并实现第二层；修正支持 Box 的 conservative bound 已足以消除上面的漏报，并能独立验证。

已有 [Runtime138:187–193](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/99zm-physics-world-review.md#L187-L193>) 泛列 snapshot queries 与 rotated primitive 近似。这是相关历史背景，不冒称首次发现该领域有近似；本次新增的是可逐分支核验的**支持 Box 的非保守缩界反例与现成 UE 用例映射**。若 owner 决定所有旋转 Box 本来就完全不支持，应先统一能力/文档契约，而不是静默声称已完成完整旋转物理。

## 4. 不发明新架构的 Rust 做法

- 保留 plugin-owned query helper、现有 `Transform` / `Quat` / `Vec3`、snapshot 和 query filter。
- 在 Box bounds owner 内，使用现有 quaternion-vector 运算，计算三个旋转后轴向量的绝对值乘相应 scaled half extent，再累加为 world half extent；也可以用已有矩阵类型的绝对基列。无需新 solver、shape cache、服务、job 或 ECS 系统。
- 对本例固定使用已合法接纳的 normalized rotation；非单位/零 quaternion 的统一验证政策不夹带进这条缺陷。
- 保留现有 finite checks、单位/负缩放的绝对值处理和 unsupported shape 政策。本条先限 Box，不能以此声称 capsule/cylinder/convex 的所有查询也已精确。

## 5. ABI / ECS 差异

- UE 的 UWorld/UPrimitiveComponent、Chaos implicit geometry 与 FTransform 不是 ZE 的实体组件布局。ZE 在 ECS 之外投影拥有型 `PhysicsWorldSyncState`，查询借用快照；本条不要求把 UE 对象模型搬入 ECS。
- [PhysicsShapeCastQuery:33–54](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/physics/shape_cast_query.rs#L33-L54>) 已携带完整姿态；修正 Box 包围体不需要新增 DTO 字段、序列化版本、桥接 ID 或动态库 ABI。
- 当前 plugin bridge 是 runtime-owned Rust query trait；JoltC native sweep 并未因此变得可用。不能把本条改成大范围 native query 迁移。
- ZE hit position 是 query-center path，UE 测试的 position 是接触点；等价测试只迁移语义相同的 hit、距离和适用的法线，不把两种 position 混为同一个数值 oracle。也不从 UE 的旋转处理外推 full-affine parenting；本例没有父节点、非均匀层级剪切。

## 6. ZE 具体改法：仅草案与等价测试清单

顺序必须是：**先独立问题 Draft → 同一 Draft 补等价/回归测试并实测原实现 → 再评估最小修复。** 本轮没有实施任一步外部发布或引擎改动。

- 若契约确认后允许修复，生产 owner 首选仅 Z6 的 `collider_aabb` Box 分支；Z5 仍调用同一 helper，不先引入 GJK/native sweep，也不改父子 world→local 回写。
- 测试落到现有 [query 集成模块:651–718](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/query.rs#L651-L718>) / [真实 fixture:52–85](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/mod.rs#L52-L85>)，用 QueryOnly builtin、真实 set_collider/update_transform、真实 level tick 后调用公共 manager，不能只测重写的数学函数。测试文件的最终拆分遵循现有大小规则，当前不新增测试文件。
- **测试可行性前置**：先在原 harness 独立核对上述 `spawn_node` 返回值漂移，若需修正按独立问题的 Draft-first 流程处理并保留原控制；不得夹带到本几何生产修复，也不能把静态 API drift 冒称已运行的编译失败，或把编译失败当作本缺陷 RED。原实现编译、manager 回归及 RED/GREEN 均未执行。
- 原有 `builtin_shape_cast_reports_initial_overlap_and_swept_hit` 与 [query modes:720–809](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/query.rs#L720-L809>) 继续作为控制，不弱化或删除。

### 等价测试表（均为计划，未迁入、未执行）

| 类型 / UE 测试与精确位置 | 输入（UE→ZE 表示） | UE 原期望 / ZE 对应期望 | ZE 既有落点与边界 |
|---|---|---|---|
| **现成用例迁移**：U7 `GJK.GJKRaycastTests` → U6 `GJKSphereBoxSweep` ordinary hit，[833–836](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosTestGJK.cpp#L833-L836>) | UE A min `(3,-1,0)`, max `(4,1,4)` → ZE target Box center `(3.5,0,2)`, half `(0.5,1,2)`；query Sphere radius1 从原点沿 +X，length4 | hit，distance≈2，normal≈(-1,0,0)。不迁 UE contact position=3 作为 ZE query-center position | 上述 query fixture；无旋转控制，保持原支持能力 |
| **现成用例迁移**：同测试 rotated box，[887–892](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosTestGJK.cpp#L887-L892>) | UE 将偏心 A 绕 Z90；ZE 用已旋转中心 `(0,3.5,2)`、half `(0.5,1,2)`、Rz90 的 query Box；target Sphere 在原点 radius1；方向 -Y，length4 | hit，distance≈2，normal≈(0,1,0)。此特定正交姿态下正确 AABB 的 TOI 与 UE 原测试一致；当前源码推导为1.5 | 同模块；区别在 Box 的中心与 orientation 都必须正确映射，不能把非居中 UE shape 当成 ZE 居中 Box |
| **现成用例迁移**：同测试 degenerate rotated needle，[894–899](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosTestGJK.cpp#L894-L899>) | query Box center `(0,3.5,0)`、half `(0.5,0,0)`、Rz90；target Sphere 原点 radius1；方向 -Y，length4 | hit，distance≈2，normal≈(0,1,0)；当前源码推导为2.5 | 同模块。ZE [Box geometry validation:106–110](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/backend/builtin/query_contact/geometry.rs#L106-L110>) 与 scene validation 均允许零半轴，因此不是无效输入；它仅为辅助控制，不作唯一 witness |
| **源码推导，待复核；不是 UE 现成测试迁移** | 第2段非退化 target Box + 小球；另将旋转放在 query Box，target Sphere center `(0,1.5,0)`、query center `(-3,0,0)`、其余相同 | 应命中；本特定 fixture distance≈2.65。当前源码路径均可推导为空 | 同 query fixture，分别覆盖 target/query 两次 Z6 调用，属于同一根因 |
| **源码推导，待复核；不是原测迁移** | needle 原测仅将 length 从4截断至2.25 | 正确 TOI2 仍在范围内；当前推导TOI2.5导致漏报 | 同模块；明确标扩展变体，不挂原 UE 测试名冒充相同输入 |

UE 的 `InitialDirs` 六组输入是 GJK 搜索初始方向，而不是 ZE 公共查询参数；迁移公共 manager 用例时无需捏造该字段。保留 UE 测试的几何/姿态/扫掠方向与 epsilon 意图，float 断言使用说明过的容差。

### 本地 UE 证据 SHA256（整文件字节；与上述行号绑定）

| ID / 本地源码 | SHA256 |
|---|---|
| [U1 · WorldCollision](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/WorldCollision.cpp#L163-L171>) | `cf0227e96078bc4cd84b410341111250faba082ca24eb727b101ee32eb7b842c` |
| [U2 · SceneQuery](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/SceneQuery.cpp#L1045-L1053>) | `ad077fbb4887aec4aff530154119ec4ad3b005158e6d190eb43a513144647d87` |
| [U3 · SceneQueryLowLevel](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/SceneQueryLowLevel.cpp#L108-L120>) | `ee1f356823fe5d5eefcf3a11845b190efb7e5e0a2a05a6c5b42a56c1a492af6e` |
| [U4 · SQVisitor](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Public/SQVisitor.h#L680-L698>) | `f99b1cef86098df124c36c28aae7c985f152ff36ff1b7a890309dca3540251ea` |
| [U5 · GeometryQueries](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Experimental/Chaos/Public/Chaos/GeometryQueries.h#L252-L324>) | `4a1df26aba0b2a7eaff9e892fd6bf37e879ac7019d8210473a1e338baacd650f` |
| [U6 · HeadlessChaosTestGJK](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosTestGJK.cpp#L817-L900>) | `745b2cce799d5e42d1fe73b615b11f575564480ee3fe38d651651cd4b248cabf` |
| [U7 · HeadlessChaosApp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosApp.cpp#L239-L246>) | `9611d3414e6284dcedf1a35dc90b51fbb765fe9baf81b6e776f4f45f81b53810` |

### 去重与范围结论

只计 **PH-01 一条**。与 [PR145](<https://github.com/He-Jiahui/ZirconEngine/pull/145>) 的 solver world→parent-local 回写不同：本例没有父节点、刚体或 solver pose 写回。与 [PR135](<https://github.com/He-Jiahui/ZirconEngine/pull/135>) / [PR144](<https://github.com/He-Jiahui/ZirconEngine/pull/144>) 的 render/viewport affine packet 无关。本条也不是泛“Jolt 没有 native query”的重报。领域覆盖与未查清事项另见 [物理覆盖记录](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/research/physics-coverage.md>)。

## 7. 首次公开接口实测：用原生产代码复现

[本次实际编译的提交](https://github.com/He-Jiahui/ZirconEngine/commit/a30629fd4b86d9130fe56fb623e0426ae965370e) `a30629fd4b86d9130fe56fb623e0426ae965370e`。一个普通根实体挂球或 Box collider，`build_world_sync_state → PhysicsManager::sync_world → synchronized_world` 先验证实体、中心、缩放与朝向，再通过公共 `PhysicsQueryInterface::shape_cast` 询问。Builtin / QueryOnly，无刚体求解、剪切层级、Jolt FFI、UE 二进制或完整游戏启动。

Windows x64 / MSVC，`zircon_plugin_physics_runtime`、`--no-default-features`、额外 package features 空，`--locked --offline --profile test --jobs 1 --test physics_rotated_box_query`。编译和执行证据分开：EXE SHA-256 `018315737746c007da1761c498d2391560306b4a70c115be6dfa67693e7afe72`，实际执行回执 `e1f8b8e14b1f6b29618e5f527ae724d4a4c514e7b6d06d1aeba2306580f272b0`，唯一筛选 `rotated_sweep_ --test-threads 1 --show-output`。**10 过 / 5 败**，未忽略或过滤选中的用例。

| 真实请求 | 实际结果（查询中心沿途的距离） | oracle |
|---|---|---:|
| right-angle target Box | 未命中 | 2.65 |
| right-angle query Box, q | 未命中 | 2.65 |
| right-angle query Box, -q | 未命中 | 2.65 |
| 45-degree target Box | 未命中 | 1.30901 |
| UE ordinary sphere against Box | 2 | 2 |
| UE rotated Box against sphere | 1.5 | 2 |
| UE rotated needle against sphere | 2.5 | 2 |

UE `GJK.GJKRaycastTests → GJKSphereBoxSweep` 的普通球/Box、旋转移动 Box、旋转退化针形 Box 三组保留了几何、固定方向与命中距离；旋转组先把原来偏心的 Box 中心变换到世界。UE 的接触点不等于 ZE 命中位置，不移植 GJK 内部寻向、MTD、EPA 或宣称任意角度精确 TOI。另设 90° 两侧和 45° 的 ZE 反例；45° 的数值是保守代理入口而非精确表面接触。原有无旋转、距离不足、不存在的 World、排除实体、界外方向、零程及有限非单位 query 姿态用例分别列出。

| 冻结测试 | 结果 |
|---|---|
| `rotated_sweep_control_entity_filter` | 通过 |
| `rotated_sweep_control_finite_nonunit_query_rotation` | 通过 |
| `rotated_sweep_control_identity_query_box` | 通过 |
| `rotated_sweep_control_identity_target_box` | 通过 |
| `rotated_sweep_control_oblique_short_and_off_axis` | 通过 |
| `rotated_sweep_control_right_angle_outside_bounds` | 通过 |
| `rotated_sweep_control_short_distance` | 通过 |
| `rotated_sweep_control_unpublished_world` | 通过 |
| `rotated_sweep_control_zero_distance_initial_overlap` | 通过 |
| `rotated_sweep_oblique_target_box_keeps_bound` | 失败 |
| `rotated_sweep_query_box_keeps_bound` | 失败 |
| `rotated_sweep_target_box_keeps_bound` | 失败 |
| `rotated_sweep_ue_ordinary_sphere_hits_box` | 通过 |
| `rotated_sweep_ue_rotated_box_keeps_contact_distance` | 失败 |
| `rotated_sweep_ue_rotated_needle_keeps_contact_distance` | 失败 |

此处仅提交等价回归及实际观测。原先可能修改共有 AABB 的设想也会影响静态 overlap、trigger 和 contact；后续只评估正距离 sweep 自身两端，非单位旋转、零程及不能表示的新范围均须保留原兼容边界。
