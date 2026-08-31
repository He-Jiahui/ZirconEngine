---
title: Runtime Stable Identity / Handle / Generation / Owner Epoch / Stale Reference / Exhaustion Current Source Review
category: zircon_runtime
report_id: Runtime159
review_date: 2026-08-29
baseline_head: b2e76ff33cc298ad76f7b801a1d06d1e2faa046d
canonical_owner: Runtime24
refreshes:
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
related_code:
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/resource
  - zircon_runtime_interface/src/project
  - zircon_runtime_interface/src/runtime_api/session
  - zircon_runtime_interface/src/world_sync
  - zircon_runtime/src/scene
  - zircon_runtime/src/core/framework/scene
  - zircon_runtime/src/core/runtime/handle
  - zircon_runtime/src/core/runtime/state
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/src/render_graph
  - zircon_runtime/crates/zr_rhi/src
  - zircon_runtime/crates/zr_rhi_wgpu/src
  - zircon_runtime/src/script/vm
  - zircon_runtime/src/graphics/scene
  - zircon_runtime/src/ui
  - zircon_app/src/entry/runtime_library
  - zircon_editor/src/core/gateway
  - zircon_editor/src/core/sync
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/core/play
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/WeakObjectPtr.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/ObjectHandle.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/WeakObjectPtr.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Tests/ObjectHandleTest.cpp
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/Fyrox/fyrox-core/src/pool/handle.rs
  - dev/Fyrox/fyrox-core/src/pool/mod.rs
  - dev/godot/core/templates/rid_owner.h
  - dev/godot/core/object/object_id.h
  - dev/godot/tests/core/templates/test_rid.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResources.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourcePool.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_identity_contract_incomplete
source_recheck_required: true
---

# 159 · Runtime Stable Identity、Handle、Generation、Owner Epoch、Stale Reference 与 Exhaustion 当前源码复核

## 1. 结论

Runtime24指出的问题并没有被整体关闭，但当前工作树已经补上若干重要底座。稳定UUID已从`DefaultHasher`迁到带算法版本、domain separation、长度前缀和固定向量的BLAKE3/UUIDv8编码；Scene实体分配器、ECS slot generation、session handle和Level handle已改为可失败或永久退休；RHI handle已经携带namespace、device identity、device generation、resource kind、slot与slot generation，并明确拒绝反序列化；ServiceHandle也具备generation验证和调用lease。

这些进展仍是局部owner实现，不是全引擎身份体系。`EntityId = u64`继续同时承担持久引用、authoring identity和live lookup，`WorldHandle(u64)`仍可序列化且没有project/world-instance/replacement epoch，公开ABI token仍依赖外部调用约定，World revision、archetype membership、message、observer和UI timeline等仍存在饱和、回绕、panic或别名复用。项目身份、session identity、world snapshot、device identity和service lease也没有收敛为统一的逻辑taxonomy、错误模型、manifest、teardown receipt与资格测试。

本轮对Runtime24的40项P1重判为：**8 Closed、17 Partial、15 Open**；12项P2仍为**12 Open**。不新增唯一finding，因此portfolio计数不变。24项资格门为**10 Fail、9 Partial、5 Pass**。该结论只代表2026-08-29当前工作树；选择集中有1,346个tracked修改和359个untracked文件，`Closed`不代表能力已进入干净checkout、CI或已发布BuildSet。

## 2. 审查范围与证据强度

### 2.1 Zircon选择集

| 指标 | 数值 |
|---|---:|
| 词法扫描文件 | 1,910 |
| 总行数 | 290,258 |
| bytes | 10,352,311 |
| test属性/测试信号 | 2,622 |
| ignored | 0 |
| tracked modified | 1,346 |
| untracked | 359 |
| public-ish identity声明 | 72 |
| numeric tuple wrapper | 42 |
| serde handle候选文件 | 7 |
| 选择集指纹 | `2c4f248732c804fc05b6b2c9fc1bab6dc8ac6ce882bdb281e90c504b4a2f65b9` |

扫描覆盖Interface resource/project/session/world-sync，Runtime Scene/ECS/World/Core service/event/dynamic session/operation/render graph/RHI/script/graphics/UI，App runtime library以及Editor gateway/sync/play消费者。统计是选择集的词法覆盖；本文对下列关键owner和测试进行了逐文件、逐分支复核，不把纯命中数量当作功能完成证据。

### 2.2 参考选择集

| 指标 | 数值 |
|---|---:|
| 文件 | 13 |
| 行数 | 9,493 |
| bytes | 349,455 |
| test信号 | 66 |
| 指纹 | `63048d941d4e07062fda2fcf97fa7d8c2580b65ad68bfaa5b83cdc2e2bdace49` |

Unreal用于检查弱引用、对象serial、resolved/unresolved handle与stale语义；Bevy和Fyrox用于检查Rust ECS/pool的index-generation生命周期；Godot用于检查RID owner、validator、capacity/free/leak诊断；Unity Graphics用于检查RenderGraph resource type/index/version、registry写版本和resource pool释放/泄漏。参考实现提供约束与失败语义，不代表Zircon应复制其位布局或全局对象模型。

## 3. 身份类型盘点与当前裁决

| 身份类别 | 当前代表 | 当前性质 | 裁决 |
|---|---|---|---|
| Persistent resource identity | `ResourceId`、`AssetUuid`、stable UUID | 可跨运行；算法已版本化 | 保留算法底座，补legacy catalog、artifact/BuildSet绑定 |
| Project identity | `ProjectGuid`、`ProjectManifestDigest`、`ProjectIdentity` | schema较强，生产admission消费不足 | 不得退回路径或字符串；接入session/world/artifact入口 |
| Persistent scene identity | `EntityId(u64)` | 与authoring/live lookup混用 | 必须拆成`SceneObjectId`与live handle/remap |
| Live world identity | `WorldHandle(u64)` | serde，缺owner epoch | hard cut为project-qualified world instance handle |
| Owner-local ECS handle | `InternalEntity { index, generation }` | `pub(crate)`、non-serde、World-local | 当前边界可保留，不得重新公开泄漏 |
| Device resource handle | RHI typed handles | namespace/device/device-generation/kind/slot-generation | 当前结构可保留，补allocator namespace exhaustion和全矩阵测试 |
| Runtime service handle | `ServiceHandle`、`ServiceCallGuard` | generation-bound resolve与lease | 保留，修复service generation wrap并纳入统一错误/teardown receipt |
| ABI scoped token | session/operation/subscription/watch token | 多为transparent `u64` | 需要registry-side owner binding、epoch和稳定错误映射 |
| Sequence/correlation | message、observer、operation ID | 命名和耗尽规则不一致 | 与object identity/revision分型，声明窗口与回绕规则 |
| Revision/generation | World、lifecycle、component、archetype revision | saturation/wrapping混用 | 必须checked rollover、epoch切换或显式exhaustion |
| External identity | principal/connection/device/parameter/key字符串 | normalization/trust分散 | 每域声明namespace、normalization、authorization与replay policy |
| Diagnostic identity | serialized RHI raw、日志中的handle | 有些已diagnostic-only | 统一`DebugIdentity`，绝不提供跨owner重新resolve承诺 |

全仓没有找到可作为公共合同的`IdentityKind`、`IdentityDomainId`、`AllocatorContract`、`LiveObjectKey`、`OwnerKey`、`WorldSnapshotKey`或`IdentityManifest`。Editor的`GatewaySessionIdentity`和`QualifiedWatchToken`、RHI的`ResourceHandleIdentity`及Core的ServiceHandle只是三个独立局部模型。

## 4. 已证实可保留的底座

### 4.1 Stable UUID

`zircon_runtime_interface/src/resource/stable_uuid.rs`现在定义`STABLE_UUID_ALGORITHM_VERSION = 1`，使用BLAKE3 derive-key、component count与长度前缀，设置UUID v8和RFC variant，并有固定跨平台测试向量。原P1-002关闭。仍未找到legacy `DefaultHasher` ID catalog、old-to-new alias、碰撞报告、tombstone或工程迁移receipt；算法版本也尚未绑定artifact/save/cache兼容性和BuildSet，因此P1-003仅Partial、P1-004保持Open。

### 4.2 Scene实体分配与ECS slot

`EntityIdAllocator`把0和`u64::MAX`设为reserved，通过`reserve_next`、`advance_past`和checked successor统一bootstrap、deferred、record、transaction、bundle、direct spawn和load恢复。`EntityRegistry`使用`u32::try_from`，返回`SlotCapacityExhausted`；slot generation耗尽时永久退休，并有capacity、retirement和stale测试。原P1-006、P1-021至P1-024关闭。

但外部Scene identity仍是裸`EntityId`；没有`SceneObjectId`/`EntityHandle`/`WorldEntityHandle`分层。World generation、lifecycle visibility revision、dynamic component generation继续`saturating_add`，archetype membership继续`wrapping_add`，所以不能把局部allocator修复外推成完整World identity闭环。

### 4.3 RHI owner-qualified handle

`zr_rhi`的`ResourceHandleIdentity`携带allocator namespace、`DeviceId`、`DeviceGeneration`、resource kind、slot与slot generation；resolve能区分wrong device、wrong generation、foreign allocator、wrong kind和stale。slot宽度和generation耗尽均有显式处理，句柄序列化只允许diagnostic raw value且反序列化固定拒绝。原P1-016关闭。

剩余缺口是`NEXT_RESOURCE_NAMESPACE.fetch_add(1)`仍可能回绕，foreign allocator/wrong kind/retirement/serde等未形成完整conformance矩阵，Runtime其他handle也没有采用同等边界。因此P1-008、P1-017、P1-039只是Partial。

### 4.4 Session、Level、Service与Script局部政策

- Dynamic session registry使用checked next handle，0代表永久耗尽，并返回`HandleSpaceExhausted`。
- Level handle通过checked `fetch_update`分配，达到上限后返回`LevelHandleExhausted`。
- Plugin event subscription与部分operation sequence保留headroom并有边界测试。
- `ServiceHandle`验证Core owner、name/index/generation；`ServiceCallGuard`在调用期间持有lease并参与drain。
- Script host/export/hot reload registry具有slot/generation exhaustion与stale-after-reuse测试。
- Editor的`GatewaySessionIdentity`组合runtime instance/session、gateway generation、transport epoch、project和play instance；`QualifiedWatchToken`拒绝旧gateway与token collision。

这些实现证明owner-qualified handle和fallible exhaustion可行，但公共ABI自身仍是raw `u64`，Level/World公开key仍裸，service generation仍`wrapping_add`，Editor wrapper也不能修复wire DTO。因此P1-011至P1-015、P1-017、P1-019、P1-033、P1-034均只能Partial。

## 5. Runtime24 P1重分类

| ID | 状态 | 当前源码裁决 |
|---|---|---|
| IDENTITY-P1-001 | Open | 仍无公共identity taxonomy和强制分类规则 |
| IDENTITY-P1-002 | Closed | BLAKE3/UUIDv8、版本、domain separation、长度前缀和固定向量已实现 |
| IDENTITY-P1-003 | Partial | 算法版本和canonical encoding已实现；未绑定BuildSet/artifact/save兼容性 |
| IDENTITY-P1-004 | Open | 未找到legacy alias、迁移catalog、collision/tombstone/receipt |
| IDENTITY-P1-005 | Open | `EntityId(u64)`仍混合persistent、authoring和live lookup |
| IDENTITY-P1-006 | Closed | `InternalEntity`已限制为`pub(crate)`、non-serde、World-local |
| IDENTITY-P1-007 | Open | `WorldHandle(u64)`仍serde且缺project/instance/replacement epoch |
| IDENTITY-P1-008 | Partial | RHI已deny反序列化；World/ABI等ephemeral handle尚无统一schema gate |
| IDENTITY-P1-009 | Open | 外部字符串/设备/parameter/key identity仍无统一normalization schema |
| IDENTITY-P1-010 | Open | 无机器可检查的identity manifest或CI admission |
| IDENTITY-P1-011 | Partial | RHI、Editor、Service各有qualified模型；无公共逻辑模型 |
| IDENTITY-P1-012 | Partial | Level/Editor若干路径验证replacement/gateway epoch；World API未强制 |
| IDENTITY-P1-013 | Partial | registry局部验证增强；公开ABI token仍是transparent `u64` |
| IDENTITY-P1-014 | Partial | per-session registry能侧面约束；token本身无session/registry epoch |
| IDENTITY-P1-015 | Partial | Level存在replacement generation；AI/public world-entity key仍可跨替换别名 |
| IDENTITY-P1-016 | Closed | RHI handle已验证device、device generation、allocator、kind与slot generation |
| IDENTITY-P1-017 | Partial | RHI/Service/Script各有政策；仍无共享policy contract和全owner矩阵 |
| IDENTITY-P1-018 | Closed | owner-local `InternalEntity`不再作为公开跨World身份 |
| IDENTITY-P1-019 | Partial | Editor QualifiedWatchToken补gateway identity；wire WatchKey/WorldFact仍裸 |
| IDENTITY-P1-020 | Partial | RHI等已有typed error；全引擎仍混用`None`、bool、panic与局部enum |
| IDENTITY-P1-021 | Closed | load admission拒绝reserved/max并通过checked allocator恢复 |
| IDENTITY-P1-022 | Closed | direct/deferred/bundle/transaction/load已共享fallible EntityId allocator |
| IDENTITY-P1-023 | Closed | slot index改用checked conversion并返回capacity exhausted |
| IDENTITY-P1-024 | Closed | slot generation耗尽永久retire，已有stale/retirement测试 |
| IDENTITY-P1-025 | Open | World/lifecycle/component/event revision仍可在MAX饱和冻结 |
| IDENTITY-P1-026 | Open | archetype membership仍wrapping，旧plan理论上可重获有效性 |
| IDENTITY-P1-027 | Open | message ID exhaustion仍`expect` panic，clear generation仍saturating |
| IDENTITY-P1-028 | Open | observer普通递增，clone重置owner空间，缺stale/epoch/collision合同 |
| IDENTITY-P1-029 | Partial | Level/session已checked；仍有其他atomic/global allocator回绕风险 |
| IDENTITY-P1-030 | Open | UI timeline、UI subscription等仍饱和或普通递增并可复用/覆盖 |
| IDENTITY-P1-031 | Open | 无中央identity domain/owner registry |
| IDENTITY-P1-032 | Open | 无可查询的allocator capability/contract |
| IDENTITY-P1-033 | Partial | Service/session有drain/teardown；无统一epoch invalidation和receipt |
| IDENTITY-P1-034 | Partial | ServiceCallGuard表达lease；多数public handle仍不区分lifetime形态 |
| IDENTITY-P1-035 | Partial | RHI已diagnostic-only；其他serde/JSON/wire handle仍可逃逸owner lifetime |
| IDENTITY-P1-036 | Partial | Scene load已有统一分配事务底座；无canonical persistent-to-live remap authority |
| IDENTITY-P1-037 | Open | sequence/revision/object identity仍普遍混称id/generation/handle |
| IDENTITY-P1-038 | Open | principal/connection/content token没有统一forgeability/trust/replay边界 |
| IDENTITY-P1-039 | Partial | Scene/RHI/session/script有局部边界测试；无跨allocator conformance suite |
| IDENTITY-P1-040 | Partial | 局部owner有错误/计数；无统一high-water/stale/wrong-owner/exhaustion指标 |

状态合计：Closed 8、Partial 17、Open 15。任何后续实现都应回写Runtime24的canonical finding，不在Runtime159创建平行编号。

## 6. P2状态

| 范围 | 状态 | 原因 |
|---|---|---|
| P2-001 capability token、P2-007 distributed reference、P2-011 SDK typed wrapper | Open | domain/owner/auth/ABI稳定合同未完成 |
| P2-002 packed handle、P2-003 sharded allocator | Open | 尚无证明identity布局或allocator竞争是产品瓶颈的profile |
| P2-004 handle sanitizer、P2-009 TLA+ model | Open | 统一状态机、transition名与allocator seam未固定 |
| P2-005 identity inspector、P2-010 debugger symbols、P2-012 dashboard | Open | identity manifest、diagnostic snapshot和metrics schema未完成 |
| P2-006 UUID migration preview | Open | legacy alias/catalog/receipt未完成 |
| P2-008 ULID/Snowflake event identity | Open | 无跨节点全序需求证据，不能用更宽ID掩盖owner问题 |

## 7. 关键未闭合问题

### 7.1 Persistent ID与Live Handle仍是同一个数

Scene保存、Editor authoring、World runtime lookup和异步系统都围绕裸`EntityId`运转。即便实体数值分配不再回绕，它也没有表达“属于哪个project、scene asset、World instance和replacement epoch”。工程化修复不是给`u64`继续加注释，而是让persistent reference经过load transaction映射到owner-qualified live handle；保存只写persistent ID，运行时cache只持live handle，诊断输出两者和映射generation。

### 7.2 World replacement没有成为不可拆分的owner identity

`WorldHandle`、WatchKey、WorldFact、AI state key与许多异步receipt仍能只携World/Entity裸值。Level和Editor局部generation guard能拦住部分旧结果，却无法保证所有新API都验证同一快照。目标合同必须让`WorldSnapshotKey`或等价owner key成为resolve/query/watch/apply的必需输入，而不是可选旁路字段。

### 7.3 Exhaustion语义按子系统漂移

当前同时存在checked error、permanent retirement、saturation freeze、wrapping reuse、collision probe、ordinary increment和panic。差异不一定都要消除：Sequence可以在窗口协议下回绕，slot可以永久退休，owner可以roll epoch；但每个allocator必须声明kind、reserved values、max live、reuse、generation、rollover、exhaustion、threading和diagnostics。没有该contract，就无法审查一个新增`fetch_add`是否正确。

### 7.4 Typed stale error只存在于少数owner

RHI的错误区分值得保留，但World、session、operation、watch、observer、service与UI仍不能一致区分wrong kind、wrong owner、stale generation、retired、already complete、unknown和exhausted。ABI映射也必须保持稳定，不能把所有错误压成invalid handle或not found。

### 7.5 序列化边界默认仍不安全

RHI已经证明diagnostic serialization与resolvable serialization可以分离。全引擎需要默认拒绝ephemeral handle进入serde/schema/remote DTO；确需记录时输出不可反解的`DebugIdentity`，包含domain/owner epoch/kind/raw display与明确的diagnostic-only标记。`WorldHandle`和raw ABI token不应因derive方便而获得跨运行语义。

## 8. 参考实现差异

| 参考 | 可验证机制 | Zircon当前差异 | 应吸收的原则 |
|---|---|---|---|
| Unreal `FWeakObjectPtr` | object index + serial，区分null/valid/stale并有集中测试 | Zircon多数弱/观察引用没有统一stale状态 | weak resolve必须可诊断，serial/epoch失配不能退化为普通not found |
| Unreal ObjectHandle | resolved/unresolved状态和明确resolve边界 | Zircon persistent/live身份经常透明互换 | soft/persistent reference和live handle必须由显式resolve/remap桥接 |
| Bevy Entity | index + generation，强调只在所属world/app实例内有效 | Zircon裸EntityId广泛跨World/DTO传播 | owner-local紧凑布局可以保留，跨owner必须强类型限定 |
| Fyrox Pool Handle | index + generation、invalid handle和pool验证 | Fyrox自身普通代际增长也不是无条件金标准 | 借鉴API形状，不复制未声明的wrap策略 |
| Godot RID owner | owner validator、初始化/归属检查、capacity/free/leak诊断 | Zircon缺统一owner validation与leak census | resolve必须验证owner；teardown必须报告live/retired/leaked状态 |
| Unity RenderGraph | resource type/index/version、registry写版本、pool release/leak路径 | Zircon部分render/runtime handle仍缺version/owner或释放证据 | transient graph handle必须绑定graph/registry epoch和usage lifecycle |

Unreal的全局object array、Bevy的Entity位布局、Godot RID的具体validator宽度和Unity C# registry都不是Zircon目标架构。Zircon还需要project/session/device/world replacement等多owner epoch、Rust类型边界和稳定FFI错误，这些不能由单一参考实现直接提供。

## 9. 目标架构

### 9.1 五类公共语义

1. `PersistentId`：可跨运行、可序列化，必须有namespace、schema/algorithm version和migration policy。
2. `LiveHandle`：只在owner instance/epoch内resolve，默认non-serde，slot/generation只是物理实现。
3. `ScopedHandle`：除live规则外，还绑定session/operation/graph/watch scope。
4. `Sequence`：只表达顺序或correlation；允许回绕时必须定义窗口、比较和重放规则。
5. `Revision`：表达某个owner状态版本；达到上限时roll owner epoch或显式停止，不得饱和后继续发布。

### 9.2 逻辑模型

```rust
struct OwnerKey {
    domain: IdentityDomainId,
    instance: OwnerInstanceId,
    epoch: OwnerEpoch,
}

struct SlotKey {
    index: SlotIndex,
    generation: SlotGeneration,
}

struct LiveObjectKey {
    owner: OwnerKey,
    kind: IdentityKind,
    slot: SlotKey,
}
```

这是公共逻辑和诊断模型，不要求所有热路径都物理携带全字段。ECS内部可保持紧凑index-generation；前提是owner由类型/borrow上下文不可伪造地提供，跨World、线程、ABI或持久化边界必须恢复完整qualification。

### 9.3 共享policy，不建全局巨型allocator

中央`IdentityDomainRegistry`只分配/登记domain、owner instance/epoch和manifest，不接管所有slot hot path。Scene、RHI、Service、Script等继续拥有专用allocator，但实现或声明统一`AllocatorContract`：kind、owner、reserved、width、reuse、generation、rollover、exhaustion、threading、serialization、metrics和conformance profile。

### 9.4 统一resolve与teardown结果

公共逻辑错误至少区分`WrongKind`、`WrongOwner`、`Stale`、`Retired`、`Exhausted`、`Unknown`、`AlreadyComplete`；ABI以稳定code和bounded diagnostic detail映射。Owner teardown按“停止admission -> 标记closing -> 等待lease -> 轮换/终结epoch -> invalidate -> 发布receipt”的顺序执行，receipt记录live/free/retired/leaked、late resolve和超时。

### 9.5 Persistent-to-live事务

Scene/prefab/save restore必须先验证persistent graph与schema，再创建新的World owner epoch，批量分配live slot，建立stable-to-live remap，修复引用并验证闭包，最后原子publish。失败不得暴露部分live graph；reload/replacement必须让旧handle、watch、query、cache和异步结果统一失效。

## 10. 重构里程碑

| 里程碑 | 交付物 | 首批RED证据 | 完成条件 |
|---|---|---|---|
| M0 现状冻结 | identity inventory、allocator manifest草案、owner路由 | 未分类public newtype/serde handle扫描 | 所有公开identity都有kind/owner/lifetime/codec/exhaustion记录 |
| M1 公共语义 | `IdentityKind`、`OwnerKey`、`AllocatorContract`、resolve error | wrong-kind/wrong-owner/stale/exhausted统一用例 | 不要求统一物理布局，但错误与manifest可机器检查 |
| M2 Persistent/live hard cut | `SceneObjectId`、World-qualified live entity、load remap | 跨World同bits、replacement、restore collision测试 | 保存/authoring不再写live handle，runtime cache不再持裸persistent ID |
| M3 World snapshot | `WorldInstanceId/Epoch`、`WorldSnapshotKey` | old watch/query/AI result命中新World测试 | World/Level/Watch/Fact/async apply统一验证snapshot identity |
| M4 ABI scoped handle | session/operation/subscription registry epoch与稳定错误 | wrong session、reuse、late completion、restart测试 | raw token不能在错误session/registry中偶然命中 |
| M5 Allocator convergence | 修复World/archetype/message/observer/UI/RHI namespace | 小位宽wrap、retire、collision、clone/rebuild模型 | 不再存在未声明的ordinary/wrapping/saturating identity increment |
| M6 Persistence与迁移 | legacy UUID catalog、alias、collision、receipt、BuildSet绑定 | old project fixture、mixed-era、collision、rollback | 旧工程可预览、迁移、回滚并验证引用闭包 |
| M7 Qualification | conformance suite、metrics、support dump、teardown census | fault/fuzz/property/soak与owner replacement | 每个registered allocator通过适用矩阵并发布低基数证据 |

M0-M1只建立合同和失败证据；不能以增加抽象层为由延后P1-025至P1-030的直接overflow修复。M2-M4是破坏性边界迁移，应按Runtime Interface -> Runtime owner -> App host -> Editor consumer顺序hard cut，不长期保留同语义双API。

## 11. 资格门

| Gate | 状态 | 当前证据/缺口 |
|---|---|---|
| G1 公共identity taxonomy | Fail | 无公共分类与review gate |
| G2 stable UUID canonical算法 | Pass | 版本、domain separation、length framing、fixed vector具备 |
| G3 legacy UUID migration | Partial | 新算法已在，旧ID catalog/alias/receipt缺失 |
| G4 Scene persistent/live split | Fail | 仍是裸`EntityId` |
| G5 owner-local ECS handle封装 | Pass | `InternalEntity`已crate-private、non-serde、generational |
| G6 World owner/replacement identity | Fail | `WorldHandle(u64)`无project/epoch |
| G7 ProjectIdentity admission | Partial | 类型存在，生产session/world/artifact绑定不足 |
| G8 ABI scoped token | Partial | registry局部checked，wire/token仍裸 |
| G9 RHI qualified handle | Pass | device/epoch/allocator/kind/slot-generation验证成立 |
| G10 Scene EntityId exhaustion | Pass | reserved/max/load/direct spawn统一fallible allocator |
| G11 ECS slot retirement | Pass | checked width、generation exhaustion永久退休 |
| G12 World revision exhaustion | Fail | saturation freeze仍在 |
| G13 archetype membership rollover | Fail | wrapping alias仍在 |
| G14 message/cursor exhaustion | Fail | panic与saturation仍在 |
| G15 observer identity lifecycle | Fail | ordinary increment、clone reset、无owner epoch |
| G16 engine-wide allocator policy | Fail | 多种未声明策略并存 |
| G17 unified resolve error | Partial | RHI强，其他owner漂移 |
| G18 handle ownership/lease类型 | Partial | ServiceCallGuard存在，多数handle未表达 |
| G19 world-sync snapshot qualification | Partial | Editor wrapper强，wire DTO仍裸 |
| G20 persistent-to-live remap | Partial | load transaction底座有，公共remap authority无 |
| G21 teardown invalidation receipt | Partial | 局部drain存在，无统一receipt/census |
| G22 external identity trust | Fail | namespace/auth/replay/forgeability无统一合同 |
| G23 allocator conformance tests | Partial | 局部边界测试有，全owner矩阵无 |
| G24 identity diagnostics/metrics | Fail | 无统一high-water/stale/wrong-owner/exhaustion观测面 |

## 12. Owner路由与非重复边界

| Owner | 本篇只记录的边界 |
|---|---|
| Runtime24 / Runtime159 | identity taxonomy、跨owner逻辑模型、allocator policy、stale/exhaustion资格总账 |
| Runtime05 | Scene/ECS/World具体数据结构与实体生命周期实现 |
| Runtime01 / Runtime157 | Core service handle、lease、module/service teardown |
| Runtime02 / Runtime158 | message/event/task sequence和execution owner |
| Runtime09A / Runtime101 | RHI device/resource handle与GPU lifecycle具体实现 |
| Interface01 | ABI handle layout、FFI registry与错误码 |
| Interface10 | stable UUID/project identity/serialization/world-sync public DTO与migration |
| Editor scene/gateway报告 | authoring document、qualified watch、play/runtime session消费者 |
| Tooling报告 | 通用CI、fuzz、benchmark、magic constant和error治理；本轮按用户要求排除 |

本文没有声称当前实现的性能或表现已经超过Unreal，也没有用静态扫描替代benchmark。目标架构明确允许热路径保持紧凑布局，但在正确性合同、产品规模profile和可重复benchmark完成前，不接受以“性能”为理由省略owner、epoch、stale或exhaustion语义。

## 13. 本轮边界

本轮只修改review、index与coverage，不修改production、test、Cargo或ABI；没有运行Cargo、Editor、Runtime DLL、旧项目迁移、跨进程session、device-loss、fault、fuzz、scale、soak或动态benchmark。Tooling按用户要求排除，也没有查询、轮询、等待或实时跟踪协调器状态。后续实现必须先按M0-M1补RED证据与公共合同，再按owner逐层hard cut，并在每个里程碑重新冻结干净源码与测试证据。
