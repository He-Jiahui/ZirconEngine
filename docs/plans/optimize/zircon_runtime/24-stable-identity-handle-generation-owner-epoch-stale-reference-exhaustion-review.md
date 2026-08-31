---
related_code:
  - zircon_runtime_interface/src/resource
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/runtime_api
  - zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs
  - zircon_runtime_interface/src/world_sync
  - zircon_runtime/src/core/resource
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/ecs/entity
  - zircon_runtime/src/scene/ecs/archetype
  - zircon_runtime/src/scene/ecs/messages
  - zircon_runtime/src/scene/ecs/observer
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/scene/world/project_io/document.rs
  - zircon_runtime/src/scene/world/generation
  - zircon_runtime/src/scene/inspection/subscription
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/runtime
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/rhi.rs
  - zircon_runtime/src/graphics/scene
  - zircon_runtime/src/core/framework/ai
  - zircon_runtime/src/core/framework/sound
  - zircon_runtime/src/navigation
  - zircon_runtime/src/core/framework/net
  - zircon_runtime/src/ui/surface/timeline.rs
  - zircon_runtime/src/text/sdf/font_bake
  - zircon_plugins/particles/runtime/src/component.rs
  - zircon_plugins/particles/runtime/src/service.rs
  - zircon_plugins/navigation/runtime
  - zircon_plugins/sound
  - zircon_plugins/physics/runtime
  - zircon_editor/src/core/project
  - zircon_editor/src/scene
  - zircon_editor/src/core/runtime_event_consumer
tests:
  - zircon_runtime_interface/src/tests
  - zircon_runtime/src/scene/tests/ecs_identity_storage.rs
  - zircon_runtime/src/scene/tests/ecs_events_messages.rs
  - zircon_runtime/src/scene/tests/world_basics
  - zircon_runtime/src/scene/world/generation/tests.rs
  - zircon_runtime/src/render_graph/tests
  - zircon_runtime/src/tests/runtime_absorption
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_tooling/20-cargo-package-workspace-feature-dependency-target-graph-build-receipt-review.md
  - docs/plans/optimize/zircon_tooling/22-magic-constant-sentinel-threshold-timeout-capacity-budget-policy-convergence-review.md
  - docs/plans/optimize/zircon_tooling/23-failure-contract-panic-unwind-error-propagation-poison-recovery-result-observability-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/WeakObjectPtr.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/ObjectHandle.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/WeakObjectPtr.cpp
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/Fyrox/fyrox-core/src/pool/handle.rs
  - dev/godot/core/templates/rid_owner.h
  - dev/godot/core/object/object_id.h
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 24 · Stable Identity、Handle、Generation、Owner Epoch、Stale Reference 与 Exhaustion 工程化差距

## 1. 结论

Zircon并非完全依赖裸整数身份。资源层的`ResourceId/AssetUuid`使用UUID，`ResourceHandle<T>`携类型参数；Scene ECS内部的`InternalEntity`使用slot index与generation并在查找时验证；脚本`HostRegistry`使用`index + generation`，对slot索引和generation耗尽返回显式错误，generation耗尽时保留有效记录而不回绕；native host context会在generation达到上限时永久退役slot；Render Graph handle携builder generation并拒绝跨builder使用；字体数据库handle会检查数据库generation与并发重建；`VmObjectRef`通过`Arc` root lease在最后一个引用释放时注销。这些是可保留的工程基础。

真正缺失的是统一身份架构。仓内至少同时存在五类语义：跨运行持久ID、owner内live handle、必须由session/world/device限定的live handle、message/operation correlation sequence，以及表示状态变化的revision/generation。它们经常都表现为`u64/u32/usize`新类型，部分还统一derive序列化，却没有中央`IdentityKind`、owner domain、序列化资格、耗尽策略或错误分类。结果是某些owner正确使用代际slot，另一些owner普通加一、饱和后复用、原子回绕或直接panic；调用者无法仅从类型判断值能否持久化、能否跨world/device/process使用、是否拥有资源、是否只是一次关联号。

最直接的持久身份缺陷位于`stable_uuid_from_components`：名为stable的UUID由标准库`DefaultHasher`两次输出拼成。标准库没有把该hasher算法承诺为长期持久格式，Rust/toolchain变更可能让同一label生成不同UUID，而Resource与Asset的稳定label入口依赖它。当前也没有算法版本、旧值alias/migration表或BuildSet digest来承接切换。Scene载入另有确定的边界错误：`max(entity_id) + 1`使用普通加法；包含`u64::MAX`的工程文档会在debug构建panic、在release回绕到0，并可能与active-camera的0哨兵语义相撞。

live handle的风险也不是单一模式。`InternalEntity`能阻止同一registry内常见的use-after-despawn，但自身公开、可序列化且不携World owner；不同World、clone或替换后的相同bits无法区分。slot数量被`as u32`截断，generation在`u32::MAX`后回到首代，理论上可重新接受极老引用。另一方面，World generation、lifecycle revision、message/event cursor generation等使用`saturating_add`，到上限后会冻结，不再表达状态变化。archetype membership generation直接wrapping，完整回绕后可能让旧query plan再次等于当前值。

跨模块分配策略进一步放大不一致：默认Level manager、runtime session、network connection和若干RHI路径使用`fetch_add`或普通加法而不检查0、回绕与live collision；particles在`u64::MAX`后持续产生同一handle并覆盖`BTreeMap`中的live instance；sound/navigation/AI/observer等owner也没有统一耗尽结果；message ID虽然checked，却用`expect`把容量耗尽升级成进程panic。现有脚本registry已经证明“checked conversion、slot retirement、typed error”能在本仓落地，但没有成为共享规范。

本篇拥有跨runtime的identity taxonomy、owner/domain qualification、allocation/generation/exhaustion contract、serialization eligibility、stale/wrong-owner error模型和统一qualification evidence。Runtime05继续拥有Scene/ECS具体world lifecycle与public EntityId改造；Runtime Interface 01拥有DLL ABI handle布局和session registry实现；Runtime08B/08D/08E/08F拥有audio/navigation/network/AI局部生命周期；Runtime09A拥有RHI device loss与GPU retirement；Editor03拥有document/selection identity。本文引用这些局部证据，但不把相同缺陷重复登记为第二个owner。本轮登记 **0项P0、40项P1和12项P2**，均未实施。

## 2. 审查边界、方法与 currentness

### 2.1 物理扫描

本轮对`zircon_runtime`、`zircon_runtime_interface`、`zircon_plugins`、`zircon_editor`、`zircon_app`与`zircon_hub`的11,936个production-like Rust文件执行身份信号inventory。发现167个公开identity声明分布于127个文件；其中74个是单字段数值tuple wrapper，分布于45个文件：Editor含12个`u64`，Plugins含1个`u32`和2个`u64`，Runtime Interface含2个`u32`和8个`u64`，Runtime含4个`u32`、41个`u64`和4个`usize`。generation/revision/epoch/token信号约3,308处/562文件，atomic fetch分配候选32处/29文件。

这些数量只作为路由，不自动等于缺陷。GPU offset、协议sequence、统计sample ID、持久UUID、代际slot和外部设备ID本来就应有不同形态；`saturating_add`可能是正确的累计指标策略，但不适合必须保持唯一的handle。所有finding均来自owner、分配器、查找验证、释放、持久化和跨边界调用链的联合阅读。

### 2.2 深读调用链

1. `stable label -> stable_uuid_from_components -> ResourceId/AssetUuid -> builtin/imported resource reference`。
2. `World document deserialize -> normalize_loaded_state -> next_id -> EntityRegistry stable/internal mapping -> despawn/reuse`。
3. `World replacement -> WorldGeneration/replacement epoch -> async writer/watch subscription/invalidation DTO`。
4. `runtime session -> operation/subscription/allocation handle -> ABI call -> registry lookup -> teardown/quiescence`。
5. `HostHandle/native context/render graph/font handle -> generation validation -> retirement/lease`正向模型。
6. particle、sound、navigation、AI、network、RHI、UI timeline等递增分配器的0值、max值、collision与owner teardown行为。

本轮源revision为`ae2be3d865a937b9ed368bf965592045346c64e3`。稳定UUID、Scene identity、runtime operation与多数被读owner在检查时没有工作区差异；Scene schedule/tests、Editor、Hub及其他区域仍有其他Session在途修改，因此标记`source_recheck_required: true`。既有Editor、Hub、WOC、plugin动态验证阻断没有变化；本篇是source-level identity review，不重复运行无法覆盖近耗尽、跨owner stale和长期持久格式的全量Cargo/npm lane。

## 3. 当前可保留的工程基础

| 基础 | 当前证据 | 保留条件 |
|---|---|---|
| 持久资源UUID | `ResourceId/AssetUuid`使用UUID，typed resource handle保留resource kind/type | 更换为versioned稳定算法并提供旧值迁移，不能因算法缺陷退回路径字符串或递增整数 |
| ECS内部代际句柄 | `InternalEntity { index, generation }`查找验证slot generation与stable ID | 补World owner epoch、checked slot宽度和不回绕的retirement policy |
| 脚本HostRegistry | checked `u32` slot、generation mismatch/exhaustion错误，耗尽时不伪造新代 | 抽取为共享allocator policy和conformance suite，而非复制实现细节 |
| native context retirement | generation最大值slot永久退役，lookup在pin前后复核generation | slot index耗尽不能`expect`，且ABI token要有kind/domain诊断 |
| Render Graph generation | texture/buffer/pass handle绑定builder generation | 明确generation来源和overflow策略，并继续限制为单builder临时身份 |
| lease语义 | `VmObjectRef`以共享root lease在last-drop时注销 | 将strong/weak/lease/borrow差异显式进入更多public handle合同 |
| 字体registry快照 | index、database generation与并发rebuild复核能拒绝stale snapshot | 升级为通用owner epoch模式，保持失败可观测性 |
| allocation/session校验 | runtime allocation registry验证session并在上限返回错误 | 统一到所有ABI owner，并把错误从字符串/局部enum收敛为公共分类 |
| replacement epoch局部防护 | LevelSystem异步写入会检查world replacement epoch | 把epoch纳入通用World identity，避免每条异步路径单独记忆检查 |
| 模块新类型 | 多数handle已是newtype而非裸参数 | newtype必须携语义metadata和codec policy，不能只提供类型名外观 |

## 4. 参考实现给出的边界

### 4.1 Unreal：live weak identity与可解析对象引用分层

Unreal的`FWeakObjectPtr`使用对象数组index与serial number验证live对象，不把原始地址当稳定身份；`ObjectHandle`又区分resolved/unresolved表达与对象路径解析。可借鉴的是把live weak reference、持久/软引用和已解析对象所有权分层。Zircon不应把`EntityId`、资源UUID、RHI handle和ABI session token统一包装成同一“万能ObjectId”。

### 4.2 Bevy：generation阻止常见stale，但明确限定同一App实例

Bevy Entity由index和generation组成，free后提升generation并在访问时验证；其文档也明确bits只在同一App实例中有意义，generation回绕后理论上会alias。这个边界很重要：代际句柄能解决常见slot reuse，不自动解决跨World、跨process、无限生命周期或持久化身份。Zircon需要在generation之外增加owner epoch与序列化禁令。

### 4.3 Fyrox：typed pool handle把owner元素类型带入编译期

Fyrox pool handle组合index、generation和phantom type，pool负责验证。Zircon现有`ResourceHandle<T>`和部分typed RHI handle方向一致，但类型参数只能阻止跨元素类型误用，不能识别两个World、两个Device或两次owner重建。owner qualification仍是独立问题。

### 4.4 Godot：RID owner与ObjectID是不同注册域

Godot的RID owner把索引与validator信息组合并由owner检查，ObjectID则有非零值和对象注册语义。应吸收的是registry domain与验证责任，而不是复制特定位布局。Zircon必须先定义哪些身份属于resource/device/object/session，再决定内部压缩格式。

## 5. Owner裁决与非重复边界

| Owner | 本篇拥有 | 邻接报告继续拥有 |
|---|---|---|
| Identity Schema | Persistent、Live、Scoped Live、Sequence、Revision分类与codec eligibility | Runtime04拥有asset load/residency与artifact生命周期 |
| Owner Qualification | World/Session/Device/Registry epoch、kind/domain验证和stale taxonomy | Runtime05、Interface01、Runtime09A实现各自owner lifecycle |
| Allocator Contract | 0/invalid、collision、checked increment、slot retirement、exhaustion结果 | Tooling22管理常量placement，Tooling23管理全局panic/error治理 |
| Persistence Boundary | stable ID算法版本、ephemeral handle禁序列化、stable-to-live remap | Runtime Interface02拥有公共DTO/schema migration大盘 |
| Handle Semantics | strong/weak/lease/borrow、owned/borrowed、lookup lifetime | Runtime01/07拥有service/script/plugin具体卸载quiescence |
| Qualification Evidence | stale/wrong owner/exhausted指标、near-limit测试和跨运行fixture | Tooling07/10拥有通用benchmark与测试基础设施 |

Runtime05的`EntityId` finding、Interface01的session handle回绕、Runtime08F的AI stale state和Runtime09A的device generation仍以原报告为局部canonical owner。本篇登记的是共享合同缺失、跨owner不一致以及稳定UUID/Scene load等尚未被邻接报告精确拥有的事实。

## 6. P1：Persistent Identity、Schema 与 Serialization Eligibility

| ID | 当前差距 | 需要重构 |
|---|---|---|
| IDENTITY-P1-001 | 引擎没有区分Persistent ID、Live Handle、Scoped Live Handle、Sequence和Revision的公共taxonomy | 建`IdentityKind`与设计/代码审查规则，每个public identity声明kind、owner、lifetime、codec和exhaustion policy |
| IDENTITY-P1-002 | `stable_uuid_from_components`使用未承诺长期稳定格式的`DefaultHasher`生成持久UUID | 选择versioned UUIDv5或BLAKE3/SHA-256截断方案，定义namespace、byte order、domain separation和测试向量 |
| IDENTITY-P1-003 | stable UUID算法、namespace与component编码没有schema version或BuildSet identity | 发布`StableIdAlgorithmId`和canonical length-prefixed encoding，把digest纳入artifact/save/cache兼容性 |
| IDENTITY-P1-004 | ResourceId/AssetUuid切换算法没有legacy alias、双读或迁移receipt | 建old->new alias table、离线扫描/重写工具与碰撞检测；hard cutover后禁止继续新建legacy ID |
| IDENTITY-P1-005 | Scene `EntityId = u64`同时承担保存引用、authoring identity和运行期live lookup | Runtime05实现持久`SceneObjectId`与live `EntityHandle`分层，本篇要求两者codec和remap职责不可混合 |
| IDENTITY-P1-006 | `InternalEntity`公开且可序列化，但仅包含index/generation | 将其限制为non-serializable owner-local handle，或序列化时强制携`WorldInstanceId/Epoch`并明确仅用于诊断 |
| IDENTITY-P1-007 | `WorldHandle(u64)`可序列化，却没有project/world-instance/replacement epoch | 区分持久World asset ID、运行中World instance handle和replacement epoch，禁止互相透明转换 |
| IDENTITY-P1-008 | 多个RHI/runtime handle derive serde，没有声明重启、device重建或跨process后是否有效 | 默认deny ephemeral-handle serialization，仅为明确的external/stable descriptor提供versioned codec |
| IDENTITY-P1-009 | sound external IDs、device names、parameter/key字符串等身份缺少规范化与schema owner | 为每类外部identity定义normalization、case/Unicode、namespace、collision和unknown-value policy |
| IDENTITY-P1-010 | 没有可机器检查的identity manifest，review只能靠搜索newtype和字段名发现误用 | 由derive/schema registry生成kind/owner/codec metadata，CI拒绝未分类的public handle与ephemeral serde |

## 7. P1：Live Handle、Owner Epoch 与 Stale Reference

| ID | 当前差距 | 需要重构 |
|---|---|---|
| IDENTITY-P1-011 | 全引擎没有统一`LiveObjectKey`或等价owner-qualified表示，局部代际句柄不能组合 | 建`OwnerKey { domain, instance, epoch } + SlotKey { index, generation }`的逻辑模型，物理布局可按边界优化 |
| IDENTITY-P1-012 | World replacement epoch只在LevelSystem若干异步writer局部携带，不是所有Entity/Watch/Query输入的必需上下文 | World API在resolve时验证owner/replacement epoch；异步计划、cache和receipt统一携同一快照 |
| IDENTITY-P1-013 | ABI Session/Operation/Subscription等token多为透明`u64`，kind和owner依赖调用约定 | Interface01实现ABI布局/registry；本篇要求所有lookup至少验证kind、session owner和registry epoch并返回可诊断错误 |
| IDENTITY-P1-014 | per-session operation/subscription handle自身不携session，错配只能在外部参数组合后发现 | 使用scoped handle或registry-side owner binding，错误明确区分unknown、wrong-session、stale和already-complete |
| IDENTITY-P1-015 | AI等状态虽用`(WorldHandle, EntityId)`，但相同level内World替换仍可复用key | Runtime08F清理局部状态；公共World identity必须含replacement epoch并让旧key无法命中新World |
| IDENTITY-P1-016 | typed RHI handle能区分Buffer/Texture，却不能区分两个Device或device-loss前后 | Runtime09A增加DeviceEpoch和retirement；lookup/submit拒绝wrong-device/stale-generation而非偶然命中相同raw value |
| IDENTITY-P1-017 | service、plugin、script、font与render graph各自定义不同generation/retirement语义 | 建共享policy trait与conformance测试，允许不同位宽但必须声明wrap、retire、owner reset和lookup结果 |
| IDENTITY-P1-018 | `InternalEntity`在同一registry内安全，但不同World可产生相同index/generation bits | 所有公开跨owner路径使用`WorldEntityHandle`；owner-local fast path保持紧凑但不得泄漏为全局身份 |
| IDENTITY-P1-019 | WatchKey/WorldFact/Invalidation DTO携raw EntityId或单独WorldGeneration，缺少不可拆分的world snapshot identity | 建`WorldSnapshotKey`并在订阅建立、invalidate、readback和恢复时验证epoch/generation组合 |
| IDENTITY-P1-020 | stale处理散落为`None`、bool、局部enum、panic或silent overwrite | 统一`HandleResolveError { WrongKind, WrongOwner, Stale, Retired, Exhausted, Unknown }`及ABI稳定映射 |

## 8. P1：Allocation、Generation、Overflow 与 Exhaustion

| ID | 当前差距 | 需要重构 |
|---|---|---|
| IDENTITY-P1-021 | Scene load以`max(EntityId) + 1`重建分配器，`u64::MAX`会panic或回绕为0 | load admission检查reserved/max值，用checked allocator恢复并返回带文档路径的`EntityIdExhausted` |
| IDENTITY-P1-022 | 直接`spawn_node`仍使用普通加法，而deferred/bundle/transaction部分路径已返回`EntityIdExhausted` | 所有spawn入口共享单一fallible allocator；不允许infallible facade绕过容量与reserved-value检查 |
| IDENTITY-P1-023 | EntityRegistry新增slot用`slots.len() as u32`，超宽度时静默截断 | 使用`u32::try_from`并传播`SlotIndexExhausted`，同时给owner配置容量预算与测试注入小位宽 |
| IDENTITY-P1-024 | Entity slot generation在`u32::MAX`后回到首代，极老handle理论上可重新有效 | generation耗尽时永久retire slot、扩宽generation或轮换owner epoch；禁止无证据回绕复用 |
| IDENTITY-P1-025 | World/lifecycle/dynamic-component/event等generation使用saturating add，达到MAX后冻结 | revision allocator返回Exhausted或轮换epoch；需要单调语义的状态不能以饱和值继续运行 |
| IDENTITY-P1-026 | archetype membership generation使用wrapping add，完整回绕后旧query plan可能重新匹配 | 用epoch+revision、checked rollover或cache全失效barrier，并提供可缩小位宽的wrap model test |
| IDENTITY-P1-027 | message ID耗尽通过`expect`触发panic，clear cursor generation又可能饱和冻结 | Message write变为fallible/budgeted admission，cursor generation rollover必须使旧reader确定失效 |
| IDENTITY-P1-028 | Observer ID普通加一且无0、回绕、collision处理，clone又重置store | 使用owner-scoped subscription allocator；clone/rebuild发布新owner epoch，remove/notify返回typed stale结果 |
| IDENTITY-P1-029 | Level/session/network等atomic allocator使用`fetch_add`后直接insert，回绕可产生0或覆盖live entry | 采用checked CAS或collision-probing registry，接近上限时拒绝admission并发布exhaustion telemetry |
| IDENTITY-P1-030 | particle在MAX后复用同一handle并覆盖live instance；sound/navigation/AI/UI/RHI等也存在普通或饱和递增 | 统一替换为声明式allocator policy；绝不允许饱和后继续insert，owner选择retire、epoch rollover或显式失败 |

## 9. P1：Cross-System Contract、Lifecycle 与 Qualification

| ID | 当前差距 | 需要重构 |
|---|---|---|
| IDENTITY-P1-031 | 没有中央Identity Domain registry，World/Device/Session/Plugin owner独立发号且无法检查跨域误用 | 建`IdentityDomainId`与owner registry，domain静态生成、实例/epoch动态分配并进入debug/receipt |
| IDENTITY-P1-032 | 分配器没有公共capability：是否reuse、是否generational、最大live数、耗尽行为未知 | 每个allocator声明`AllocatorContract`，启动时注册并供schema、diagnostics和qualification读取 |
| IDENTITY-P1-033 | owner teardown通常清Map即可，未统一发布invalidated range/epoch与完成receipt | teardown先停止新admission，再轮换/终结epoch、等待lease、发布invalidated/retired计数和最终receipt |
| IDENTITY-P1-034 | 多数handle不区分strong、weak、lease、borrow和owning token | public API以类型表达lifetime/ownership；weak resolve可失败，lease阻止teardown，borrow受调用scope限制 |
| IDENTITY-P1-035 | ephemeral handle可被serde/JSON/日志/远程调用随意携出owner lifetime | schema gate默认拒绝；诊断输出使用`DebugIdentity`并标注domain/epoch，不提供可重新resolve的伪持久值 |
| IDENTITY-P1-036 | scene/prefab/restore只靠raw ID重建，没有统一persistent-to-live remap authority | load transaction先分配live owner/slots，再建立stable remap、修复引用、验证闭包，最后原子publish |
| IDENTITY-P1-037 | sequence/correlation、revision和object identity经常都叫`id/generation/handle` | 使用独立类型与命名：Sequence允许有界回绕但需窗口；Revision用于比较；Handle必须resolve；PersistentId可跨运行 |
| IDENTITY-P1-038 | 外部client/principal/connection/content identity没有统一trust与forgeability边界 | 区分trusted internal handle与untrusted external ID；所有wire token做namespace、authorization、replay和rate-limit检查 |
| IDENTITY-P1-039 | 没有跨allocator的near-exhaustion、wrong-owner、wrap、stale、double-free和rebuild模型测试 | 提供小位宽test allocator、property/fuzz/state-machine suite，并对所有注册owner运行同一conformance矩阵 |
| IDENTITY-P1-040 | 没有live/free/retired slot、generation high-water、stale/wrong-owner/exhausted resolve指标 | 发布低cardinality owner-domain metrics、trace receipt和support dump；不得把raw高基数handle当metric label |

## 10. P2：后续能力

| ID | 能力 | 前置条件 |
|---|---|---|
| IDENTITY-P2-001 | 对不可信远程控制面提供128-bit随机或MAC capability token | P1 domain/owner/auth contract完成，不能用更长整数掩盖错误owner |
| IDENTITY-P2-002 | 在保持逻辑schema的前提下为热路径设计niche/packed handle布局 | 有profile证明handle密度或cache miss为瓶颈，并保留debug全字段解码 |
| IDENTITY-P2-003 | 分片/sharded allocator与thread-local slot cache | 单owner allocator成为并发热点且回收/epoch语义已有形式化测试 |
| IDENTITY-P2-004 | Handle sanitizer调试模式：隔离区、延迟复用、随机generation与resolve backtrace | P1统一registry seam与可控内存预算完成 |
| IDENTITY-P2-005 | Editor identity inspector显示stable/live映射、owner epoch、leases和stale原因 | runtime提供只读snapshot与脱敏diagnostic contract |
| IDENTITY-P2-006 | 旧工程稳定UUID迁移预览、冲突修复和引用影响图 | P1算法版本、alias表和离线migration receipt完成 |
| IDENTITY-P2-007 | 跨进程/分布式对象引用服务与location-independent soft reference | 单机stable/live分层、authority与安全模型先闭环 |
| IDENTITY-P2-008 | 需要全局时间排序时引入ULID/Snowflake类event ID | 明确时钟、节点、隐私和回绕需求；普通runtime handle不得因此膨胀 |
| IDENTITY-P2-009 | 对slot lifecycle和epoch rollover做TLA+/模型检查 | P1状态机与失败语义固定，模型与生产conformance测试共享transition名 |
| IDENTITY-P2-010 | debugger pretty-printer与crash dump符号化owner/slot/generation | identity manifest和domain registry可随BuildSet发布 |
| IDENTITY-P2-011 | 跨语言SDK生成typed handle wrapper和ownership注解 | ABI handle version、错误映射和language binding policy稳定 |
| IDENTITY-P2-012 | identity qualification dashboard追踪长时soak的high-water与stale率 | 指标schema、test artifact currentness和产品规模baseline完成 |

## 11. 目标架构

### 11.1 Identity Schema Registry

建立生成式`IdentitySchemaRegistry`，每个identity type至少声明：`type_id`、`kind`、`domain`、`owner_kind`、`invalid_value`、`codec_policy`、`algorithm/version`、`allocator_policy`与`resolve_error_map`。Rust derive、ABI schema、serialization registry、文档和CI从同一manifest生成，避免类型名与真实语义漂移。

建议逻辑分类如下；这不是要求所有值拥有相同物理宽度。

| Kind | 典型例子 | 必需属性 |
|---|---|---|
| Persistent | Asset UUID、SceneObjectId、package/content ID | 跨运行稳定、算法版本、namespace、迁移与collision contract |
| Live | owner内slot+generation | 不持久化、resolve验证generation、free/retire规则 |
| Scoped Live | WorldEntity、DeviceResource、SessionOperation | Live全部属性，加owner instance/epoch验证 |
| Sequence/Correlation | message sequence、request correlation | window/order/wrap规则，不可resolve成对象 |
| Revision/Epoch | world generation、catalog revision、owner epoch | 比较/rollover/失效范围，不能当对象handle |

### 11.2 Owner 与 allocator

公共模型使用`OwnerKey { domain_id, instance_id, epoch }`和`SlotKey { index, generation }`。热路径可把owner存在registry或调用上下文中、将slot压缩到64位；但每次跨owner、跨thread长期缓存、ABI或持久化边界都必须恢复并验证完整语义。

allocator只有三种合法的末端行为：返回`Exhausted`并拒绝新建；永久retire耗尽slot后使用其他slot；在受控barrier轮换owner epoch并使全部旧handle失效。普通回绕、饱和后继续insert、覆盖live entry和panic都不属于合法策略。0等reserved值必须由类型或构造器保证不可生成。

### 11.3 Resolve、ownership 与 teardown

`resolve`先验证kind/domain，再验证owner instance/epoch，最后验证slot/generation和对象状态；错误保留`WrongKind/WrongOwner/Stale/Retired/Unknown`差异。strong handle延长对象寿命，weak handle只允许fallible resolve，lease阻止owner完成teardown，borrowed handle不能逃逸调用scope。owner teardown遵守`close admission -> cancel/drain -> await leases -> retire epoch -> receipt`。

### 11.4 Persistence 与 remap

serializer默认只接受`codec_policy = Persistent`。World/Scene/Prefab载入在隔离transaction中验证schema与ID范围、分配新live owner、建立persistent-to-live remap、修复引用闭包、报告dangling/duplicate/collision，最后原子publish。运行期handle出现在save、artifact或wire schema中必须成为编译/验证错误，而不是“反序列化后碰巧还能查到”。

### 11.5 Diagnostics 与 evidence

每个owner发布容量、live/free/retired、高水位、epoch rollover、stale/wrong-owner/unknown resolve和exhaustion计数；trace可记录脱敏`domain/owner epoch/slot/generation`，metrics不使用raw handle作为label。qualification artifact包含manifest digest、allocator config、seed、operation trace和最终invariant结果。

## 12. Hard Cutover 与迁移规则

1. 先冻结新增未分类public identity；inventory生成owner表，并为74个数值wrapper逐项标kind、owner和codec。
2. 引入stable UUID v2算法与固定测试向量，但先双读legacy/v2；扫描全部save、scene、asset metadata和cache引用，生成alias与collision报告。
3. 新写路径只生成v2，旧值读取后通过明确migration transaction升级；达到产品迁移门槛后删除legacy生成器，不永久保留双写。
4. 将Scene persistent object ID、World instance、live Entity handle分层；先改内部API和异步cache，再迁移Editor/ABI consumer。
5. 统一allocator/result；所有普通/饱和/wrapping handle分配点必须迁移或登记为有窗口证明的Sequence/Revision。
6. serializer和wire schema启用ephemeral deny gate；必要的兼容字段通过versioned stable reference替换，禁止继续写raw live bits。
7. ABI通过版本升级增加kind/owner validation与稳定错误码；旧API在兼容窗口内adapter到新registry，最终hard cutover移除无owner token。
8. 删除旧helper、legacy serde与旁路counter前，必须通过reference closure、wrong-owner、near-exhaustion、restart和migration资格矩阵。

## 13. 里程碑

| Milestone | 交付 | 验收 |
|---|---|---|
| M0 Inventory | identity manifest、owner地图、kind/codec/allocator分类、canonical finding路由 | public identity 100%分类，未分类或ephemeral serde使CI失败 |
| M1 Stable ID | stable UUID v2、测试向量、alias/collision scanner、migration receipt | 跨toolchain/平台固定向量一致；legacy fixture引用闭包无静默丢失 |
| M2 Allocator Kernel | checked/generational allocator、slot retirement、owner epoch、typed errors | 小位宽穷举覆盖0/MAX/reuse/wrap/double-free/wrong-owner |
| M3 Scene/World | SceneObjectId、WorldInstanceId、WorldEntityHandle、load remap与replacement epoch | clone/load/replace/undo/async stale测试都不能误命中新对象 |
| M4 Runtime Owners | service/script/plugin/message/observer/AI/audio/nav/net/UI/RHI采用统一policy | owner conformance矩阵通过；无panic、饱和覆盖或未声明回绕 |
| M5 ABI/Persistence | scoped ABI token、ephemeral deny、schema migration与SDK wrapper | restart/device loss/session teardown/foreign misuse返回稳定错误而非UB或误命中 |
| M6 Qualification | soak、fuzz/model、容量/性能baseline、support artifact | 产品规模下无stale acceptance、live overwrite和未解释ID变化，overhead满足预算 |

## 14. 验证矩阵

| 维度 | 场景 | 失败门槛 |
|---|---|---|
| Stable algorithm | 多Rust/toolchain、Windows/Linux、component边界、legacy/v2 fixture | 同一v2输入不同值，或legacy迁移无alias/receipt |
| Scene admission | duplicate、0、`u64::MAX`、dangling reference、malformed generation | panic、wrap、默认节点占用reserved ID、部分publish |
| Slot lifecycle | allocate/free/reuse、generation max、owner epoch rollover、double free | 任何旧handle重新resolve到不同对象 |
| Cross-owner | 两World、两Device、两Session产生相同slot bits并交叉调用 | wrong-owner被接受或只返回模糊unknown |
| Teardown | outstanding strong/weak/lease/operation下关闭owner | use-after-free、无限等待、epoch未退休或无receipt |
| Messages/events | cursor clear、retention drop、sequence窗口、近耗尽 | panic、旧reader被当新reader、顺序无定义 |
| Persistence | save/load/restart/cook/cache/remote schema扫描 | ephemeral handle可写入，或live bits重启后碰巧命中 |
| Concurrency | allocate/free/resolve与owner reset并发 | ABA、double allocation、锁内回调、generation复核缺失 |
| Performance | handle resolve吞吐、registry内存、contention、teardown latency | 无baseline声称优于参考引擎，或安全检查被默认关闭 |
| Soak | 长时World replace、device loss、plugin reload、session churn | high-water不受控、stale率上升、epoch/generation达到阈值无预警 |

## 15. 实施约束与最终判断

- 不建立一个跨所有域的全局万能ObjectId。持久资源、World entity、GPU resource、session token和message sequence有不同信任、生命周期与性能边界。
- 不机械把所有`u64`升级为UUID或128位。宽度不提供owner、generation、codec与耗尽语义；热路径布局应由profile决定。
- 不把理论上的2^32/2^64回绕直接描述成已发生事故，但必须在工程合同中选择retire、epoch rollover或failure，不能依赖“正常不会跑到”。
- 不因Bevy也允许generation理论回绕就接受Zircon的无策略回绕；参考实现明确了适用域，Zircon目标是更强的长期产品资格。
- 不重复Runtime05、Interface01、Runtime09A的局部实现owner；共享schema/allocator先落地，局部报告再按其生命周期完成hard cutover。
- 不以新增wrapper数量作为完成标准。验收必须证明跨owner拒绝、stale不误命中、stable算法跨运行一致、ephemeral不可持久化、耗尽不覆盖live对象。

最终判断：Zircon已经拥有数套质量不错的局部身份机制，特别是脚本HostRegistry、Render Graph generation、字体registry和root lease；这说明工程化目标无需从零开始。当前阻碍是这些机制没有收敛为引擎级identity/owner/allocator contract，且稳定UUID和Scene载入存在真实持久边界缺陷。正确顺序是先分类与冻结新债务，再完成稳定ID迁移和共享allocator/owner epoch，随后改造Scene与跨runtime owner，最后关闭ABI/persistence旁路并以近耗尽、跨owner、restart与soak证据验收。
