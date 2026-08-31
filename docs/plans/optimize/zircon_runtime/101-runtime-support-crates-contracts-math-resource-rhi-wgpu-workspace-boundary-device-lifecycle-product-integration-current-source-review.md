---
title: Runtime Support Crates、Contracts、Math、Resource、RHI、WGPU、Workspace Boundary、Device Lifecycle 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime153
review_date: 2026-08-29
baseline_head: a832f97403033c08e8ed60967c717d25145621dd
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/crates/zr_contracts
  - zircon_runtime/crates/zr_math
  - zircon_runtime/crates/zr_resource
  - zircon_runtime/crates/zr_rhi
  - zircon_runtime/crates/zr_rhi_wgpu
  - zircon_runtime/src/core/runtime/random
  - zircon_runtime/src/graphics/backend/render_backend
tests:
  - zircon_runtime/crates/zr_contracts/src/random/tests.rs
  - zircon_runtime/crates/zr_math/src/tests
  - zircon_runtime/crates/zr_resource/src/tests
  - zircon_runtime/crates/zr_rhi/src/tests
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/tests
plan_sources:
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/DynamicRHI.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHIResources.h
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Submission.h
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Allocation.cpp
  - dev/godot/servers/rendering/rendering_device_driver.h
  - dev/godot/drivers/vulkan/rendering_device_driver_vulkan.h
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_render/src/renderer/render_device.rs
  - dev/bevy/crates/bevy_render/src/gpu_readback.rs
  - dev/Fyrox/fyrox-core/src/pool/handle.rs
  - dev/Fyrox/fyrox-graphics/src/read_buffer.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourcePool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
---

# Runtime Support Crates、Contracts、Math、Resource、RHI 与 WGPU 当前源码工程化差距

## 1. 结论

这五个 support crate 已经不是“只有类型名”的空壳。当前工作树中存在可保留的 deterministic random checkpoint/lease、checked math、resource transaction journal、generation-qualified RHI handle/submission/surface、WGPU production registry 与 fail-closed capability mapping。Runtime90 关于“没有 production `RenderDevice`”的旧结论已经被当前源码部分推翻，不能继续照搬。

但它们还不能被认定为工程级 foundation，首要原因不是功能数量，而是**当前实现没有形成可复现、单一所有权、可验证的产品边界**：287 个输入中只有 78 个已被 Git 跟踪，277 个处于 modified/untracked；`zircon_runtime` 已直接引用 `zr_contracts`，根 workspace 和 runtime manifest 却都没有声明该 crate；graphics 产品层同时保存 `WgpuRenderDevice` 与裸 `wgpu::Device/Queue`，大量资源继续绕过中立 RHI 创建；neutral readback ABI 是同步 `Vec<u8>`，production WGPU 实现直接返回 unavailable。

本报告登记 **2 项 P0、14 项 P1、4 项 P2**，以及 **24 项资格门：20 Fail / 4 Partial / 0 Pass**。这是 Runtime153 的局部复核计数；与 Runtime22/23/24/64/90 重叠的 canonical finding 仍由原报告拥有，不在全局总账重复加总。

性能方面也不能宣称优于 Unreal：当前没有 clean-clone build evidence、统一 GPU readback、单 owner resource path、多队列实现或受管压力基线。任何“性能更好”的结论都必须在 P0/P1 结构门关闭后，通过相同场景、相同画质、相同硬件的帧时间、CPU submit、GPU bubble、显存峰值、资源 churn 与 device-loss 恢复证据建立。

## 2. 冻结范围与证据口径

统计包含各 crate 下所有 `.rs` 与 `Cargo.toml`。fingerprint 口径为 normalized lower-case repo-relative path、文件 SHA-256，按路径排序后以 `path<TAB>hash`、LF 拼接并再次 SHA-256。dirty 由当前工作树的 modified/untracked 状态计算，不表示这些修改由本报告产生。

| 范围 | files / lines / nonempty / bytes / tests / ignored / unsafe | tracked / dirty | fingerprint |
|---|---:|---:|---|
| `zr_contracts` | 15 / 866 / 752 / 25,899 / 9 / 0 / 0 | 0 / 15 | `5907307f22dc91c5f90fa4bb88707f18625cd41bba1ff8d91f4289122b28dd15` |
| `zr_math` | 14 / 1,488 / 1,306 / 44,264 / 24 / 0 / 0 | 0 / 14 | `5486cb7de1fc98399bcdb27ae6956f502e4554c57ee235cd619347cfcd963592` |
| `zr_resource` | 71 / 14,936 / 13,624 / 513,421 / 175 / 3 / 4 | 0 / 71 | `7f38c33c2796add609b48f4698018e20b86eb61c6f23d1a07392731a1d6e3867` |
| `zr_rhi` | 39 / 10,737 / 9,686 / 352,010 / 80 / 0 / 0 | 16 / 34 | `6fba3dc1d00c577cd46e4a08fdd6693d3c81f52dfcdae08b55b469fab56a767b` |
| `zr_rhi_wgpu` | 148 / 43,685 / 40,728 / 1,570,032 / 389 / 0 / 2 | 62 / 143 | `acd62bc6dbbc6988d59d0aac543cf250d6e3a46748b5d4828519063829ba9429` |
| Zircon union | **287 / 71,712 / 66,096 / 2,505,626 / 677 / 3 / 6** | **78 / 277** | `60c1c22df1af1dada577d6d6e42ba4bb8cbbfdbb046a93274efae1a5d6ebf6ce` |
| 16 个本地参考文件 | **16 / 18,284 / 15,453 / 724,419 / 26** | n/a | `5d16e60eaeb3c1d04ff8247869393eb0f739e87d71d2341364d27918d8dc5334` |

本轮逐文件完成物理清单、manifest/tracking/test/unsafe 扫描，并对所有 production owner、产品消费路径及风险实现逐段复核。未运行 Cargo、GPU、device-loss、fault、scale、soak 或跨引擎动态 benchmark；这是 review 和重构计划，不是实现验收。Tooling 按用户要求排除，也未查询、轮询、等待或实时跟踪协调器。

## 3. 当前真实所有权图

```text
zr_contracts (random DTO/checkpoint, untracked, not a workspace member)
        ^
        | source import exists, Cargo dependency does not
zircon_runtime::core::runtime::random (service/registry/lease, mostly untracked)

zircon_runtime -> zr_math / zr_resource / zr_rhi / optional zr_rhi_wgpu
                                      |
                                      v
graphics::RenderBackend
  |- Arc<WgpuRenderDevice>     intended neutral generation owner
  |- wgpu::Device              second product-visible native path
  `- wgpu::Queue               second product-visible native path
       |- OffscreenTarget owns raw WGPU textures/buffer
       `- SurfaceBlitResources builds raw WGPU sampler/layout/pipeline
```

目标不是把每个 native 对象隐藏到一个大 facade，而是让 native 创建、代际验证、最后使用、延迟销毁、读回与提交都由同一个 generation owner 记账。上层只能持有中立 handle、immutable descriptor/artifact 与 generation-qualified receipt；确需 native interop 时，必须通过有生命周期、能力和线程约束的显式 lease。

## 4. P0：当前不能进入 clean build/集成候选

### Runtime153-P0-001：五个支撑 crate 的当前能力无法由干净 checkout 重现

**证据**

- `zr_contracts`、`zr_math`、`zr_resource` 的 100 个输入全部 untracked；`zr_rhi` 39 个输入仅 16 个 tracked；`zr_rhi_wgpu` 148 个输入仅 62 个 tracked。
- `zr_rhi_wgpu/src/production/` 整目录 untracked；当前被 `src/lib.rs` 导出的 production `WgpuRenderDevice`、registry、submission、surface 与 diagnostic code 因而不属于可审计基线。
- 总计 287 个输入中 277 个 dirty。工作树可以表现出“已实现”，干净 clone、CI、bisect 和其他开发者却无法获得同一源码图。

**风险**

- 任何测试通过都只证明本机瞬时工作树，不证明仓库产品。
- 基于未跟踪 implementation 写出的 capability、文档和性能结论会成为 false-green。
- `git bisect`、release source archive、SBOM 与安全审计无法定位实际实现。

**必须重构/关闭条件**

1. 以一个原子 integration candidate 纳入 manifest、source、tests、lockfile 变化；禁止只提交 facade 或只提交 tests。
2. 在 clean clone 上验证 workspace metadata、targeted check/test、feature matrix 与 source guard。
3. 生成受管输入 manifest 与 fingerprint；任何后续变更必须更新证据，不得沿用本报告快照。

### Runtime153-P0-002：`zr_contracts` 已被生产源码消费，但 Cargo 图中不存在

**证据**

- 根 `Cargo.toml:2-10` 声明 `zr_math/zr_resource/zr_rhi/zr_rhi_wgpu`，没有 `zr_contracts`；workspace dependencies `Cargo.toml:58-61` 同样没有它。
- `zircon_runtime/Cargo.toml:243-246` 只声明上述四个 crate。
- `zircon_runtime/src/core/runtime/runtime.rs:9` 与 `zircon_runtime/src/core/runtime/random/service.rs:3-6` 已直接导入 `zr_contracts::random`。
- crate 自身 manifest 存在，但目录完全 untracked；当前 source graph 与 Cargo graph 不一致。

**风险**

- Rust 编译器无法通过正常依赖解析看到该 crate；当前随机服务切片不具备构建闭环。
- 若临时把类型复制回 runtime 或增加 facade alias，会产生双 owner、双 serde schema 和 checkpoint 不兼容。

**必须重构/关闭条件**

1. 将 `zr_contracts` 作为明确 workspace member 和 workspace dependency 原子接入，或在切片未就绪时删除生产 import 并回退整个切片；不允许半接线。
2. 对 `zr_contracts` 建立禁止依赖 `zircon_runtime`/graphics/WGPU 的 DAG guard。
3. 用 serialization golden、checkpoint migration 与 clean `cargo check` 证明 contract/service 双侧一致。

## 5. P1：工程边界与产品闭环差异

### Runtime153-P1-001：RHI generation owner 仍被裸 WGPU product path 绕过

`WgpuRenderDevice` 自称唯一 production owner，并在 `production/device.rs:51-73` 持有 instance/adapter/device/queue、generation registry、submission、surface 与 diagnostic services；但 `graphics/backend/render_backend/render_backend.rs:7-18` 同时保存 `Arc<WgpuRenderDevice>`、`wgpu::Device` 和 `wgpu::Queue`。`offscreen_target.rs:4-33` 直接拥有十余个 native texture/view/buffer，`offscreen_target_construct/construct.rs:13-31` 直接用 `wgpu::Device` 创建资源，`viewport_surface.rs:253-305` 又直接创建 sampler/layout/shader/pipeline。

这不是单纯“封装不漂亮”：这些资源不会天然进入 neutral handle generation、memory budget、last-use ticket、device-loss invalidation 与 deferred retirement。应将 offscreen/surface blit/system texture 全部硬切到 RHI descriptor/handle；仅 surface/raw-window interop 可保留受限 native lease。

### Runtime153-P1-002：neutral readback ABI 是同步返回值，production backend 实际不可用

`zr_rhi/src/device/render_device.rs:291-297` 要求 `read_buffer/read_texture -> Result<Vec<u8>>`，没有 request identity、ticket、row pitch、format、subresource、budget、cancel、map completion 或 owner epoch。production WGPU 在 `production/device.rs:716-737` 验证 handle 后固定返回 `ReadbackUnavailable`，capability 也在 `production/device/capabilities.rs:96` 明确为 false。

应删除同步返回字节的核心 ABI，建立 `ReadbackRequest -> accepted ticket -> copy submission -> mapped artifact/terminal receipt`。diagnostic readback 可复用预算和状态机，但普通 buffer/texture、capture、screenshot、streaming 与 editor inspection 必须共享同一 owner，不得各自维护 staging path。

### Runtime153-P1-003：默认 submission wait 通过 yield loop 主动轮询

`RenderDevice::wait_for_submission` 在 `render_device.rs:241-257` 循环 `poll_submissions`、查询状态并 `std::thread::yield_now()`。它有 timeout，但仍把调用线程变成 completion pump；若在 render/game/editor UI 线程误用，会制造高 CPU 占用、调度抖动和隐式 device polling authority。

应改为显式 nonblocking poll + executor/waker integration；blocking wait 只能存在于明确命名的 off-thread diagnostic/test adapter，并由 thread-role guard 拒绝主线程调用。

### Runtime153-P1-004：device/profile/ticket identity 仍可由普通调用者伪造

`RenderDeviceProfile::new` 在 `device_profile.rs:541-564` 直接接收并保存 device ID、generation、feature negotiation、limits、queue topology 与 budgets，没有统一 `try_new` admission。多种 ID/generation/ticket constructor 同样公开。backend 虽在 `WgpuRenderDevice::new` 做若干 context 验证，但中立层仍允许构造互相矛盾或非权威的 profile/receipt。

应由 `DeviceAdmissionAuthority` 独占 ID/generation 分配，公开侧只反序列化 candidate，经过验证后得到不可伪造 token；submission/surface/readback ticket 只能由对应 generation service 产生。

### Runtime153-P1-005：资源与 pipeline descriptor 在中立层可处于非法状态

`zr_rhi/src/descriptors.rs:68-76` 的 buffer size/usage、`230-238` 的 texture dimensions/mips/sample count/format/usage、`408-416` 的 sampler LOD/filter 都是 public field；构造器返回 `Self`。零尺寸、非法 sample/mip、非 finite LOD、usage/format 冲突会一直传播到 backend-specific validator。

工程级 RHI 需要两级 admission：backend-neutral structural validation 产生 `Validated*Desc`，provider negotiation 再产生 immutable `Compiled*Desc/CreationPlan`。这样相同错误不会在 WGPU、未来 D3D12/Vulkan 与 mock backend 中产生不同语义。

### Runtime153-P1-006：neutral `RayTracing` vocabulary 没有可执行 provider contract

neutral `PipelineKind` 暴露 `RayTracing`，production registry 在 `production/registry/pipelines.rs:147-170` 明确返回“not implemented by the WGPU production backend”。fail-close 是正确行为，但上层仍可能把“enum 存在”误读成产品能力。

应把硬件 RT 变成 provider-owned extension contract：capability negotiation 返回具体 pipeline/SBT/acceleration ABI version，只有被选择 provider 才能解析对应 descriptor。WGPU backend 保持 unsupported；未来 D3D12/Vulkan owner 不能借用 WGPU 类型伪装支持。

### Runtime153-P1-007：ResourceManager 用一把 `RwLock` 同时拥有所有 hot/cold state

`resource_manager.rs:18-38` 将 registry、management projection、payload、runtime slot、readiness 与 token allocator 放在一个 `ResourceAuthority`；`108-169` 用单一 `Arc<RwLock<_>>` 和 commit mutex 保护。最常见的 `acquire` 在 `lease_ops.rs:11-36` 也取得 write lock、downcast payload、增加 refcount并刷新 readiness；lease drop 再取得同一 write lock并可能删除 payload。

这会把并行读、lease churn、registry commit、reload 与 readiness projection 串到同一临界区。应分离 immutable registry generation、sharded residency slots、payload ArcSwap/RCU view、mutation serial lane 和 projection publisher；hot acquire 不得重建/排序 readiness 更新。

### Runtime153-P1-008：resource foundation 没有一等 load request/dependency/budget/cancel owner

当前 runtime slot 只记录 state、residency token 与 refcount；manager 能 register/get/store/acquire/commit，但没有 generation-qualified `LoadRequestId`、priority、deadline、owner/principal budget、dependency DAG lease、cancel reason、worker admission、artifact provenance 或 terminal receipt。上层 asset/graphics 各自补装载逻辑后，资源 currentness 和内存回收无法由一个 authority 证明。

应建立 `RuntimeResourceService`：request admission、dependency closure、loader/provider dispatch、artifact verify、staging payload、atomic generation publish、lease/ref pin、reload fallback、cancel/retire 都从同一 journal 派生。Runtime64 继续拥有完整产品状态机，本报告只负责 crate boundary 和并发模型。

### Runtime153-P1-009：resource event stream 只能报告 lag，不能恢复一致视图

`event_stream.rs:10-14` 固定 4,096 entries、4 MiB、60 秒；日志按 resource identity coalesce。receiver 可以得到 `Lagged(ResourceEventGap)`，但没有 snapshot generation、ack、resume token、durable cursor 或 resync API。新 subscriber 从当前尾部开始，也不能重建此前状态。

应将事件定义为 generation projection：subscriber 先取得 immutable snapshot + sequence fence，再消费 delta；lag 后必须强制 resnapshot。需要 durable replay 的 import/cook/recovery 事件进入独立 journal，不能把 in-process UI notification log冒充持久事件源。

### Runtime153-P1-010：atomic transaction 的 durability 在平台间不是同一合同

Unix 会在 `io/atomic_file/platform.rs:7-24` fsync directory/parent；非 Unix 实现在 `12-29` 直接 `Ok(())`。Windows rename 使用 `MoveFileExW(...WRITE_THROUGH)`，replace 使用 `ReplaceFileW`，并只在 Windows 对 committed target `sync_all`；非 Windows `sync_committed_target` 又是 no-op（`64-75`）。

这些实现可能是各平台可用的最佳机制，但公开合同必须区分 `AtomicVisibility`、`FileDataDurability`、`DirectoryEntryDurability` 与 `BestEffort`，并返回 platform evidence。不能让同一个“durable commit”名词在不同平台静默降级。

### Runtime153-P1-011：runtime-to-render narrowing 当前是恒等转换

`zr_math/src/lib.rs:41-52` 将 `Real`、`RenderScalar` 及其向量/矩阵全部定义为 f32。`render_conversion.rs:77-101` 随后检查 f32 是否落在 f32 范围，再把 f32 cast 为 f32；`absolute_error` 在有限值上恒为 0。该 receipt 看似证明了精度边界，实际上没有任何 narrowing。

应明确选择：若 runtime 仍固定 f32，就删除虚假的 narrowing 语义，只保留 finite/admission receipt；若目标包含 large world/f64 simulation，则 `Real=f64`、render=f32，转换必须记录 origin rebasing、range、误差与 overflow policy。

### Runtime153-P1-012：`STRICT` 数值策略允许近退化值，非 fallible API 又静默修正

`numeric_policy.rs:35-41` 将三个 minimum 全设为 0，并称为 strict；这只拒绝 exact zero，不拒绝 f32 下会放大误差的 near-zero determinant/norm/scale。与此同时，infallible perspective/affine/look-at 路径存在 clamp、fallback、normalize-or-zero，调用者拿不到退化诊断。

应建立按域命名的 policy registry（simulation/render/import/editor），阈值由精度、世界尺度和操作类型决定；production authoring/import/compiler 使用 fallible API 并携带 source path，infallible convenience 仅允许在经过 validated type 封印后的内部热路径。

### Runtime153-P1-013：空间安全只是一枚运行时 tag，没有 transform graph authority

`Position3/Vector3/Normal3` 阻止部分同类型误用，但 `SpaceKind` 是动态枚举，缺少 compile-time source/destination relation、world/view/clip/local transform edge、origin epoch 与 conversion receipt。调用者仍可给值贴错 tag，或绕过 wrapper 使用裸 glam 类型。

应以 typed `Transform<From, To>` 或稳定 space ID + validated graph 为边界；每次跨空间转换校验 generation/origin/convention。GPU packet 只能消费已转换到声明 render space 的值。

### Runtime153-P1-014：`zr_contracts` 物理 crate 只切出 random，尚未达到声明的跨域 contract owner

`zr_contracts/src/lib.rs:1-5` 只有 `pub mod random`；random module 注释明确执行留在 Runtime，这是正确方向，且当前 `RandomService` 已有唯一 lease、bounded registry、checkpoint/reseed exclusion 的真实实现。但架构计划要求的按域 feature、render/UI/native surface、lifecycle 与其他低层 DTO 尚未迁入，现有 crate 也没有 feature gate。

不能为了“填满 crate”批量搬动 `core/framework`。应继续按依赖 DAG 做小型 hard cut：先 contract schema/golden，后 runtime implementation consumer，再删除旧 owner；每一批都验证 contracts 不依赖 kernel/graphics/editor。

## 6. P2：长期可扩展性与性能资格

### Runtime153-P2-001：precision、depth 和 convention 被编译期固定为单一策略

当前 math 固定 f32、column vector/column-major、zero-to-one、near-to-far。它适合建立默认合同，但不足以表达 reversed-Z、infinite far、large-world origin、offline f64 或 backend-specific clip transform。应保留 canonical world convention，同时把 render projection policy 和 conversion receipt 显式化。

### Runtime153-P2-002：WGPU owner 的 registry/diagnostic/surface 为粗粒度 mutex

`production/device.rs:56-73` 用三个 mutex 包住 registry、diagnostic readback、surface service；所有资源创建/销毁和 last-use 记录最终串行。此处不能只凭静态代码宣判性能失败，但它是必须基准化的热点。重构前先采集 lock hold/wait、resource churn、command record concurrency；只有数据证明后再分 shard、frame arena 或 deferred command lane。

### Runtime153-P2-003：当前 WGPU topology 诚实地只有一个物理队列，但没有可替换多队列 owner

`RenderDeviceQueueTopology::single_serialized_queue` 在 `device_profile.rs:502-524` 明确 async compute/copy 为 false。这比伪报支持更正确，但 engine-level scheduler 还不能把 graphics/compute/copy workload 映射到 D3D12/Vulkan 多队列、ownership transfer、timeline semaphore 与 cross-queue retirement。需要 provider-neutral queue plan，而不是让 WGPU 约束固化所有 backend。

### Runtime153-P2-004：`#[doc(hidden)] pub assembly` 是可调用的信任面

`zr_contracts` 与 `zr_resource` 都暴露 hidden public assembly module。隐藏文档不等于限制访问；任何 workspace crate 都可绕过 curated facade 调用内部 constructor。物理迁移完成后，应以 private module、sealed trait 或单独 integration crate 收口，不把“内部约定”当成类型系统边界。

## 7. 可保留底座

以下实现应迁移和加固，不应推倒重写：

1. `RandomService` 的 deterministic derivation、stable stream key、唯一 mutable lease、active-lease reseed/checkpoint exclusion、canonical checkpoint 顺序。
2. `zr_math` 的 finite checks、fallible affine inverse/perspective、validated transform/unit direction 类型。
3. `zr_resource` 的 stable locator/ID、typed handle/lease、staged mutation preflight、transaction journal、staging sibling、digest evidence 与 recovery。
4. `zr_rhi` 的 operation matrix fail-close、device/generation-qualified handle、submission ticket/history、surface session/frame、fault gate 与 bounded diagnostic readback状态机。
5. `zr_rhi_wgpu` 的 adapter/device/profile context validation、generation-local registry、last-use ticket retirement、single native queue submission owner和 production capability fail-close。

这些基础目前的状态是 **source-present / integration-incomplete / qualification-pending**，不能写成 implemented/complete。

## 8. 与参考引擎的实质差异

| 参考 | 可借鉴的工程边界 | Zircon 当前差异 | 不应照搬的部分 |
|---|---|---|---|
| Unreal RHI / D3D12RHI | DynamicRHI provider、resource lifetime、command context、submission queue、allocation/retirement 与 device-loss 分层 | neutral RHI 与产品 raw WGPU 并存；没有 native D3D12/Vulkan provider、多队列或统一 readback artifact | Unreal 宏/全局单例/平台历史包袱不适合直接移植到 Rust owner 图 |
| Godot RenderingDeviceDriver / Vulkan | driver-owned resource IDs、command/sync primitive、Vulkan backend显式实现 | Zircon descriptor validation与 provider admission未分层，产品仍可直接创建 native WGPU 资源 | Godot面向自身 server架构，不能替代 Zircon generation/lease 合同 |
| Bevy RenderDevice / GPU Readback / Asset Handle | typed strong handle、drop event、native device wrapper、map/readback异步流程 | Zircon ResourceLease缺完整 load/dependency request；neutral RHI readback仍同步且 production unavailable | Bevy RenderDevice 本身较薄，不能作为完整多后端 RHI 的终点 |
| Fyrox pool Handle / graphics read buffer | typed generational index、pool stale rejection、明确 read-buffer future/owner | Zircon已有generation handle底座，但 resource/RHI identity constructor和产品绕过仍削弱 authority | Fyrox规模与backend范围不能直接证明 AAA 级吞吐 |
| Unity Graphics RenderGraph pools/registry | transient pool、resource registry、lifetime和command buffer边界 | Zircon WGPU registry还没有与 render graph transient alias/pool形成唯一产品创建路径 | C# managed lifetime策略不应机械复制到 Rust |

参考源码只能证明成熟系统如何划分责任，不能直接证明 Zircon 的性能目标。最终决策以 Zircon 的 clean-build、correctness oracle、trace 与 benchmark 为准。

## 9. 目标架构

```text
zr_math
  canonical simulation types + validated transforms + explicit render conversion

zr_resource
  immutable registry generation
  + sharded residency/lease table
  + async request/dependency service
  + durable transaction/artifact boundary

zr_contracts (pure DTO/trait, domain features, no runtime implementation)
  random / lifecycle / render interop / resource event projection / ...

zr_rhi
  validated neutral descriptors
  + unforgeable device-generation authority
  + command/submission/readback/surface tickets
  + provider extension negotiation

zr_rhi_wgpu
  sole WGPU generation owner
  + registry/submission/readback/surface implementation
  + bounded native interop leases only

graphics/render graph/editor/capture
  immutable artifacts + neutral handles + generation-qualified receipts
  (no stored wgpu::Device/Queue/Texture/Buffer/Pipeline)
```

## 10. 重构里程碑

### M0：恢复 source/Cargo 可复现性

- 原子纳入五 crate 当前 source/tests/manifest，或撤回未完成切片；禁止混合状态。
- 接入 `zr_contracts` workspace/dependency，关闭 P0-002。
- clean clone 执行 metadata、targeted check/test、feature matrix、source manifest fingerprint。

### M1：封印 identity、profile 与 descriptor admission

- 引入 `DeviceAdmissionAuthority` 与不可伪造 generation token。
- 将 public-field descriptor 改为 candidate builder + `try_validate` + immutable validated descriptor。
- serializer 只生成 candidate；任何反序列化数据必须重新 admission。

### M2：RHI/WGPU 单 owner hard cut

- 迁移 offscreen target、surface blit、system texture 和 product upload/readback 到 neutral handles。
- 从 `RenderBackend` 删除裸 `wgpu::Device/Queue` 字段。
- native surface/external image 只通过 scoped interop lease，记录 generation、thread、capability 与 last-use ticket。

### M3：统一异步 readback 与 completion

- 建立 buffer/texture subresource request、layout/format receipt、copy ticket、map completion、cancel/timeout、budget与 artifact ownership。
- diagnostic、capture、Editor inspection 共用 service；删除同步 `Vec<u8>` ABI和 yield-loop 主线程等待。

### M4：资源 authority 拆分

- immutable registry generation 与 hot residency table 分离。
- 建立 load request、dependency DAG、priority/deadline、owner budget、cancel、reload、last-good 与 terminal receipt。
- event subscriber 使用 snapshot+fence+delta；lag 必须 resnapshot，durable事件进入 journal。

### M5：数学与空间合同收敛

- 决定 runtime precision；若仍 f32，删除伪 narrowing；若 large world，建立 f64/origin epoch 到 render f32 的显式转换。
- production import/authoring/compiler 只使用 fallible API和域 policy。
- 建立 source/destination space transform graph 与 projection convention receipt。

### M6：provider extension 与多队列

- 将 RayTracing 从通用 enum promise 改为 negotiated provider extension。
- 建立 graphics/compute/copy queue plan、cross-queue dependency、fence/timeline 与 retirement contract。
- WGPU继续诚实映射为一个 serialized physical queue；D3D12/Vulkan 由独立 provider 实现。

### M7：性能与故障资格

- clean cold/incremental build、resource churn、parallel record、submit、readback、surface resize、device loss/recreate、memory pressure、shutdown/restart。
- 相同场景/画质/硬件对比 Unreal/Fyrox/Bevy/Godot/Unity reference product，不以微基准替代产品帧。
- 记录 p50/p95/p99 frame time、CPU submit、GPU time/bubbles、lock wait、allocations、VRAM peak、readback latency、recovery time 与 dropped work。

## 11. 资格门

| Gate | 当前 | 关闭条件 |
|---|---|---|
| G01 clean checkout 包含全部五 crate 当前能力 | Fail | source/manifest/tests 原子提交并从 clean clone 验证 |
| G02 受管 source manifest/fingerprint 可重现 | Fail | CI 双实现复算一致 |
| G03 `zr_contracts` workspace/dependency 图闭合 | Fail | metadata + targeted check 通过 |
| G04 contracts 保持 pure DTO/trait、无反向依赖 | Partial | random 满足方向；全域 guard 尚未闭合 |
| G05 WGPU device generation 只有一个产品 owner | Fail | graphics 无裸 device/queue owner |
| G06 RHI handle/submission/surface 均有 generation | Partial | 基础已存在；绕过路径必须清零 |
| G07 neutral descriptor 构造即合法 | Fail | candidate/validated 类型分离 |
| G08 device/profile/ticket identity 不可伪造 | Fail | authority-only constructors |
| G09 submission terminal history 有界且可查询 | Partial | 当前底座保留；需产品/故障闭环 |
| G10 普通 buffer/texture readback 可异步完成 | Fail | request/ticket/map/artifact E2E |
| G11 render/game/UI 主线程无 blocking/yield wait | Fail | thread-role guard + async completion |
| G12 queue topology 真实映射 provider | Partial | WGPU truth 已有；多队列 provider 未实现 |
| G13 resource load request 有 ID/owner/budget/cancel | Fail | RuntimeResourceService E2E |
| G14 dependency lease/currentness/reload 原子闭合 | Fail | DAG + generation swap + last-good |
| G15 hot acquire 不竞争全局 authority write lock | Fail | sharded/RCU benchmark证据 |
| G16 resource event 可 snapshot/resync/ack | Fail | gap recovery E2E |
| G17 durability level 跨平台显式且经 fault test | Fail | platform receipt + crash matrix |
| G18 runtime/render precision 边界表达真实转换 | Fail | 删除 no-op receipt 或实现 f64->f32 |
| G19 near-degenerate numeric policy 可配置并 fail-close | Fail | 域 policy + adversarial tests |
| G20 space conversion 具有 source/destination/origin epoch | Fail | typed graph + stale reject |
| G21 RayTracing 只由可执行 provider 宣告 | Fail | extension negotiation + source guard |
| G22 registry/mutex/queue hotspot 有受管 profile | Fail | lock/queue trace 与预算基线 |
| G23 cross-platform/device-loss 恢复无 stale handle | Fail | recreate/retire/fault tests |
| G24 同画质产品性能超过目标基线 | Fail | 受管 benchmark 和原始 trace |

## 12. 首个允许实施的切片

第一步必须只做 M0，不应从 RHI API 美化或锁拆分开始：

1. 冻结当前五 crate 与 random consumer 的准确 source manifest。
2. 补齐 `zr_contracts` workspace/runtime dependency，并保证它只依赖 serde/thiserror 等低层依赖。
3. 将当前 production/tests 一次性纳入可复现提交候选。
4. 在 clean target 运行 workspace metadata、`zr_contracts`、`zr_math`、`zr_resource`、`zr_rhi`、`zr_rhi_wgpu` targeted check/test。
5. 只有 clean build 和 source guard 全绿，才进入 M1/M2；否则后续重构没有稳定基线。

本报告不授权保留旧路径、compat alias 或双 owner。实施时采用 hard cut：同一切片迁移 producer、consumer、tests、docs 与 guard，并删除旧入口。
