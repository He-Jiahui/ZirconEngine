---
title: Runtime Resource Authority、Asset Loading、CPU Lease、Render Asset Residency 当前工作树复核
category: zircon_runtime
report_id: Runtime224
review_date: 2026-09-02
baseline_head: 9963f8eb72e2d725d2536eb50b393b30387a1ffa
baseline_epoch: 2026-09-02
verification_head: working-tree
verification_epoch: 2026-09-02
supersedes_currentness_of:
  - zircon_runtime/99m-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-current-source-review.md
  - zircon_runtime/09d-render-asset-streaming-residency-review.md
related_owner_reports:
  - zircon_runtime/04-core-resource-asset-serialization-review.md
  - zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - zircon_runtime/88-runtime-asset-watch-change-ingress-coalescing-rename-overflow-targeted-reimport-generation-reload-product-integration-current-source-review.md
  - zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md
  - zircon_runtime/93-runtime-mesh-geometry-section-lod-instancing-skinning-morph-deformation-bounds-collision-streaming-product-integration-current-source-review.md
  - zircon_runtime/101-runtime-support-crates-contracts-math-resource-rhi-wgpu-workspace-boundary-device-lifecycle-product-integration-current-source-review.md
  - zircon_runtime/223-runtime-render-graph-gpu-scene-render-scene-frame-submission-current-working-tree-review.md
related_code:
  - zircon_runtime_interface/src/resource
  - zircon_runtime/crates/zr_resource
  - zircon_runtime/src/core/resource
  - zircon_runtime/src/asset/facade
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager
  - zircon_runtime/src/asset/pipeline/manager/resource_sync
  - zircon_runtime/src/asset/project/manager/artifact_access.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/scene/resources/gpu_model
  - zircon_runtime/src/graphics/scene/resources/gpu_texture
  - zircon_runtime/src/graphics/scene/resources/prepared
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
tests:
  - zircon_runtime/crates/zr_resource/src/manager/tests
  - zircon_runtime/crates/zr_resource/src/manager/readiness_projection/tests
  - zircon_runtime/crates/zr_resource/src/io
  - zircon_runtime/src/asset/tests/facade
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime/tests.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/tests
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/StreamableManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/StreamableManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/AssetManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/Serialization/AsyncLoading2.h
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/assets.rs
  - dev/bevy/crates/bevy_asset/src/event.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-resource/src/state.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/Fyrox/fyrox-resource/src/entry.rs
  - dev/godot/core/io/resource_loader.h
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourcePool.cs
doc_type: current_working_tree_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime224: Resource Authority、Asset Loading、CPU Lease、Render Asset Residency 当前工作树复核

## 1. 结论

当前工作树已经把 `zr_resource` 作为 workspace 成员接入，并形成了 registry、management/readiness generation、preflight/apply mutation、bounded event log、typed facade、CPU lease、artifact transaction 和 RenderAssetResidency 状态机等真实底座。Runtime153 中“`zr_resource` 未进入 Cargo workspace”的旧结论不再适用；但这只是结构修复，不代表资源系统已经完成。

本轮最重要的事实是：资源系统仍存在两条互不闭合的生产路径。

```text
ProjectAssetManager::load_* / ResourceStreamer::ensure_*
  -> ensure_resident
  -> 同步 artifact read + 解压/反序列化 + store payload
  -> typed Arc downcast + clone
  -> 旧 ResourceStreamer map + 直接 wgpu 创建/上传

RenderScene journal delta
  -> RenderAssetResidencyManager::apply_scene_reference_deltas
  -> QueuedIo ticket 写入 pending map
  -> 在途 semantic executor 可读 manifest/block 并生成 upload plan
  -> 生产代码没有 executor owner/caller、RHI submitter 或 draw publication consumer
```

因此，现有的 RenderAssetResidency ticket、semantic block loader、GPU upload artifact 和 device epoch 目前仍是未闭合的局部协议。共享工作树新增并在 `mod.rs` 中纳入了未跟踪的 `semantic_executor.rs`：它能驱动 semantic manifest/block IO、CPU prepare 和 `ReadyCpu -> QueuedUpload`，但全量生产调用点搜索只找到模块导出及自身测试；`ResourceStreamer` 没有构造、admit、maintain 或领取 ready upload plan，也没有 RHI submit、completed receipt publication 和 draw accessor cutover。该内核不能计为产品集成完成。旧 `models/meshes/materials/textures` 永久 map 仍是 draw preparation 的资源来源，帧提交仍可能执行磁盘读取、完整反序列化、完整资产 clone、CPU 转换和 `wgpu` 创建，故 RAR-P0-002、09D P0-1、P0-5、P0-6 仍开放。

另一个明确问题是 exact type。`ResourceHandle<T>` 和 `Handle<T>` 都只序列化 `ResourceId`，untyped handle 只附带 broad `ResourceKind`；公开 `ResourceManager::store_payload<T>` 只校验 id/revision/Ready，不校验 catalog 中的 exact payload/schema。当前测试仍把 `ShaderAsset` 写入 Texture record 并期待 mutation 成功，随后 generic readiness、typed facade 和 `ensure_resident` 会产生不同事实。RAR-P0-001 仍开放。

本轮是 review-only：没有修改 Rust、Cargo、shader、ABI、Editor 或参考源码，没有运行 Cargo/GPU/device-loss/OOM/soak/benchmark。静态证据不能支持“性能或表现优于 Unreal”的结论。

## 2. 选择集与复核方法

### 2.1 当前物理冻结

选择集覆盖 `zr_resource` crate、interface resource contracts、core re-export、typed facade、ProjectAssetManager loading/resource sync、artifact access、RenderAssetResidency、ResourceStreamer、prepared/GPU resource 和 frame submission consumer。路径去重后的当前统计为：

| files | lines | non-empty | bytes | test attrs | ignored | `unsafe` tokens | dirty |
|---:|---:|---:|---:|---:|---:|---:|---:|
| **302** | **63,812** | **58,602** | **2,330,838** | **612** | **32** | **13** | **73** |

统计是本次终检时点的工作树词法/结构证据；73 个 dirty 文件来自共享工作树中的在途修改，本报告不把它们归属于本轮，也不将统计当成功能覆盖率。统计包含已被 `render_asset_residency/mod.rs` 纳入但仍为 untracked 的 `semantic_executor.rs` 及其测试；正式账本只使用上述去重 union，不把文件存在或模块导出等同于产品调用闭环。

### 2.2 逐步检查

1. 从 `ResourceMarker/ResourceRecord/ResourceHandle` 追到 mutation preflight、registry、payload/runtime slot、readiness generation、event stream、lease drop 和 facade query。
2. 从 `ProjectAssetManager::load_*` 追到 `ensure_resident -> prepared artifact read -> store_runtime_payload -> downcast/clone`，再追到 `ResourceStreamer::ensure_scene_resources` 和 frame submission。
3. 对 `RenderAssetResidencyManager` 逐文件检查 ticket issuance、reference delta、state transition、semantic block load、GPU plan/submit、completion、retirement、device recovery，并搜索所有生产调用点。
4. 将每个可见能力与 Unreal StreamableManager/AssetManager、Bevy AssetServer/StrongHandle/AssetEvent、Fyrox async ResourceManager/TimedEntry、Godot threaded loader/cache、Unity RenderGraph registry/pool 的真实代码对应，不以同名函数推断完成度。
5. 检查现有测试是否验证生产接线，特别区分 `include_str!` 源码形状断言、纯状态机单元测试和真实产品消费者。

### 2.3 关键证据位置

| 文件与行 | 证据 | 判定 |
|---|---|---|
| `zircon_runtime_interface/src/resource/resource_handle.rs:7-38` | Handle 只有 `ResourceId + PhantomData`，序列化跳过 marker；untyped conversion 只有 broad kind | 缺 owner epoch/exact type/schema/slot generation |
| `zircon_runtime_interface/src/resource/resource_record.rs:8-22` | Record 保存 kind/revision/state/dependencies/diagnostics，但没有 exact type id、schema id 或 provider generation | broad record 不能作为版本化 typed authority |
| `zircon_runtime/crates/zr_resource/src/manager/payload_ops.rs:30-40` | `store_payload` 直接提交 erased payload | public mutation 可绕过 exact type admission |
| `zircon_runtime/crates/zr_resource/src/manager/commit.rs:241-266` | StorePayload 只验证 record、revision、Ready，Replace 后标 Loaded | wrong payload 进入 authority |
| `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/ensure_resident.rs:11-102` | residency stripe 内同步读取 prepared artifact，并发布 runtime payload | 冷盘 IO/decode 在调用线程 |
| `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_typed.rs:15-27` | ensure 后 `asset.as_ref().clone()` | frame/consumer 复制完整 CPU 资产 |
| `zircon_runtime/src/asset/facade/assets.rs:23-35` | `get` 返回裸 Arc，`get_cloned` 深 clone，`acquire` 才创建 lease | untracked Arc 与 tracked lease 并行 |
| `zircon_runtime/crates/zr_resource/src/manager/lease_ops.rs:48-67` | 最后一个 lease drop 直接移除 payload（reload/error 除外） | 没有 TTL/cost/priority/cache policy，Arc 仍可在外部存活 |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs:47-162` | 每帧按 mesh/material/sprite/LUT/output 等调用 ensure | 资源需求仍由旧同步 consumer 驱动 |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_texture.rs:60-73,94-106` | `load_texture_asset_snapshot` 后准备完整 upload work，上传前发布 prepared texture | 未完成 fence 前即可被旧 accessor 使用；非 residency manager 路径 |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_mesh.rs:20-53` | cache miss 调 `load_mesh_asset`、转换 primitive、创建独立 GPU mesh | 一资源一 buffer，未消费 semantic GPU artifact |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_model.rs:20-55` | model revision/dependency 检查后 load/resolve primitives 并创建 GpuModelResource | 与 residency ticket 的 revision/device 体系平行 |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_residency.rs:48-127` | 只应用 scene deltas、retain request/release；没有构造或调用 semantic executor | QueuedIo 在产品桥中仍是 dead-end |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_residency.rs:181-220` | 只在 RHI poll 后维护已有 GPU residency submission | 没有读取/解码/prepare/upload 生产入口 |
| `zircon_runtime/src/graphics/scene/resources/render_asset_residency/mod.rs:7,38-44` | 未跟踪 `semantic_executor.rs` 已被生产模块声明并 re-export | 是在途源码，不是仅测试文件；仍需按 caller 证明集成 |
| `zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_executor.rs:255-389` | executor 可 admit/maintain semantic load、异步准备 upload plan，并由 `take_next_ready_upload` 交给外部 owner | 全量调用点只在自身测试；没有 ResourceStreamer owner、RHI submit 或 publication consumer |
| `zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_executor.rs:314-320` | admission 只接受 `SemanticBlocks` route | Model 的 `CanonicalMeshSet` 等 route 仍无执行路径 |
| `zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_blocks/load.rs:34-81` | semantic block loader 具备 begin/advance 协议 | 已被在途 executor 内核使用，但仍未接入产品 owner/draw consumer |
| `zircon_runtime/src/graphics/scene/resources/render_asset_residency/gpu_upload/submit.rs:324-355` | 可创建并上传 texture/mesh artifact、返回 submission ticket | 生产 draw accessors 仍读取旧 `Gpu*Resource` |
| `zircon_runtime/src/graphics/scene/resources/render_asset_residency/manager.rs:126-244` | reference delta 创建 QueuedIo entry，状态机有 Reading/Decoding/ReadyCpu/QueuedUpload/Uploading | 生命周期模型存在，但缺产品 owner 对 executor、typed payload source 与 submit/publication 的闭环 |
| `zircon_runtime/crates/zr_resource/src/event_stream.rs:12-14,707-717` | event log 4,096 条/4 MiB/60 秒，超限丢弃并报告 lag | 有界性存在，但产品消费者必须具备 gap resync |
| `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs:18-75` | Texture/UiIcon、UiLayout/V2View、UiStyle/Theme/V2Style 通过 fallback 链共用 broad kind | 同一 kind 映射多个 payload，exact type 不可审计 |

## 3. 可保留的工程底座

- `zr_resource` 已是 workspace crate，`ResourceAuthority` 将 registry、management/readiness projection、payload/runtime slot 放在同一 authority lock 内；prepare/apply mutation 和 event publish permit 能提供零变更失败边界。
- management/readiness generation 使用 immutable `Arc`、shard 和 reverse dependency closure，能够为后续 typed catalog/version snapshot 提供读取挂点。
- `ResourceLease` 的 identity token 能阻止旧 lease drop 误删新 slot；`ResourceSnapshot` 能把 payload 与 record revision 配对。
- artifact IO 有 atomic write、journal/recovery、BLAKE3 identity 和 bounded event/chunk diagnostics；应在其上增加 semantic manifest，不应重造另一套文件系统 authority。
- RenderAssetResidency 已有 reference delta preflight、duplicate/overflow/underflow 检查、device epoch、GPU submission frontier、retirement backpressure、partial artifact cleanup 和 failure report。
- texture/mipmap、mesh bounds、container layout、GPU upload rollback 和 revision cache 有可测试的 policy kernel；迁移应复用这些验证，不应把它们继续隐藏在同步 `ensure_*` 中。

## 4. 继承 P0 当前状态

本节不重复占有已有 canonical 编号，只给当前源码重判。

| canonical owner | 当前状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| RAR-P0-001 exact payload admission | **Open** | `store_payload<T>` 无 exact type/schema 参数；wrong-payload 测试仍允许 commit；`ensure_resident` 只检查任意 erased payload | `ExactAssetTypeCatalog`、typed mutation admission、schema/provider generation、失败零变更 |
| RAR-P0-002 frame cold load/clone | **Open** | `ensure_resident` 同步 artifact read；`load_typed` clone；frame/resource streamer 仍有多个 `load_*_asset` | typed nonblocking request、ready snapshot/fallback、稳定帧 IO/decode/clone=0 |
| 09D P0-2 semantic subresource streaming | **Open** | semantic block state/plan 有测试，但没有 ProjectAssetManager/artifact store/streamer production consumer | versioned manifest、随机 block reader、mip/lod/page descriptor、block checksum/codec |
| 09D P0-4 single residency authority | **Open** | `RenderAssetResidencyManager` 和在途 semantic executor 已存在，但 executor 未被产品 owner 构造/维护，旧 prepared maps 仍是 draw authority | 一个跨 CPU block/GPU handle/retirement/bytes/pin 的 owner |
| 09D P0-5 CPU/GPU lease ownership | **Open** | graphics 使用 clone load；prepared map 长期保留 CPU Arc 与 GPU object；`get` 返回裸 Arc | versioned CPU lease、bulk/source 分离、GPU upload 后按 policy 释放 |
| 09D P0-6 async generation/cancel/fence | **Open** | 状态枚举和 ticket 字段存在，但没有后台 load/decode/upload caller 或撤回路径 | request id、priority/deadline/cancel、asset/demand/device generation 全链校验 |

## 5. 当前 P1 差距与重构要求

| ID | 状态 | 当前差异与重构要求 |
|---|---|---|
| RAR224-P1-001 | Open | Record/Handle 只有 broad `ResourceKind` 和 `ResourceId`。引入 `ExactAssetTypeId + SchemaId + ProviderGeneration`，并让 typed handle 序列化 qualified identity。 |
| RAR224-P1-002 | Open | `store_payload`/`UpsertReady` 接受 erased payload。mutation preflight 必须比对 catalog exact type、artifact schema、kind、revision，错误 payload 必须保持 authority、generation、event 全不变。 |
| RAR224-P1-003 | Open | `get/get_cloned` 返回不受 authority 记账的 Arc/clone。公开 API 应区分 metadata、`VersionLease<T>`、短期 read guard 和显式 clone；refcount=0 只表示可驱逐，不能直接等价 unload。 |
| RAR224-P1-004 | Open | `ResourceLease` drop 直接删 payload，没有 TTL、bytes、cost、priority、owner/pin、eviction reason。实现 CPU cache policy 与 eviction receipt，reload 的 active/candidate/retired slot 必须可观测。 |
| RAR224-P1-005 | Open | `ensure_resident` 在 64 条 stripe 中持锁完成磁盘读取、解压、反序列化和 publication；改成 `AssetLoadCoordinator` single-flight request，锁内只做 admission，I/O/decode 在任务域执行。 |
| RAR224-P1-006 | Open | project generation 只在完整读取后二次检查，generation 变化会浪费整次 decode；请求必须携带 owner/project epoch，在读前、decode 后、publish 前取消/拒绝，并提供 deterministic retry。 |
| RAR224-P1-007 | Open | `load_imported_asset` 以 fallback 链把多个 payload 映射到一个 broad kind。每个 asset type 必须有唯一 catalog descriptor；兼容旧 type 只能通过显式 migration，不得用 `or_else` 猜测。 |
| RAR224-P1-008 | Open | dependency readiness 能做 reverse closure/SCC，但 load request 没有 typed dependency plan、priority inheritance、cycle diagnostic 或 dependency lease。实现 graph plan 后一次性 admission closure。 |
| RAR224-P1-009 | Open | event log bounded/coalesced 会产生 gap；Asset/Editor/Render consumer 必须持有 cursor、gap receipt 和 generation resync，不能将“收到最后一个 Modified”当作完整历史。 |
| RAR224-P1-010 | Open | RenderScene admission 只把 QueuedIo tickets 放入 `BTreeMap`。在途 `RenderAssetSemanticExecutor` 已实现 SemanticBlocks 的 manifest/block IO、decode 与 upload-plan prepare，但没有产品构造/admit/maintain/ready-plan caller，且拒绝其他 route。将它接为唯一 request executor，补齐 route、RHI submit、completed publication，并在每个状态转换返回 release/diagnostic。 |
| RAR224-P1-011 | Open | semantic block loader 仍未连接 artifact store；当前 chunk 只保证 content-addressed integrity，不能按 texture mip/layer、mesh LOD/cluster、material dependency block 随机读取。cook manifest 必须声明 semantic range、alignment、format、codec、dependencies。 |
| RAR224-P1-012 | Open | `ResourceStreamer` 的 `models/meshes/materials/textures/output/LUT/shader` map 与 residency manager 并行持有资源。迁移到 `PreparedRenderAssetGeneration`/residency handle，旧 map 只能作为兼容索引并有明确拆除期限。 |
| RAR224-P1-013 | Open | 旧 mesh/model/texture ensure 仍直接 clone payload、创建独立 wgpu object，并在 upload completion 前 publish prepared object。draw accessors 必须读取 residency artifact，且 publication 以 completed submission receipt 为条件。 |
| RAR224-P1-014 | Open | 目前只有 texture mip 预算，mesh buffers、CPU source、material uniform/bind group、sampler、SDF/VG、output/LUT 不在统一 bytes ledger。预算必须覆盖 compressed IO、decode scratch、staging、destination、old+new、in-flight fence 和 retire queue 峰值。 |
| RAR224-P1-015 | Open | residency policy 对 Model/Mesh/Texture/Material/Shader 仅有固定 scope/route，未覆盖 sprite/UI/lightmap/cookie/probe/particle/secondary view。由统一 view-family demand bus 合并 priority、pin、deadline 和 fallback。 |
| RAR224-P1-016 | Open | `resource_revision` 每次 ensure 查询 registry，model 还递归查 dependency；稳定帧应消费 changed-generation journal/reverse closure，轮询仅保留诊断接口。 |
| RAR224-P1-017 | Open | reload 直接按 revision 重建单个 prepared object，缺少 dependency closure candidate、atomic swap、last-good receipt、backoff 和旧 GPU artifact fence retirement。 |
| RAR224-P1-018 | Open | device epoch 只保护 residency ticket/submit 校验，未见生产 caller 将 device recovery 传播到旧 prepared maps、sampler cache、materials、history、surface 与重新 extract。实现全资源 stale/cancel/rebuild/re-extract。 |
| RAR224-P1-019 | Partial | event stream、project generation wake、submission poll receipt 和 GPU maintenance diagnostics 已有真实底座，但缺少跨 CPU/GPU request 的统一 trace id、queue/bytes/latency/fallback/eviction counters。 |
| RAR224-P1-020 | Partial | `zr_resource` 的 atomic file/journal/recovery 已接入 workspace，仍需明确 runtime artifact namespace、cook output namespace 和 live registry generation 的 durable receipt，避免 source/artifact/catalog 各自成功。 |

状态统计：**18 Open / 2 Partial / 0 Closed**。这些是本报告的实施分解，不改变 Runtime112/09D 的 canonical 计数。

## 6. P2 目标差距

| ID | 状态 | 目标 |
|---|---|---|
| RAR224-P2-001 | Open | persistent/soft/weak/strong handle、owner scope 和 cross-world lease 的完整 ABI；旧 project/device/provider epoch 永久 stale。 |
| RAR224-P2-002 | Open | CPU cache 分层（metadata/source/decoded/derived）、按 cost/priority/age/pin 的可预测 eviction 与压测报告。 |
| RAR224-P2-003 | Open | GPU mesh arena/suballocation、LOD/cluster/page streaming、compaction 和 fence-safe defragmentation。 |
| RAR224-P2-004 | Open | compressed texture partial residency、KTX2/BC/ASTC/ETC 平台 cook、GPU-ready block 及 fallback quality ladder。 |
| RAR224-P2-005 | Open | remote/DLC/package provider、IO bandwidth scheduler、offline prefetch 和 cache namespace/eviction。 |
| RAR224-P2-006 | Open | virtual texture/VG page demand 与 scene visibility/HZB/occlusion/camera prediction 的统一 feedback loop。 |
| RAR224-P2-007 | Partial | device recovery、submission frontier、artifact retirement 已有局部协议；缺跨 queue、multi-GPU/UMA 和 platform memory pressure 证据。 |
| RAR224-P2-008 | Partial | bounded event/generation diagnostics 已存在；缺 persistent replay/capture、asset request timeline 和 deterministic soak corpus。 |
| RAR224-P2-009 | Open | shipping cook gate：禁止 source image/model parse、runtime mipgen、不可预算 transcode；缺 artifact 时 cook 失败而不是玩家首帧失败。 |
| RAR224-P2-010 | Open | Editor preview/details 使用同一 runtime ticket/version/catalog generation，preview pin 与 game demand 不能复制 authority。 |

状态统计：**8 Open / 2 Partial / 0 Closed**。

## 7. 参考引擎差异

| 参考 | 已有工程机制 | Zircon 当前差异 |
|---|---|---|
| Unreal StreamableManager/AssetManager | `RequestAsyncLoad` 返回可持有 handle，具备 progress、priority、cancel/release、stalled start、primary asset bundle load/unload 和 management rules | Zircon 没有产品级 load request/handle、progress/unload 与统一 owner；`load_*` 是同步 Result，在途 residency executor 未被产品调用 |
| Unreal AsyncLoading2 | package IO、依赖递归、async package state、flush/timeout/device-independent loader boundary | Zircon 的 project artifact read 在调用线程完成，dependency readiness 不是可调度 package graph |
| Bevy AssetServer/StrongHandle | 强句柄 drop 发送 drop event，AssetServer 统一 load sharing、Loaded/Failed/Unused/Removed 事件和 handle type checking | Zircon `Handle<T>` 可复制但不携带强弱/owner/type metadata；裸 Arc 不产生 unused 事件，event gap 也没有强制 resync consumer |
| Fyrox ResourceManager/TimedEntry | request 非阻塞、loader task pool、Future wait、资源共享、unused TTL 后释放 | Zircon 没有 Future/request，最后 lease drop 立即删除 payload，缺 TTL/cost/loader task owner |
| Godot ResourceLoader | `load_threaded_request/status/get`、LoadToken、cache mode、clear threaded tasks、missing-resource policy | Zircon 没有 threaded request/status/get/cancel token；asset cache 与 renderer cache 分裂 |
| Unity RenderGraph registry/pool | frame execution 中统一 create/import/release，跨帧 pool、last-use、purge、异常清理、显式 release callback | Zircon residency manager 有 retirement queue，但旧 ResourceStreamer map 仍是 draw source；创建/发布/释放不由一份 frame/resource lease 控制 |

## 8. 目标架构与实施顺序

### 8.1 Authority contract

1. 先完成 `ExactAssetTypeCatalog`：record 绑定 exact type/schema/importer/provider generation，所有 Upsert/Store/Reload mutation 在 preflight 比对，错误零变更。
2. 把 `ResourceVersionSlot` 拆成 active/candidate/retired；`ResourceSnapshot` 和 `VersionLease<T>` 只读 immutable version，裸 Arc 不作为长期产品合同。
3. 将 `AssetLoadCoordinator` 放在 runtime resource/asset absorption 层：request 携带 owner、priority、deadline、cancel token、project/catalog generation、dependency plan 和 cache policy；同一 qualified version single-flight。

### 8.2 IO/decode 与 CPU cache

1. 在既有 artifact transaction/store 上发布 `RenderArtifactManifest`，header/metadata 与 texture mip/layer、mesh LOD/cluster、material block 分离，block 具 content id、compressed/decoded bytes、range、alignment、codec、platform format 和 dependency。
2. ProjectAssetManager 只负责 catalog/provider admission；任务 lane 负责 async read/decode/transcode。render thread 只能领取已完成 `ReadyCpu` snapshot 或显式 fallback。
3. CPU cache 分 source/decoded/derived 三类计量，lease/ref/pin/priority/age 进入 eviction policy；generation 改变时取消旧 request，last-good 继续服务。

### 8.3 Render residency hard cutover

1. 让 `ResourceStreamer::apply_render_asset_residency_reference_deltas` 产生 demand，唯一 executor 消费 `pending_render_asset_residency_requests`，驱动 semantic load -> GPU upload plan -> RHI submission -> completed artifact publish。
2. GPU draw accessors 改为读取 generation-qualified residency artifact；旧 `Prepared*` map 只能引用 immutable artifact handle，禁止从 map 再触发 `load_*`。
3. 所有 upload 先 reserve peak bytes，完成 submission 后才发布新 artifact；旧 artifact 进入 fence retirement。device loss、project close、scene unload 统一走 cancel/stale/retire/re-extract。
4. 由 RenderScene/visibility 输出 view-family demand，合并 mesh/material/texture/sprite/UI/lightmap/cookie/LUT 等需求；RenderGraph/Submission receipt 提供 terminal commit/rollback。

### 8.4 依赖顺序与禁止事项

实现顺序固定为：

```text
M224.1 exact type/schema admission
 -> M224.2 qualified version slot + VersionLease/cache
 -> M224.3 AssetLoadCoordinator + artifact semantic manifest/async block reader
 -> M224.4 CPU cache and dependency/reload/cancel receipts
 -> M224.5 RenderAssetResidency executor and single draw consumer
 -> M224.6 device-loss/project-close/scene-unload retirement
 -> M224.7 demand feedback, compression/LOD/VG and scale gates
```

禁止先给同步 `load_*` 套一层 async facade；禁止继续扩大 `ResourceStreamer` 永久 HashMap；禁止在 semantic manifest 未可读、exact type 未 admission、GPU completion 未 receipt 前宣称“streaming 完成”；禁止用 warning、fallback 或一个 `Arc` clone 掩盖超预算峰值。

## 9. 验收门禁

1. wrong payload/type/schema/kind/revision/owner epoch 矩阵：失败后 registry、payload、runtime/readiness generation、event sequence 全不变。
2. single-flight/priority/cancel/deadline/gap-resync：同一 qualified version 只产生一次 IO/decode/upload；取消、项目切换、reload、device loss 不可发布旧代。
3. cold/warm/reload/teleport/visibility churn：稳定帧 filesystem/decompress/full asset clone/runtime source parse/GPU create 必须为 0；只允许读取 ready snapshot/fallback。
4. semantic block：单 mip/LOD/cluster 请求不得读取或反序列化完整 asset；compressed format 必须有平台 cook artifact 和 partial upload 证据。
5. GPU residency：所有 texture/mesh/material derived allocation、staging、old+new、in-flight submission、retirement 都进入 peak ledger；超预算必须 delay/degrade，不能先分配后 warning。
6. draw safety：residency artifact 只能在 completed submission receipt 后发布；旧 generation 在 GPU 使用结束前保持可读，retirement 失败可重试且有界。
7. project close/device recovery：停止 admission、取消 owner requests、drain callbacks、标记旧 handle stale、退休 GPU/CPU version，并重新 extract 必需资源。
8. scale/performance：100K assets、10K visible instances、multi-view、OOM、fault、device-loss、60 分钟 reload/stream soak；同时采集 hitch p50/p95/p99、CPU/RSS/VRAM、IO/decode/upload bandwidth、residency churn、dropped events 和 fallback rate。

## 10. 当前状态与下一执行切片

- canonical inherited P0：RAR-P0-001、RAR-P0-002、09D P0-2/P0-4/P0-5/P0-6 均保持 **Open**。
- 本报告新增实施分解：P1 **18 Open / 2 Partial**，P2 **8 Open / 2 Partial**；不改变既有 Runtime112/09D canonical 计数。
- `zr_resource` workspace/Cargo 边界已修正，不能再把“未入 workspace”当作当前 finding；在途 semantic executor 内核也已出现，但资源请求 owner、完整 route、RHI submit/completion publication 和 draw consumer 仍缺失。
- 本轮仅 review/index/coverage 文档，未修改 Rust/Cargo/ABI/shader/UI，未运行 Cargo/GPU/benchmark；Tooling 按用户要求排除，也未查询、轮询、等待或实时跟踪协调器。
- 下一切片必须从 M224.1 exact type/schema admission 开始；在其通过行为、并发、失败零变更和 ABI 测试前，不得开始 renderer hard cutover。
