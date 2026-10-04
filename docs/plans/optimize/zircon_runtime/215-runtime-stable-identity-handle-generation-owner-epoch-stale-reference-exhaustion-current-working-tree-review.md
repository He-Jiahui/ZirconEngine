---
title: Runtime Stable Identity、Handle、Generation、Owner Epoch、Stale Reference、Exhaustion 当前工作树复核
category: zircon_runtime
report_id: Runtime215
review_date: 2026-09-01
baseline_head: 5798051603e7f7f565538125c9aba96d5beabae2
baseline_epoch: 2026-09-01
verification_head: working-tree
verification_epoch: 2026-09-01
canonical_owner: Runtime24
refreshes:
  - zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - zircon_runtime/159-runtime-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-current-source-review.md
related_owner_reports:
  - zircon_runtime/187-runtime-scene-ecs-world-entity-component-query-schedule-current-working-tree-review.md
  - zircon_runtime/193-runtime-module-service-registry-dependency-activation-unload-current-working-tree-review.md
  - zircon_runtime/194-runtime-event-message-observer-bus-current-working-tree-review.md
  - zircon_runtime/205-runtime-resource-lifecycle-load-ticket-cache-residency-generation-reload-cancellation-current-working-tree-review.md
  - zircon_runtime/209-runtime-support-crates-contracts-math-resource-rhi-wgpu-workspace-boundary-device-lifecycle-product-integration-current-working-tree-review.md
  - zircon_editor/270-editor-scene-viewport-runtime-render-scene-visibility-hzb-picking-surface-product-integration-current-working-tree-review.md
related_code:
  - zircon_runtime_interface/src
  - zircon_runtime/src
  - zircon_runtime/crates
  - zircon_app/src
  - zircon_editor/src
  - zircon_plugins
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
doc_type: current_working_tree_review
review_status: complete
implementation_status: partial_foundation_identity_contract_incomplete
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime215: Stable Identity / Handle / Generation / Owner Epoch / Stale Reference / Exhaustion 当前工作树复核

- 复核日期：2026-09-01
- 复核 HEAD：`5798051603e7f7f565538125c9aba96d5beabae2`
- 复核类型：review-only；未修改 Rust、Cargo、ABI、tests、shader 或产品 UI，也未运行 Cargo、Runtime、Editor、网络、真实 GPU、fault、fuzz、scale、soak 或 benchmark。
- 当前约束：MVP 基础门仍未通过；身份分类、owner、epoch、stale、exhaustion 和持久化边界必须先闭合，不能以热路径性能为由继续增加裸整数身份。
- Tooling：按用户要求排除；本轮没有查询、轮询、等待或实时跟踪协调器。

## 1. 结论

Runtime24/159 的总判断仍成立：Zircon 已经拥有若干工程级局部实现，但还没有全引擎身份系统。Stable UUID 的 BLAKE3/UUIDv8 算法、Scene 实体分配、ECS 内部 slot generation、RHI device/allocator/kind/slot qualification、Dynamic Session checked handle、Script registry stale rejection、Service call lease 和 Editor gateway qualification 都是应保留的底座。它们证明紧凑句柄、显式 owner 和可失败耗尽可以同时存在，并不要求所有热路径携带一个巨型通用结构。

阻塞项仍在公共和跨 owner 边界：`EntityId = u64`同时承担持久 Scene identity、authoring identity 和 live lookup；`WorldHandle(u64)`可 serde 且不携带 project/world instance/replacement epoch；world-sync 的 `EntityId`、`WatchToken`、`WorldFact` 和多数 query result 仍是裸 wire 值；Runtime 把已存在的 `ProjectIdentity`降级成项目名 `Option<String>`。World/component/event/UI/render revision 继续 saturation 或 wrap，Service generation 会从 `u32::MAX`回到 1，RHI allocator namespace 和多种全局 ID 使用无耗尽的 `fetch_add`。网络插件又把 7 类可 serde ID 统一实现成 `u64`，socket/listener/connection/route/session 分配没有 owner epoch、冲突或耗尽合同，handshake 默认 nonce 还是固定字符串。

本轮扩展到五个产品源码族后，旧 40 项 P1 仍为 **8 Closed / 17 Partial / 15 Open**，12 项 P2 仍为 **12 Open**，24 道资格门仍为 **10 Fail / 9 Partial / 5 Pass**。不新增平行 canonical ID；所有后续实现继续回写 Runtime24 的 `IDENTITY-P1/P2-*`。

## 2. 冻结范围与证据强度

### 2.1 产品源码全量边界

本轮枚举 Runtime Interface、Runtime、App、Editor 和全部 `zircon_plugins` 的 Rust 产品/测试/bench 源码；排除 `target`、`native/vendor`、`dev` 参考树。统计按当前工作树读取，不只看 HEAD。`tests`是 `#[test]`/`#[tokio::test]`词法数，`ignored`是 ignore/cfg_attr ignore 词法数，`unsafe`也是词法数；这些数字不等于动态通过数。指纹为排序后的规范相对路径和 byte length 计算 SHA-256。

| 范围 | files | lines | bytes | tests | ignored | unsafe | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Runtime Interface | 581 | 83,540 | 2,767,645 | 713 | 30 | 117 | `1eaea98e009315a34f313f73ceee0ee25efce6e234b63ffaf9505d676087346c` |
| Runtime（含内部 crates） | 9,419 | 1,627,258 | 58,322,819 | 16,881 | 901 | 783 | `3f4e584be1dafd5df7b56e663ffa092ecce1965d15e886fdcca82ee3f6a84512` |
| App | 244 | 41,647 | 1,554,066 | 652 | 2 | 158 | `33c5a532fb7ff93f9d5a07b501912990f0ea3d3b09fc86e6d04c7cc56fa544df` |
| Editor | 6,364 | 797,158 | 27,571,501 | 8,423 | 632 | 195 | `df6fd49c380a1feb10d0b016418bf1c45f8ecb5e1e316d12bd309a8ad19ea990` |
| 全部 plugins | 2,908 | 285,357 | 10,125,616 | 2,580 | 226 | 526 | `8eb28221bbe20adbacab104ec8a7d3b135c09abd4ce4e0dac39ff7987d2d81ae` |
| 去重联合集 | **19,516** | **2,834,960** | **100,341,647** | **29,249** | **1,791** | **1,779** | `a6892dc27d44463fc32f5c35b38b917e4d31f17dd5e34f3206f7447a218435ea` |

冻结时工作树另有 215 个 tracked dirty 和 96 个 untracked path。`Closed`只表示该局部源码合同在这次快照中成立，不代表已进入干净 checkout、CI 或发布 BuildSet。

### 2.2 深审选择集与负扫描

对身份 owner、allocator、wire/serde、revision、consumer 路径进一步冻结 6,587 个文件、956,326 行、33,818,098 bytes、9,402 个 test markers、452 个 ignored 和 787 个 unsafe 词项，指纹 `7d3dda92411313b5d9354df674768b3a72e018526054d1a7a9003f9ccbf90bee`。该集合命中 151 个 public-ish ID/Handle/Token/Generation/Epoch/Revision/Sequence 声明和 38 个数值 tuple wrapper。扩大后的精确 serde-numeric wrapper 匹配为 19 个，其中 3 个是 RHI bitmask/value wrapper，至少 16 个属于 identity/revision-like 公共值。

排除常规 test/bench 路径后的 3,785 个执行源码候选中，词法命中 `wrapping_add` 100 次、`saturating_add` 1,823 次、`fetch_add` 113 次、`checked_add` 331 次。这里包含 checksum、统计和容量算法，不能把命中数直接当 finding；本轮逐行裁决的身份相关命中包括 Service generation、RHI namespace、World/component revision、archetype membership、Message/Observer/Watch、UI timeline/virtualization、render resource generation、App wake token、Editor snapshot sequence 和 Network IDs。

### 2.3 参考选择集

13 个本地参考文件共 9,493 行、349,455 bytes、76 个测试宏/属性信号，路径长度指纹 `c045f7fd3078f091c982ba98142073f82a1b58e1764e1d0bf58ddb3655292d95`。Unreal 用于 weak stale 与 resolved/unresolved 边界，Bevy/Fyrox 用于 owner-local index-generation，Godot 用于 RID owner/validator/capacity/free/leak，Unity Graphics 用于 RenderGraph type/index/version、registry 与 pool lifecycle。本文不把任何参考的位布局、全局对象表或语言实现直接复制为 Zircon 架构。

## 3. 五类身份语义盘点

| 语义类 | 当前代表 | 已有底座 | 当前断口 |
|---|---|---|---|
| Persistent ID | `ResourceId`、`AssetUuid`、`ProjectGuid`、`ProjectIdentity`、Scene `EntityId` | Stable UUID 有版本、domain separation、长度 framing、固定向量；Project 有 GUID + manifest digest | Scene persistent/live 共用裸 `u64`；legacy UUID alias/migration/receipt 缺失；Runtime admission 把 `ProjectIdentity`降级成项目名 |
| Live Handle | `InternalEntity`、RHI handles、`WorldHandle`、window/font/render handles | ECS 内部与 RHI 有 slot generation/stale validation | `WorldHandle`无 owner epoch；大量 live handle 可 serde 或只靠调用约定限定 owner |
| Scoped Handle | Session/operation/subscription/watch/service call | Session registry、operation service、ServiceCallGuard、Editor QualifiedWatchToken 有局部 scope | wire token 仍裸；scope 信息在旁路参数或 consumer wrapper，不在公共可验证 identity 中 |
| Sequence | message、event delivery、operation correlation、network request、frame/ticket | 若干 owner 使用 checked add 或显式窗口 wrap | 仍混用 panic、ordinary increment、saturation、wrap-and-probe、unchecked atomic；没有窗口/replay/排序合同 |
| Revision | World/component/lifecycle/archetype/UI/render/cache generation | replacement 局部 guard、部分 checked publication | saturation 后冻结或 wrapping alias；owner epoch rollover 和 terminal disposition 未统一 |

全产品源码没有 engine-wide `IdentityDomainId`、`AllocatorContract`、`LiveObjectKey`、`OwnerKey`、`WorldSnapshotKey`、`IdentityManifest`或`DebugIdentity`。UI 内部存在自己的 `UiModelIdentityKind`，但它不是跨 Runtime/RHI/Scene/Script/Network 的公共合同。

## 4. 已证实可保留的工程底座

### 4.1 Stable UUID 和 Project schema

`zircon_runtime_interface/src/resource/stable_uuid.rs` 使用 BLAKE3 derive-key、算法版本、component count/length framing、UUID v8 和固定跨平台向量，P1-002 保持 Closed。`ProjectGuid`拒绝 nil，`ProjectManifestDigest`使用 32-byte digest，`ProjectIdentity`组合 canonical descriptor、GUID 和 manifest digest。

缺口在 lifecycle：未找到旧 DefaultHasher ID catalog、old-to-new alias、collision/tombstone 和 migration receipt；stable algorithm version 未进入 save/cache/artifact/BuildSet admission。`zircon_runtime/src/dynamic_api/session/construction.rs`仍从 `project_info.name`生成 `Option<String>`，`GatewaySessionIdentity.project`也只是可选字符串，没有消费已存在的 `ProjectIdentity`。

### 4.2 Scene allocator 与 ECS slot

`EntityIdAllocator`统一 direct/deferred/bundle/transaction/load 路径，保留 0 和 `u64::MAX`，耗尽返回 `SceneError::EntityIdExhausted`。`EntityRegistry`用 checked slot capacity，`InternalEntity { index, generation }`是 crate-private、non-serde、World-local；generation 耗尽永久退役 slot，并有 stale/capacity/atomicity 测试。P1-006、018、021 至 024 保持 Closed。

这不能外推到公共 Scene identity。`zircon_runtime/src/core/framework/scene/mod.rs`仍定义 `EntityId = u64`，同一数值进入持久文件、authoring、runtime World map、watch/query、AI/physics/navigation/plugin key。`stable_to_internal`只是一个 World 内部 map，不是带 project/scene/world owner 的 canonical persistent-to-live remap authority。

### 4.3 RHI 与 Script registry

RHI resource handle 已携带 allocator namespace、`DeviceId`、`DeviceGeneration`、kind、slot 和 slot generation；resolve 区分 wrong device/generation、foreign allocator、wrong kind 和 stale，slot generation 耗尽永久退役。序列化只输出 diagnostic ID，反序列化固定拒绝。Script host/export/hot-reload registry也有 slot/index capacity、generation mismatch、checked exhaustion 和 stale-after-reuse 测试。

剩余边界是 owner namespace 自身：`NEXT_RESOURCE_NAMESPACE`、`NEXT_SURFACE_NAMESPACE`和 WGPU device ID 仍以 `fetch_add`分配，没有 zero/reserved/exhaustion 或 process restart contract。Script 各 registry 的错误/teardown 也没有并入公共 taxonomy/receipt。

### 4.4 Session、Operation、Service 与 Editor gateway

Dynamic Session handle 达到 `u64::MAX`后进入永久 exhausted state；Operation 和 plugin-event subscription 使用 checked add，调用时 registry/service instance 提供局部 session scope。ServiceHandle 绑定 Core owner、service name/index/generation，ServiceCallGuard 持有 in-flight lease，shutdown 先关闭 admission 再 drain。Editor 的 `GatewaySessionIdentity`组合 runtime instance/session、gateway generation、transport epoch、project/play 信息，`QualifiedWatchToken`拒绝旧 gateway token 和 collision。

局部模型仍不等于公共合同。ABI 的 session/operation/subscription/viewport/plugin handle 是 transparent `u64`；operation/subscription 还可 serde。Service generation 使用 `wrapping_add`并在 0 时回到 1，旧 handle 最终可以别名新实例。Editor wrapper只能保护走该 wrapper 的 consumer，不能修复 Runtime Interface wire DTO。

## 5. 关键工程缺口

### 5.1 Persistent Scene ID 与 live handle 没有 hard cut

`EntityId`既可保存又可直接 lookup，无法表达 project、scene asset、World instance 或 replacement epoch。两个 World 可拥有相同 bits；旧异步结果、watch、AI fact 或 plugin key 只要数值相同就可能命中新 World。目标不是把 `u64`机械扩大，而是拆成 `SceneObjectId`和 owner-qualified live entity，在 load transaction 内建立一次性 remap，失败时不得发布部分 live graph。

### 5.2 World replacement identity 不是不可拆分 key

`WorldHandle(pub u64)`直接 derive serde。Level 内部 `world_replacement_epoch`通过 checked `fetch_update`前进，但耗尽会 `expect` panic；epoch 也没有进入 `WorldHandle`、`WatchToken`、大多数 `WorldQuery`或 `WorldFact`。只有 transform snapshot result显式带 replacement epoch，其他 hierarchy/component/inspection result只带 generation。结果是“有 generation”与“属于同一 World instance”被错误等同。

### 5.3 Revision 到上限后静默失效

`WorldGeneration`、lifecycle visibility、dynamic component generation、event/message generation、frame state、schema registry、UI surface/navigation/publication 和大量 render cache revision使用 `saturating_add`。达到上限后 mutation 继续成功但 revision 不再变化，依赖 `==`或 generation hint 的 cache/watch 会失去 invalidation。`WorldQuery`对 `u64::MAX`禁用 NotModified 只避免一个读取短路，不会恢复 revision identity。

`ArchetypeRecord.membership_generation`、UI virtualization/hit-test、Service generation 和许多 render resource generation使用 wrapping。合法的 change tick 可以在明确 age window 下回绕；普通 object/revision generation 没有窗口或 owner rollover 时不能照搬该策略。

### 5.4 Message、Observer 与 Watch 不是一个生命周期系统

- `Messages::write_at_frame`在 ID 耗尽时 `expect("message id space exhausted")`；clear generation和 store frame使用 saturation。
- 公开 `ObserverStore`用 `next_id += 1`，release 可回绕，debug 可 panic；Clone 返回全新默认 store，重新从 0 发号，没有 owner epoch。
- 新 `EventStore`已经用 checked observer ID 和 non-cloneable reader lease，但 exhaustion 返回 `Option`，queue generation仍 saturation；旧/new observer/event/message 形成平行语义。
- `SubscriptionTable::allocate_token`使用 wrapping + live-map probe；空间占满时没有 exhausted result，理论上永久循环。

这不是要求事件系统共享一个 allocator，而是要求它们声明同一类 Sequence/ScopedHandle 合同、统一 terminal error，并禁止 clone/reset/replacement 复用旧 scope。

### 5.5 UI、Render 与 GPU 资源 revision 继续制造别名

`UiDebugTimelineStore`在 `u64::MAX`后把 `next_handle`饱和在最大值，后续 snapshot 重复同一 handle；其 `contains_handle`又按首尾数值区间判断，无法表达重复或 owner。UI virtual list/materialization、surface/tree revision、binding registry也混用 wrap/saturation。

Render 侧可见 `next_material_draw_generation`、UI texture generation、frame generation、history domain、bindless slab、HZB buffer revision、indirect workspace revision、IBL/probe epoch 等大量 `wrapping_add(1).max(1)`或 saturation。很多只需 owner-local cache revision，但必须在 allocator manifest 中声明“回绕前清空 owner/cache”或转成 checked epoch rollover；当前没有统一可审查合同。

### 5.6 Network identity 和 trust/replay 是新的高风险证据

`zircon_runtime/src/core/framework/net/ids.rs`用同一宏把 `NetListenerId`、`NetConnectionId`、`NetSessionId`、`NetRequestId`、`NetRouteId`、`NetDownloadId`、`NetObjectId`实现成可 serde `u64`，constructor 接受 0 和任意伪造值。Net runtime 的 socket/listener/connection/route allocator用 `fetch_add(1) + 1`，RPC session使用普通 `+= 1`；没有 owner instance、generation、collision probe、typed exhaustion 或 restart invalidation。

`NetSessionHandshakePolicy`默认固定 challenge nonce，control DTO携带字符串 `player_id`和 response；现有测试覆盖流程与 unknown ID，不覆盖 ID exhaustion、跨 runtime owner、replay 或 nonce freshness。这使 P1-009 与 P1-038 继续 Open，不能把“有 newtype/serde”视为网络身份安全。

### 5.7 没有公共 allocator/resolve/teardown 资格面

RHI、Script、Service、Scene、Editor 各自有不同强度的错误和测试，但没有共享 `WrongKind/WrongOwner/Stale/Retired/Exhausted/Unknown/AlreadyComplete`逻辑 taxonomy；ABI 又把很多失败压成 invalid/not found/string。没有 allocator manifest记录 reserved values、width、reuse、generation、rollover、exhaustion、threading、serialization 和 metrics。

Teardown 有局部 drain/release/receipt，却没有统一顺序和 census：停止 admission、标记 closing、等待 lease、轮换 owner epoch、invalidate、发布 live/free/retired/leaked/late resolve receipt。仓库有大量 stale/exhaustion 单元测试，但没有把每个注册 allocator 放进同一 conformance matrix；Network ID、World revision、Message/Observer/Watch 和 UI timeline 的边界缺口正因此长期并存。

## 6. Runtime24 P1 当前重判

| ID | 状态 | 当前工作树裁决 |
|---|---|---|
| IDENTITY-P1-001 | Open | 无 engine-wide 五类 taxonomy、admission rule 或 review gate |
| IDENTITY-P1-002 | Closed | Stable UUID 已有 BLAKE3/UUIDv8、版本、domain separation、长度 framing、固定向量 |
| IDENTITY-P1-003 | Partial | canonical algorithm 已有；未绑定 BuildSet/artifact/save/cache compatibility |
| IDENTITY-P1-004 | Open | 无 legacy alias/catalog、collision、tombstone、preview/rollback receipt |
| IDENTITY-P1-005 | Open | `EntityId = u64`继续混合 persistent、authoring、live lookup |
| IDENTITY-P1-006 | Closed | `InternalEntity`保持 crate-private、non-serde、World-local generational handle |
| IDENTITY-P1-007 | Open | `WorldHandle(u64)`可 serde，无 project/world instance/replacement epoch |
| IDENTITY-P1-008 | Partial | RHI 拒绝反序列化；World/operation/subscription/watch/net 等 ephemeral identity 仍可 serde |
| IDENTITY-P1-009 | Open | project name降级及 Network/string/device/parameter identity 缺统一 namespace/trust normalization |
| IDENTITY-P1-010 | Open | 无 identity manifest 和 CI admission |
| IDENTITY-P1-011 | Partial | RHI/Editor/Service/Script 有局部 qualified model；无公共逻辑模型 |
| IDENTITY-P1-012 | Partial | Level/Editor 有 replacement guard；World public API和多数 DTO 没有 |
| IDENTITY-P1-013 | Partial | 多个 registry 有 validation；公开 ABI 仍是 transparent raw integer |
| IDENTITY-P1-014 | Partial | session registry/operation service提供旁路 scope；token自身没有 owner epoch |
| IDENTITY-P1-015 | Partial | Level replacement generation存在；AI/Network/public World-Entity key仍可裸别名 |
| IDENTITY-P1-016 | Closed | RHI resource handle具备 device/generation/allocator/kind/slot generation |
| IDENTITY-P1-017 | Partial | owner-local policy增多；没有 shared contract/conformance matrix，namespace仍可回绕 |
| IDENTITY-P1-018 | Closed | ECS内部 generational entity未重新泄漏为公共跨 World identity |
| IDENTITY-P1-019 | Partial | Editor QualifiedWatchToken较强；wire WatchKey/WorldFact/WatchToken仍裸 |
| IDENTITY-P1-020 | Partial | RHI/Core有 typed stale error；其他 owner 混用 None/bool/string/panic/domain enum |
| IDENTITY-P1-021 | Closed | Scene load/allocator拒绝 reserved 0 和 terminal max |
| IDENTITY-P1-022 | Closed | direct/deferred/bundle/transaction/load共享 EntityIdAllocator |
| IDENTITY-P1-023 | Closed | ECS slot capacity checked并返回 typed error |
| IDENTITY-P1-024 | Closed | ECS slot generation耗尽永久退役 |
| IDENTITY-P1-025 | Open | World/lifecycle/component/event/frame/schema/UI revisions仍大量 saturation |
| IDENTITY-P1-026 | Open | archetype membership仍 wrapping alias |
| IDENTITY-P1-027 | Open | message ID exhaustion panic，clear/cursor/frame generation saturation |
| IDENTITY-P1-028 | Open | ObserverStore普通自增、clone重置；新 EventStore 不能替代旧公共机制 |
| IDENTITY-P1-029 | Partial | session/operation/plugin event/Level handle有 checked路径；RHI/global/render/network ID仍 wrap/fetch |
| IDENTITY-P1-030 | Open | UI timeline重复最大 handle，virtualization/surface/render revision仍 wrap/saturate |
| IDENTITY-P1-031 | Open | 无中央 domain/owner instance/epoch registry |
| IDENTITY-P1-032 | Open | 无 allocator capability/contract/manifest |
| IDENTITY-P1-033 | Partial | service/session/RHI/World有局部 drain/release；无统一 epoch invalidation receipt/census |
| IDENTITY-P1-034 | Partial | ServiceCallGuard/EventReaderLease存在；多数公共 handle不表达 lease/lifetime |
| IDENTITY-P1-035 | Partial | RHI serialization是 diagnostic-only；其他 serde/wire handle仍可逃逸和重放 |
| IDENTITY-P1-036 | Partial | Scene load有原子分配/校验底座；无 canonical persistent-to-live remap authority |
| IDENTITY-P1-037 | Open | object identity、sequence、revision继续普遍混称 id/generation/handle |
| IDENTITY-P1-038 | Open | principal/connection/content token无统一 forgeability、authorization、nonce/replay合同 |
| IDENTITY-P1-039 | Partial | 局部 owner 边界测试丰富；无所有 allocator 共享的 conformance suite |
| IDENTITY-P1-040 | Partial | owner局部有错误/计数；无统一 high-water/stale/wrong-owner/exhaustion/late-resolve census |

状态合计：**Closed 8 / Partial 17 / Open 15**。

## 7. Runtime24 P2 当前重判

| ID | 状态 | 当前裁决 |
|---|---|---|
| IDENTITY-P2-001 capability token | Open | public trust/domain/owner/auth contract未完成 |
| IDENTITY-P2-002 packed handle | Open | 先完成逻辑合同与 profile，不预设全引擎位布局 |
| IDENTITY-P2-003 sharded allocator | Open | 无 allocator contention 产品 profile |
| IDENTITY-P2-004 handle sanitizer | Open | 公共状态机/transition/registration seam未固定 |
| IDENTITY-P2-005 identity inspector | Open | manifest与 diagnostic snapshot缺失 |
| IDENTITY-P2-006 UUID migration preview | Open | legacy catalog/alias/receipt缺失 |
| IDENTITY-P2-007 distributed reference | Open | local owner/replay/auth边界尚未闭合 |
| IDENTITY-P2-008 ULID/Snowflake event identity | Open | 无跨节点全序需求证据，不能用宽 ID 掩盖 owner 缺失 |
| IDENTITY-P2-009 TLA+ model | Open | allocator/teardown 状态机尚未统一 |
| IDENTITY-P2-010 debugger symbols | Open | DebugIdentity/schema未完成 |
| IDENTITY-P2-011 SDK typed wrapper | Open | ABI稳定合同和 owner qualification未完成 |
| IDENTITY-P2-012 dashboard | Open | 统一 metrics/census schema未完成 |

## 8. 参考实现差异

| 参考 | 本地源码可验证机制 | Zircon差异 | 应吸收的原则 |
|---|---|---|---|
| Unreal `FWeakObjectPtr` | object index + serial；明确 null/valid/stale；resolve failure 有测试和 metrics | 多数弱/观察引用只返回 missing/false，不能区分从未存在与已经 stale | stale 是一等状态；serial/epoch mismatch 不得降级为普通 not found |
| Unreal ObjectHandle | resolved/unresolved 分态，显式 resolve 并可记录 read/resolve/failure | persistent/live identity经常透明互换 | soft/persistent reference只能通过显式 resolve/remap 进入 live owner |
| Bevy Entity | index + generation，文档强调只在所属 World/App instance有意义；change tick另有 wrap window | Zircon把 owner-local ECS思想外推成跨 World 裸 EntityId | owner-local紧凑布局可保留，跨 owner 必须恢复 qualification |
| Fyrox Pool Handle | index + generation、invalid handle、pool侧验证与 free | Zircon多个 registry缺统一 pool/owner诊断；Fyrox普通 generation策略也不能盲抄 | 借鉴 API 形状和 owner validation，不复制未声明的 wrap策略 |
| Godot RID owner | owner validator、初始化/归属检查、capacity/free/leak诊断和 RID 测试 | Zircon缺统一 owner validator 与 teardown leak census | resolve必须验证 owner；teardown必须报告 live/retired/leaked |
| Unity RenderGraph | resource type/index/version、registry version、pool release/leak lifecycle | Zircon render/runtime handle有些仍无 graph/registry epoch和完成边 | transient handle必须绑定 graph/registry owner和usage lifecycle |

参考实现也不自动满足 Zircon 的 project/session/device/world replacement、Rust类型边界、stable ABI error 和 multi-plugin 要求。目标是吸收约束和失败语义，不是宣称当前性能或表现已经超过 Unreal。

## 9. 目标架构

### 9.1 公共逻辑模型

```rust
enum IdentityClass {
    Persistent,
    Live,
    Scoped,
    Sequence,
    Revision,
}

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

这是公共逻辑、错误和诊断模型，不要求 ECS/RHI/UI 热路径物理携带所有字段。owner可由不可伪造的 borrow/type/registry context隐含；一旦跨 World、线程、ABI、serde、network、async task 或 persistence 边界，就必须恢复完整 qualification。

### 9.2 Shared policy，不建全局巨型 allocator

中央 `IdentityDomainRegistry`只登记 domain、owner instance/epoch、manifest 和 diagnostic census。Scene、RHI、Service、Script、UI、Network继续拥有专用 allocator，但必须声明统一 `AllocatorContract`：kind/class、owner、reserved、width、max live、reuse、generation、rollover、exhaustion、threading、serialization、resolve error、teardown 和 metrics。

### 9.3 Persistent-to-live 事务

Scene/prefab/save restore先验证 persistent graph/schema/project identity，再创建新的 World owner epoch，批量分配 live slots，建立 stable-to-live remap，修复引用并验证闭包，最后原子 publish。失败不暴露部分 graph；replacement 必须统一失效旧 handle、query/watch、AI/network/plugin cache和异步结果。

### 9.4 统一 resolve 和 teardown

逻辑错误至少区分 `WrongKind`、`WrongOwner`、`Stale`、`Retired`、`Exhausted`、`Unknown`、`AlreadyComplete`；ABI映射为稳定 code和 bounded diagnostic detail。Owner teardown顺序固定为 stop admission -> closing -> drain lease -> rotate/terminate epoch -> invalidate -> publish receipt；receipt记录 live/free/retired/leaked、late resolve、timeout 和 forced disposition。

## 10. 重构里程碑

| 里程碑 | 交付物 | 首批 RED 证据 | 完成条件 |
|---|---|---|---|
| M0 Inventory | machine-readable identity/allocator manifest和未分类 public/serde 扫描 | 新 public numeric identity未登记即失败 | 所有 public identity都有 class/owner/lifetime/codec/exhaustion owner |
| M1 Common contract | `IdentityClass`、`OwnerKey`、`AllocatorContract`、resolve error、DebugIdentity | wrong-kind/owner/stale/retired/exhausted统一用例 | 不统一物理布局，但逻辑错误和 manifest可机器检查 |
| M2 Scene hard cut | `SceneObjectId`、World-qualified live entity、load remap | same bits cross-World、replacement、restore collision | persistence只写 persistent ID，runtime cache只持 live handle |
| M3 World snapshot | `WorldInstanceId/Epoch`、`WorldSnapshotKey` | old watch/query/AI/plugin result命中新 World | World/Level/query/watch/fact/async apply统一验证 snapshot |
| M4 ABI/scoped handle | session/operation/subscription/viewport registry epoch和稳定错误 | wrong session、reuse、restart、late completion/replay | raw token不能在错误 registry/session中偶然命中 |
| M5 Direct overflow repair | World/archetype/message/observer/watch/UI/service/RHI namespace/network IDs | 小位宽 wrap/saturate/panic/model tests | 不再存在未声明 ordinary/wrapping/saturating identity increment |
| M6 Persistence/migration | UUID legacy catalog、alias/collision/tombstone、BuildSet/save binding | old project fixture、mixed-era、collision、rollback | 可预览迁移、回滚并验证引用闭包 |
| M7 Qualification | cross-allocator conformance、metrics/support dump、teardown census | fault/fuzz/property/owner replacement/scale/soak | 每个 registered allocator通过适用矩阵并发布低基数证据 |

M0-M1不能成为延迟 P1-025 至 030 直接 overflow 修复的借口。M2-M4 是破坏性边界迁移，应按 Runtime Interface -> Runtime owner -> App host -> Editor/plugin consumer 顺序 hard cut，不长期保留同语义双 API。Network identity应在 M4-M5 同步修复，不能继续把可伪造 wire ID当成授权凭据。

## 11. 资格门

| Gate | 状态 | 当前证据/缺口 |
|---|---|---|
| G1 公共 identity taxonomy | Fail | 无 engine-wide class/domain/owner admission |
| G2 stable UUID canonical algorithm | Pass | version/domain/framing/fixed vector具备 |
| G3 legacy UUID migration | Partial | 新算法已在；legacy catalog/alias/receipt缺失 |
| G4 Scene persistent/live split | Fail | 仍是裸 `EntityId` |
| G5 owner-local ECS handle | Pass | `InternalEntity` private/non-serde/generational |
| G6 World owner/replacement identity | Fail | `WorldHandle(u64)`无 project/epoch |
| G7 ProjectIdentity admission | Partial | schema存在；Runtime session仍降级成 name String |
| G8 ABI scoped token | Partial | registry旁路约束存在；token自身仍裸 |
| G9 RHI qualified handle | Pass | device/epoch/allocator/kind/slot generation验证成立 |
| G10 Scene EntityId exhaustion | Pass | reserved/max/load/direct spawn统一 fallible allocator |
| G11 ECS slot retirement | Pass | checked width、generation exhaustion永久退役 |
| G12 World revision exhaustion | Fail | saturation freeze仍在 |
| G13 archetype membership rollover | Fail | wrapping alias仍在 |
| G14 message/cursor exhaustion | Fail | panic与saturation仍在 |
| G15 observer/watch lifecycle | Fail | ordinary increment/clone reset/full-space loop |
| G16 engine-wide allocator policy | Fail | 多种未声明策略并存 |
| G17 unified resolve error | Partial | RHI/Core强；其余 owner 漂移 |
| G18 handle ownership/lease type | Partial | Service/Event有 lease；多数 handle没有 |
| G19 world-sync snapshot qualification | Partial | Editor wrapper与单个 result较强；wire DTO仍裸 |
| G20 persistent-to-live remap | Partial | load transaction底座有；公共 authority无 |
| G21 teardown invalidation receipt | Partial | 局部 drain/release有；统一 receipt/census无 |
| G22 external identity trust | Fail | Network/project/principal/content namespace/auth/replay合同缺失 |
| G23 allocator conformance tests | Partial | 大量局部测试；没有 registered-owner矩阵 |
| G24 identity diagnostics/metrics | Fail | 无统一 high-water/stale/wrong-owner/exhaustion/late resolve 观测面 |

## 12. Owner 路由与非重复边界

| Owner | 本篇责任 |
|---|---|
| Runtime24 / Runtime215 | identity taxonomy、跨 owner逻辑模型、allocator policy、stale/exhaustion 总账 |
| Runtime05 / Runtime187 | Scene/ECS/World具体数据结构和实体生命周期实现 |
| Runtime01 / Runtime193 | Core service registry、lease、activation/unload teardown |
| Runtime02 / Runtime194 | event/message/observer具体机制和 execution owner |
| Runtime09A / Runtime209 | RHI device/resource/surface handle和 GPU lifecycle |
| Runtime Interface owner | ABI layout、serde/wire、stable errors、ProjectIdentity/world-sync contract |
| Plugin/Network owner | connection/session/request/object identity、trust/authorization/replay |
| Editor gateway/sync owner | authoring document、qualified watch、play/runtime session consumer |
| Tooling owner | CI、fuzz、benchmark、magic constant和通用 error治理；本轮按用户要求排除 |

## 13. 本轮边界

本轮只写 review、index 和 coverage 文档。没有把静态词法命中当作动态正确性或性能结论，没有运行 Cargo、Runtime DLL、Editor、旧项目迁移、跨进程 session、network replay、device loss、真实 GPU、fault、fuzz、scale、soak 或 benchmark。下一实现阶段应先建立 M0/M1 的机器可检查清单和 RED conformance case，同时直接修复会 panic、永久循环、饱和冻结或别名回绕的 P1-025 至 P1-030；每个 owner 完成 hard cut 后重新冻结源码与动态证据。
